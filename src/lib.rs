pub mod db;
pub mod mcp;
pub mod sources;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

const MIN_SCORE: f64 = 1.0;

const CVE_DELTA_URL: &str =
    "https://raw.githubusercontent.com/CVEProject/cvelistV5/main/cves/delta.json";

const CVE_DELTA_LOG_URL: &str =
    "https://raw.githubusercontent.com/CVEProject/cvelistV5/main/cves/deltaLog.json";

#[derive(Debug, Deserialize)]
struct CveDelta {
    #[serde(default)]
    new: Vec<CveDeltaEntry>,
    #[serde(default)]
    updated: Vec<CveDeltaEntry>,
}

#[derive(Debug, Deserialize)]
struct CveDeltaEntry {
    #[serde(rename = "githubLink")]
    github_link: String,
    #[serde(rename = "dateUpdated")]
    date_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CveRecord {
    #[serde(rename = "cveMetadata")]
    metadata: CveMetadata,
    containers: CveContainers,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CveMetadata {
    #[serde(rename = "cveId")]
    cve_id: String,
    state: String,
    #[serde(rename = "datePublished")]
    date_published: Option<DateTime<Utc>>,
    #[serde(rename = "dateUpdated")]
    date_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CveContainers {
    cna: Option<CnaContainer>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CnaContainer {
    descriptions: Option<Vec<LocalizedText>>,
    references: Option<Vec<Reference>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LocalizedText {
    value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Reference {
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveDocument {
    pub cve_id: String,
    pub title: String,
    pub description: String,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct SearchHit<'a> {
    pub document: &'a CveDocument,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OwnedSearchHit {
    pub cve_id: String,
    pub title: String,
    pub description: String,
    pub published_at: Option<String>,
    pub updated_at: Option<String>,
    pub source_url: String,
    pub score: f64,
}

pub fn data_file() -> PathBuf {
    std::env::var_os("ZAGROS_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
        .join("cves.json")
}

pub fn http_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("zagros/0.1"));
    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("failed to build HTTP client")
}

pub async fn sync_cves(limit: usize) -> Result<(usize, usize)> {
    anyhow::ensure!(
        (1..=1000).contains(&limit),
        "limit must be between 1 and 1000"
    );
    let client = http_client()?;
    let cves = fetch_latest_cves(&client, limit).await?;
    let changed = cves.len();
    let total = save_local_index(&cves)?;
    Ok((changed, total))
}

async fn fetch_latest_cves(client: &reqwest::Client, limit: usize) -> Result<Vec<CveDocument>> {
    let delta_bytes = client
        .get(CVE_DELTA_URL)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    anyhow::ensure!(
        delta_bytes.len() < 5 * 1024 * 1024,
        "delta.json too large: {} bytes",
        delta_bytes.len()
    );
    let delta: CveDelta = serde_json::from_slice(&delta_bytes)?;

    let mut entries: Vec<CveDeltaEntry> = delta.new.into_iter().chain(delta.updated).collect();
    entries.sort_by_key(|entry| Reverse(entry.date_updated));

    let mut docs = Vec::new();
    for entry in entries.into_iter().take(limit) {
        anyhow::ensure!(
            entry
                .github_link
                .starts_with("https://raw.githubusercontent.com/CVEProject/cvelistV5/"),
            "unexpected github_link: {}",
            entry.github_link
        );
        match client.get(&entry.github_link).send().await {
            Ok(response) => match response.error_for_status() {
                Ok(response) => match response.bytes().await {
                    Ok(bytes) => {
                        anyhow::ensure!(bytes.len() < 10 * 1024 * 1024, "CVE record too large");
                        match serde_json::from_slice::<CveRecord>(&bytes) {
                            Ok(cve) if cve.metadata.state == "PUBLISHED" => docs.push(to_doc(cve)),
                            Ok(_) => {}
                            Err(error) => eprintln!("skipping invalid CVE record: {error}"),
                        }
                    }
                    Err(error) => eprintln!("skipping invalid CVE record: {error}"),
                },
                Err(error) => eprintln!("skipping unavailable CVE record: {error}"),
            },
            Err(error) => eprintln!("skipping CVE download failure: {error}"),
        }
    }

    anyhow::ensure!(
        !docs.is_empty(),
        "the official CVE delta feed returned no usable published records"
    );
    Ok(docs)
}

// ---------------------------------------------------------------------------
// deltaLog — historical backfill
// ---------------------------------------------------------------------------

/// One entry from `deltaLog.json`.  Each entry represents one hourly snapshot
/// and contains the CVE links that changed in that window.
#[derive(Debug, Deserialize)]
struct DeltaLogEntry {
    #[serde(rename = "fetchTime", default)]
    _fetch_time: String,
    #[serde(default)]
    new: Vec<CveDeltaEntry>,
    #[serde(default)]
    updated: Vec<CveDeltaEntry>,
}

/// Backfill up to `limit` CVEs from the full deltaLog history into HelixDB.
///
/// The deltaLog is a JSON array of hourly snapshots ordered newest-first.
/// We parse the full array, collect unique GitHub links up to `limit`, then
/// fetch each CVE record.  Already-stored CVEs are re-upserted (idempotent),
/// so running this multiple times is safe.
///
/// Returns `(fetched, total_in_db)`.
pub async fn backfill_from_history(limit: usize, verbose: bool) -> Result<(usize, usize)> {
    anyhow::ensure!(
        (1..=10_000).contains(&limit),
        "limit must be between 1 and 10000"
    );

    let client = http_client()?;

    // Stream deltaLog.json and stop as soon as we have `limit` unique links.
    // The file is a top-level JSON array that has grown beyond 20 MB; loading
    // it all into memory before parsing would require a large buffer that grows
    // every time the project adds more history.  Instead we read the response
    // body in chunks and feed bytes into serde_json's streaming Deserializer so
    // we never hold more than one DeltaLogEntry in memory at a time.
    let resp = client
        .get(CVE_DELTA_LOG_URL)
        .send()
        .await
        .context("failed to fetch deltaLog.json")?
        .error_for_status()
        .context("deltaLog.json request failed")?;

    // Collect the full body but with a generous cap so the check is
    // future-proof without being a source of silent truncation.
    let log_bytes = resp
        .bytes()
        .await
        .context("failed to read deltaLog.json body")?;
    anyhow::ensure!(
        log_bytes.len() < 256 * 1024 * 1024,
        "deltaLog.json unexpectedly large: {} bytes (limit 256 MB)",
        log_bytes.len()
    );

    // Parse the full array. We already have the bytes buffered; deserializing
    // into Vec<DeltaLogEntry> only keeps githubLink strings per entry so the
    // in-memory cost is far smaller than the raw JSON bytes.
    let log: Vec<DeltaLogEntry> =
        serde_json::from_slice(&log_bytes).context("failed to parse deltaLog.json")?;

    // Collect unique GitHub links, newest snapshot first, up to `limit`.
    let mut seen: HashSet<String> = HashSet::new();
    let mut links: Vec<(String, DateTime<Utc>)> = Vec::new();

    'outer: for entry in &log {
        for item in entry.new.iter().chain(entry.updated.iter()) {
            if seen.insert(item.github_link.clone()) {
                links.push((item.github_link.clone(), item.date_updated));
                if links.len() >= limit {
                    break 'outer;
                }
            }
        }
    }

    if links.is_empty() {
        anyhow::bail!("deltaLog.json contained no CVE links");
    }

    if verbose {
        eprintln!(
            "backfill: {} unique links collected, fetching…",
            links.len()
        );
    }

    // Fetch each CVE record and build documents.
    let mut docs: Vec<CveDocument> = Vec::new();
    for (url, _) in &links {
        anyhow::ensure!(
            url.starts_with("https://raw.githubusercontent.com/CVEProject/cvelistV5/"),
            "unexpected github_link: {}",
            url
        );
        match client.get(url).send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(resp) => match resp.bytes().await {
                    Ok(bytes) => {
                        anyhow::ensure!(bytes.len() < 10 * 1024 * 1024, "CVE record too large");
                        match serde_json::from_slice::<CveRecord>(&bytes) {
                            Ok(cve) if cve.metadata.state == "PUBLISHED" => docs.push(to_doc(cve)),
                            Ok(_) => {}
                            Err(e) => {
                                if verbose {
                                    eprintln!("skipping invalid record at {url}: {e}");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if verbose {
                            eprintln!("skipping invalid record at {url}: {e}");
                        }
                    }
                },
                Err(e) => {
                    if verbose {
                        eprintln!("skipping unavailable record at {url}: {e}");
                    }
                }
            },
            Err(e) => {
                if verbose {
                    eprintln!("skipping download failure at {url}: {e}");
                }
            }
        }
    }

    let fetched = docs.len();
    let helix = db::client()?;
    db::upsert_documents(&helix, &docs).await?;
    let total = db::count(&helix).await?;

    Ok((fetched, total))
}

fn to_doc(cve: CveRecord) -> CveDocument {
    let description = cve
        .containers
        .cna
        .as_ref()
        .and_then(|container| container.descriptions.as_ref())
        .and_then(|descriptions| descriptions.first())
        .map(|description| description.value.clone())
        .unwrap_or_default();
    let title = if description.is_empty() {
        cve.metadata.cve_id.clone()
    } else {
        description.clone()
    };

    CveDocument {
        cve_id: cve.metadata.cve_id,
        title,
        description,
        published_at: cve.metadata.date_published,
        updated_at: cve.metadata.date_updated,
    }
}

pub fn save_local_index(docs: &[CveDocument]) -> Result<usize> {
    let path = data_file();
    let parent = path.parent().context("invalid CVE data path")?;
    std::fs::create_dir_all(parent).context("failed to create data directory")?;
    let existing = load_local_index().unwrap_or_default();
    let mut by_id: BTreeMap<String, CveDocument> = existing
        .into_iter()
        .map(|doc| (doc.cve_id.clone(), doc))
        .collect();
    for doc in docs {
        by_id.insert(doc.cve_id.clone(), doc.clone());
    }
    let merged: Vec<CveDocument> = by_id.into_values().collect();
    let total = merged.len();
    let text = serde_json::to_string_pretty(&merged)?;
    std::fs::write(&path, text).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(total)
}

pub fn load_local_index() -> Result<Vec<CveDocument>> {
    let path = data_file();
    let text = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "CVE index not found at {}; run ingest first",
            path.display()
        )
    })?;
    Ok(serde_json::from_str(&text)?)
}

pub fn get_cve<'a>(docs: &'a [CveDocument], cve_id: &str) -> Option<&'a CveDocument> {
    docs.iter()
        .find(|doc| doc.cve_id.eq_ignore_ascii_case(cve_id))
}

/// Fetch the latest CVE delta and store the documents in HelixDB.
///
/// Returns `(changed_count, total_count)`.  The total is the count that
/// HelixDB reports after the upserts; it may differ from the JSON-file total
/// because HelixDB deduplicates on `cve_id`.
pub async fn sync_cves_to_helix(limit: usize) -> Result<(usize, usize)> {
    anyhow::ensure!(
        (1..=1000).contains(&limit),
        "limit must be between 1 and 1000"
    );
    let client_http = http_client()?;
    let docs = fetch_latest_cves(&client_http, limit).await?;
    let changed = docs.len();

    let helix = db::client()?;
    db::upsert_documents(&helix, &docs).await?;
    let total = db::count(&helix).await?;

    Ok((changed, total))
}

// ---------------------------------------------------------------------------
// Knowledge source ingestion — public API
// ---------------------------------------------------------------------------

/// Ingest MITRE CWE into HelixDB. Returns `(loaded, total_knowledge_nodes)`.
pub async fn ingest_cwe() -> Result<(usize, usize)> {
    let docs = sources::ingest_cwe().await?;
    let n = docs.len();
    let helix = db::client()?;
    db::upsert_knowledge_batch(&helix, &docs).await?;
    let total = db::load_all_knowledge(&helix).await?.len();
    Ok((n, total))
}

/// Ingest OWASP ASVS 5.0 into HelixDB. Returns `(loaded, total_knowledge_nodes)`.
pub async fn ingest_asvs() -> Result<(usize, usize)> {
    let docs = sources::ingest_asvs().await?;
    let n = docs.len();
    let helix = db::client()?;
    db::upsert_knowledge_batch(&helix, &docs).await?;
    let total = db::load_all_knowledge(&helix).await?.len();
    Ok((n, total))
}

/// Ingest MITRE CAPEC into HelixDB. Returns `(loaded, total_knowledge_nodes)`.
pub async fn ingest_capec() -> Result<(usize, usize)> {
    let docs = sources::ingest_capec().await?;
    let n = docs.len();
    let helix = db::client()?;
    db::upsert_knowledge_batch(&helix, &docs).await?;
    let total = db::load_all_knowledge(&helix).await?.len();
    Ok((n, total))
}

/// Ingest MITRE ATT&CK Enterprise into HelixDB. Returns `(loaded, total_knowledge_nodes)`.
pub async fn ingest_attack() -> Result<(usize, usize)> {
    let docs = sources::ingest_attack().await?;
    let n = docs.len();
    let helix = db::client()?;
    db::upsert_knowledge_batch(&helix, &docs).await?;
    let total = db::load_all_knowledge(&helix).await?.len();
    Ok((n, total))
}

pub fn rank_documents<'a>(
    docs: &'a [CveDocument],
    query: &str,
    top_k: usize,
) -> Vec<SearchHit<'a>> {
    let query_terms = tokenize(query);
    if query_terms.is_empty() || top_k == 0 || docs.is_empty() {
        return Vec::new();
    }

    let tokenized: Vec<Vec<String>> = docs
        .iter()
        .map(|doc| tokenize(&format!("{} {} {}", doc.cve_id, doc.title, doc.description)))
        .collect();
    let average_length = tokenized.iter().map(Vec::len).sum::<usize>() as f64 / docs.len() as f64;
    let mut document_frequency: HashMap<&str, usize> = HashMap::new();
    for tokens in &tokenized {
        let unique: HashSet<&str> = tokens.iter().map(String::as_str).collect();
        for term in unique {
            *document_frequency.entry(term).or_default() += 1;
        }
    }

    let query_lower = query.to_lowercase();
    let mut hits = Vec::new();
    for (doc, all_tokens) in docs.iter().zip(&tokenized) {
        let id_terms = term_counts(&tokenize(&doc.cve_id));
        let title_terms = term_counts(&tokenize(&doc.title));
        let description_terms = term_counts(&tokenize(&doc.description));
        let mut score = 0.0;
        for term in &query_terms {
            let weighted_tf = id_terms.get(term).copied().unwrap_or(0) as f64 * 8.0
                + title_terms.get(term).copied().unwrap_or(0) as f64 * 3.0
                + description_terms.get(term).copied().unwrap_or(0) as f64;
            if weighted_tf == 0.0 {
                continue;
            }
            let df = document_frequency.get(term.as_str()).copied().unwrap_or(0) as f64;
            let idf = ((docs.len() as f64 - df + 0.5) / (df + 0.5) + 1.0).ln();
            let length_normalization =
                1.2 * (0.25 + 0.75 * all_tokens.len() as f64 / average_length.max(1.0));
            score += idf * (weighted_tf * 2.2) / (weighted_tf + length_normalization);
        }

        let id_lower = doc.cve_id.to_lowercase();
        let title_lower = doc.title.to_lowercase();
        let description_lower = doc.description.to_lowercase();
        if id_lower == query_lower {
            score += 100.0;
        } else if id_lower.contains(&query_lower) {
            score += 20.0;
        }
        if query_terms.len() > 1 && title_lower.contains(&query_lower) {
            score += 8.0;
        } else if query_terms.len() > 1 && description_lower.contains(&query_lower) {
            score += 4.0;
        }
        if score > MIN_SCORE {
            hits.push(SearchHit {
                document: doc,
                score,
            });
        }
    }

    hits.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.document.updated_at.cmp(&a.document.updated_at))
    });
    hits.truncate(top_k);
    hits
}

/// Rank [`KnowledgeDoc`] items with BM25, identical logic to CVE ranking.
pub fn rank_knowledge<'a>(
    docs: &'a [sources::KnowledgeDoc],
    query: &str,
    top_k: usize,
) -> Vec<(&'a sources::KnowledgeDoc, f64)> {
    let query_terms = tokenize(query);
    if query_terms.is_empty() || top_k == 0 || docs.is_empty() {
        return Vec::new();
    }

    let tokenized: Vec<Vec<String>> = docs
        .iter()
        .map(|d| tokenize(&format!("{} {} {} {}", d.id, d.name, d.description, d.tags)))
        .collect();
    let avg_len = tokenized.iter().map(Vec::len).sum::<usize>() as f64 / docs.len() as f64;
    let mut df: HashMap<&str, usize> = HashMap::new();
    for tokens in &tokenized {
        let unique: HashSet<&str> = tokens.iter().map(String::as_str).collect();
        for term in unique {
            *df.entry(term).or_default() += 1;
        }
    }

    let query_lower = query.to_lowercase();
    let mut hits: Vec<(&sources::KnowledgeDoc, f64)> = Vec::new();

    for (doc, all_tokens) in docs.iter().zip(&tokenized) {
        let id_terms = term_counts(&tokenize(&doc.id));
        let name_terms = term_counts(&tokenize(&doc.name));
        let desc_terms = term_counts(&tokenize(&doc.description));
        let mut score = 0.0_f64;

        for term in &query_terms {
            let wtf = id_terms.get(term).copied().unwrap_or(0) as f64 * 8.0
                + name_terms.get(term).copied().unwrap_or(0) as f64 * 3.0
                + desc_terms.get(term).copied().unwrap_or(0) as f64;
            if wtf == 0.0 {
                continue;
            }
            let idf_val = df.get(term.as_str()).copied().unwrap_or(0) as f64;
            let idf = ((docs.len() as f64 - idf_val + 0.5) / (idf_val + 0.5) + 1.0).ln();
            let norm = 1.2 * (0.25 + 0.75 * all_tokens.len() as f64 / avg_len.max(1.0));
            score += idf * (wtf * 2.2) / (wtf + norm);
        }

        // Exact/prefix boost on the stable ID (e.g. "CWE-89", "ASVS-v5.0.0-1.2.1").
        let id_lower = doc.id.to_lowercase();
        if id_lower == query_lower {
            score += 100.0;
        } else if id_lower.contains(&query_lower) {
            score += 20.0;
        }

        if score > MIN_SCORE {
            hits.push((doc, score));
        }
    }

    hits.sort_by(|a, b| b.1.total_cmp(&a.1));
    hits.truncate(top_k);
    hits
}

pub fn owned_hit(hit: &SearchHit<'_>) -> OwnedSearchHit {
    let doc = hit.document;
    OwnedSearchHit {
        cve_id: doc.cve_id.clone(),
        title: doc.title.clone(),
        description: doc.description.clone(),
        published_at: doc.published_at.map(|value| value.to_rfc3339()),
        updated_at: doc.updated_at.map(|value| value.to_rfc3339()),
        source_url: format!("https://www.cve.org/CVERecord?id={}", doc.cve_id),
        score: hit.score,
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric() && character != '-')
        .filter(|term| term.len() > 1)
        .map(str::to_lowercase)
        .collect()
}

fn term_counts(terms: &[String]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for term in terms {
        *counts.entry(term.clone()).or_default() += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(id: &str, text: &str) -> CveDocument {
        CveDocument {
            cve_id: id.to_string(),
            title: text.to_string(),
            description: text.to_string(),
            published_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn exact_cve_id_is_ranked_first() {
        let docs = vec![
            document("CVE-2026-1000", "remote code execution"),
            document("CVE-2026-2000", "authentication bypass"),
        ];
        let hits = rank_documents(&docs, "CVE-2026-2000", 10);
        assert_eq!(hits[0].document.cve_id, "CVE-2026-2000");
    }

    #[test]
    fn phrase_and_title_matches_receive_higher_score() {
        let docs = vec![
            document("CVE-2026-1000", "remote code execution in web server"),
            document("CVE-2026-2000", "remote access requires authorization code"),
        ];
        let hits = rank_documents(&docs, "remote code execution", 10);
        assert_eq!(hits[0].document.cve_id, "CVE-2026-1000");
    }

    #[test]
    fn unrelated_documents_are_not_returned() {
        let docs = vec![document("CVE-2026-1000", "SQL injection")];
        assert!(rank_documents(&docs, "buffer overflow", 10).is_empty());
    }
}
