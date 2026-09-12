/// Ingestion of authoritative security knowledge sources.
///
/// All sources produce [`KnowledgeDoc`] values and are stored in HelixDB as
/// `Knowledge` nodes alongside the existing `Cve` nodes.
///
/// ## Sources
///
/// | Source | Node label | URL |
/// |--------|-----------|-----|
/// | MITRE CWE 4.x | `Knowledge` | https://cwe.mitre.org/data/xml/cwec_latest.xml.zip |
/// | OWASP ASVS 5.0 | `Knowledge` | OWASP GitHub release CSV |
/// | MITRE CAPEC 3.x | `Knowledge` | https://capec.mitre.org/data/xml/capec_latest.xml |
/// | MITRE ATT&CK Enterprise | `Knowledge` | attack-stix-data GitHub |
use crate::http_client;
use crate::provenance::{
    Provenance, append_source_manifest, build_provenance, normalized_record_hash,
    preserve_raw_snapshot, sha256_hex,
};
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use quick_xml::escape::unescape;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};

// ── Shared document type ─────────────────────────────────────────────────────

/// A single retrievable knowledge chunk from any authoritative source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDoc {
    /// Stable identifier: `CWE-89`, `ASVS-v5.0.0-1.2.5`, `CAPEC-66`, `ATT&CK-T1190`.
    pub id: String,
    /// Short human-readable name.
    pub name: String,
    /// Full descriptive text used for BM25 search.
    pub description: String,
    /// Source name: `cwe`, `asvs`, `capec`, `attack`.
    pub source: String,
    /// Optional URL pointing to the authoritative record.
    pub url: String,
    /// Optional extra context (platforms, level, tactic, etc.).
    pub tags: String,
    /// Provenance required for newly ingested records; legacy rows get a migration-safe fallback.
    #[serde(default)]
    pub provenance: Provenance,
}

fn apply_provenance(
    docs: &mut [KnowledgeDoc],
    source: &str,
    source_version: &str,
    retrieved_at: DateTime<Utc>,
) {
    for doc in docs {
        let hash = normalized_record_hash(&doc.id, &doc.name, &doc.description, &doc.tags);
        doc.provenance =
            build_provenance(source, source_version, &doc.url, retrieved_at, hash, None);
    }
}

fn record_source_snapshot(
    docs: &[KnowledgeDoc],
    raw_bytes: &[u8],
    raw_extension: &str,
    source: &str,
    retrieved_at: DateTime<Utc>,
) {
    let raw_hash = sha256_hex(raw_bytes);
    if let Some(first) = docs.first() {
        append_source_manifest(&first.provenance, &raw_hash, docs.len());
    }
    preserve_raw_snapshot(source, raw_bytes, raw_extension, retrieved_at);
}

fn xml_catalog_version(xml: &[u8], root_local_name: &[u8]) -> Option<String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_reader(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if e.local_name().as_ref() == root_local_name =>
            {
                for attr in e.attributes().flatten() {
                    if attr.key.local_name().as_ref() == b"Version" {
                        return Some(String::from_utf8_lossy(&attr.value).into_owned());
                    }
                }
                return None;
            }
            Ok(Event::Eof) => return None,
            Err(_) => return None,
            _ => {}
        }
        buf.clear();
    }
}

fn attack_source_version(json: &[u8]) -> String {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(json) else {
        return "master".to_string();
    };
    value
        .get("objects")
        .and_then(|v| v.as_array())
        .and_then(|objects| {
            objects.iter().find_map(|obj| {
                let is_collection =
                    obj.get("type").and_then(|v| v.as_str()) == Some("x-mitre-collection");
                is_collection
                    .then(|| obj.get("x_mitre_version").and_then(|v| v.as_str()))
                    .flatten()
            })
        })
        .unwrap_or("master")
        .to_string()
}

// ── CWE ──────────────────────────────────────────────────────────────────────

const CWE_ZIP_URL: &str = "https://cwe.mitre.org/data/xml/cwec_latest.xml.zip";

/// Download and parse the CWE XML ZIP.  Returns one [`KnowledgeDoc`] per
/// `<Weakness>` entry that has a description.
pub async fn ingest_cwe() -> Result<Vec<KnowledgeDoc>> {
    let client = http_client()?;
    let bytes = client
        .get(CWE_ZIP_URL)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .context("failed to fetch CWE ZIP")?
        .error_for_status()
        .context("CWE ZIP request failed")?
        .bytes()
        .await
        .context("failed to read CWE ZIP bytes")?;
    ensure!(
        bytes.len() < 50 * 1024 * 1024,
        "CWE ZIP too large: {} bytes",
        bytes.len()
    );

    // Decompress the single XML file from the ZIP in memory.
    let cursor = Cursor::new(bytes.as_ref());
    let mut archive = zip::ZipArchive::new(cursor).context("failed to open CWE ZIP archive")?;
    let mut xml_bytes = Vec::with_capacity(4 * 1024 * 1024);
    archive
        .by_index(0)
        .context("CWE ZIP is empty")?
        .read_to_end(&mut xml_bytes)
        .context("failed to decompress CWE XML")?;
    ensure!(
        xml_bytes.len() < 50 * 1024 * 1024,
        "CWE XML too large after decompression: {} bytes",
        xml_bytes.len()
    );

    let retrieved_at = Utc::now();
    let version = xml_catalog_version(&xml_bytes, b"Weakness_Catalog")
        .unwrap_or_else(|| "latest".to_string());
    let mut docs = parse_cwe_xml(&xml_bytes)?;
    apply_provenance(&mut docs, "cwe", &version, retrieved_at);
    record_source_snapshot(&docs, bytes.as_ref(), "zip", "cwe", retrieved_at);
    Ok(docs)
}

fn parse_cwe_xml(xml: &[u8]) -> Result<Vec<KnowledgeDoc>> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut docs = Vec::new();
    let mut current_id = String::new();
    let mut current_name = String::new();
    let mut in_description = false;
    let mut desc_buf = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if e.local_name().as_ref() == b"Weakness" =>
            {
                // Flush any previous weakness.
                if !current_id.is_empty() && !desc_buf.is_empty() {
                    docs.push(KnowledgeDoc {
                        id: format!("CWE-{current_id}"),
                        name: current_name.clone(),
                        description: desc_buf.trim().to_string(),
                        source: "cwe".into(),
                        url: format!("https://cwe.mitre.org/data/definitions/{current_id}.html"),
                        tags: String::new(),
                        provenance: Provenance::default(),
                    });
                }
                // Reset for next weakness.
                current_id.clear();
                current_name.clear();
                desc_buf.clear();
                in_description = false;

                for attr in e.attributes().flatten() {
                    match attr.key.local_name().as_ref() {
                        b"ID" => {
                            current_id = String::from_utf8_lossy(&attr.value).into_owned();
                        }
                        b"Name" => {
                            current_name = String::from_utf8_lossy(&attr.value).into_owned();
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"Description" => {
                in_description = true;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Description" => {
                in_description = false;
            }
            Ok(Event::Text(ref e)) if in_description => {
                if !desc_buf.is_empty() {
                    desc_buf.push(' ');
                }
                let raw = match std::str::from_utf8(e) {
                    Ok(s) => s,
                    Err(err) => {
                        eprintln!("[zagros] warning: invalid UTF-8 in XML text, skipping: {err}");
                        ""
                    }
                };
                match unescape(raw) {
                    Ok(s) => desc_buf.push_str(&s),
                    Err(err) => {
                        eprintln!("[zagros] warning: XML unescape failed, skipping entity: {err}")
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("CWE XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    // Flush the last weakness.
    if !current_id.is_empty() && !desc_buf.is_empty() {
        docs.push(KnowledgeDoc {
            id: format!("CWE-{current_id}"),
            name: current_name,
            description: desc_buf.trim().to_string(),
            source: "cwe".into(),
            url: format!("https://cwe.mitre.org/data/definitions/{current_id}.html"),
            tags: String::new(),
            provenance: Provenance::default(),
        });
    }

    Ok(docs)
}

// ── OWASP ASVS ───────────────────────────────────────────────────────────────

const ASVS_CSV_URL: &str = "https://raw.githubusercontent.com/OWASP/ASVS/v5.0.0/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.csv";

/// Download and parse the OWASP ASVS 5.0.0 CSV.
/// Each row becomes one [`KnowledgeDoc`] keyed by its requirement ID.
pub async fn ingest_asvs() -> Result<Vec<KnowledgeDoc>> {
    let client = http_client()?;
    let bytes = client
        .get(ASVS_CSV_URL)
        .send()
        .await
        .context("failed to fetch ASVS CSV")?
        .error_for_status()
        .context("ASVS CSV request failed")?
        .bytes()
        .await
        .context("failed to read ASVS CSV bytes")?;
    ensure!(
        bytes.len() < 10 * 1024 * 1024,
        "ASVS CSV too large: {} bytes",
        bytes.len()
    );

    let text = String::from_utf8(bytes.to_vec()).context("ASVS CSV is not valid UTF-8")?;

    let retrieved_at = Utc::now();
    let mut docs = parse_asvs_csv(&text)?;
    apply_provenance(&mut docs, "asvs", "5.0.0", retrieved_at);
    record_source_snapshot(&docs, bytes.as_ref(), "csv", "asvs", retrieved_at);
    Ok(docs)
}

/// ASVS CSV columns: chapter_id, chapter_name, section_id, section_name,
///                   req_id, req_description, L
#[derive(Debug, Deserialize)]
struct AsvsRow {
    #[serde(rename = "chapter_id")]
    _chapter_id: String,
    chapter_name: String,
    #[serde(rename = "section_id")]
    _section_id: String,
    section_name: String,
    req_id: String,
    req_description: String,
    #[serde(rename = "L")]
    level: String,
}

fn parse_asvs_csv(text: &str) -> Result<Vec<KnowledgeDoc>> {
    let mut rdr = csv::Reader::from_reader(text.as_bytes());
    let mut docs = Vec::new();

    for result in rdr.deserialize::<AsvsRow>() {
        let row = match result {
            Ok(r) => r,
            Err(_) => continue, // skip malformed rows
        };
        if row.req_id.is_empty() || row.req_description.is_empty() {
            continue;
        }
        let tags = format!(
            "{} | {} | Level {}",
            row.chapter_name, row.section_name, row.level
        );
        docs.push(KnowledgeDoc {
            id: format!("ASVS-v5.0.0-{}", row.req_id),
            name: format!(
                "{}: {}",
                row.req_id,
                &row.req_description[..row.req_description.len().min(80)]
            ),
            description: row.req_description.clone(),
            source: "asvs".into(),
            url: "https://github.com/OWASP/ASVS/tree/v5.0.0".into(),
            tags,
            provenance: Provenance::default(),
        });
    }

    Ok(docs)
}

// ── CAPEC ─────────────────────────────────────────────────────────────────────

const CAPEC_XML_URL: &str = "https://capec.mitre.org/data/xml/capec_latest.xml";

/// Download and parse the CAPEC XML.
/// Returns one [`KnowledgeDoc`] per `<Attack_Pattern>` with a description.
pub async fn ingest_capec() -> Result<Vec<KnowledgeDoc>> {
    let client = http_client()?;
    let bytes = client
        .get(CAPEC_XML_URL)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .context("failed to fetch CAPEC XML")?
        .error_for_status()
        .context("CAPEC XML request failed")?
        .bytes()
        .await
        .context("failed to read CAPEC XML bytes")?;
    ensure!(
        bytes.len() < 50 * 1024 * 1024,
        "CAPEC XML too large: {} bytes",
        bytes.len()
    );

    let retrieved_at = Utc::now();
    let version = xml_catalog_version(&bytes, b"Attack_Pattern_Catalog")
        .unwrap_or_else(|| "latest".to_string());
    let mut docs = parse_capec_xml(&bytes)?;
    apply_provenance(&mut docs, "capec", &version, retrieved_at);
    record_source_snapshot(&docs, bytes.as_ref(), "xml", "capec", retrieved_at);
    Ok(docs)
}

fn parse_capec_xml(xml: &[u8]) -> Result<Vec<KnowledgeDoc>> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut docs = Vec::new();
    let mut current_id = String::new();
    let mut current_name = String::new();
    let mut current_status = String::new();
    let mut in_description = false;
    let mut desc_buf = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if e.local_name().as_ref() == b"Attack_Pattern" =>
            {
                if !current_id.is_empty() && !desc_buf.is_empty() {
                    if matches!(current_status.as_str(), "Deprecated" | "Obsolete") {
                        current_id.clear();
                        current_name.clear();
                        current_status.clear();
                        desc_buf.clear();
                    } else {
                        docs.push(build_capec_doc(&current_id, &current_name, &desc_buf));
                    }
                }
                current_id.clear();
                current_name.clear();
                current_status.clear();
                desc_buf.clear();
                in_description = false;

                for attr in e.attributes().flatten() {
                    match attr.key.local_name().as_ref() {
                        b"ID" => current_id = String::from_utf8_lossy(&attr.value).into_owned(),
                        b"Name" => current_name = String::from_utf8_lossy(&attr.value).into_owned(),
                        b"Status" => {
                            current_status = String::from_utf8_lossy(&attr.value).into_owned()
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"Description" => {
                // Only capture the top-level description, not nested ones.
                in_description = desc_buf.is_empty();
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Description" => {
                in_description = false;
            }
            Ok(Event::Text(ref e)) if in_description => {
                if !desc_buf.is_empty() {
                    desc_buf.push(' ');
                }
                let raw = match std::str::from_utf8(e) {
                    Ok(s) => s,
                    Err(err) => {
                        eprintln!("[zagros] warning: invalid UTF-8 in XML text, skipping: {err}");
                        ""
                    }
                };
                match unescape(raw) {
                    Ok(s) => desc_buf.push_str(&s),
                    Err(err) => {
                        eprintln!("[zagros] warning: XML unescape failed, skipping entity: {err}")
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("CAPEC XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    if !current_id.is_empty()
        && !desc_buf.is_empty()
        && !matches!(current_status.as_str(), "Deprecated" | "Obsolete")
    {
        docs.push(build_capec_doc(&current_id, &current_name, &desc_buf));
    }

    Ok(docs)
}

fn build_capec_doc(id: &str, name: &str, description: &str) -> KnowledgeDoc {
    KnowledgeDoc {
        id: format!("CAPEC-{id}"),
        name: name.to_string(),
        description: description.trim().to_string(),
        source: "capec".into(),
        url: format!("https://capec.mitre.org/data/definitions/{id}.html"),
        tags: String::new(),
        provenance: Provenance::default(),
    }
}

// ── ATT&CK ───────────────────────────────────────────────────────────────────

const ATTACK_URL: &str = "https://raw.githubusercontent.com/mitre-attack/attack-stix-data/master/enterprise-attack/enterprise-attack.json";

/// Download and parse the MITRE ATT&CK Enterprise STIX bundle.
/// Returns one [`KnowledgeDoc`] per non-deprecated, non-revoked technique.
pub async fn ingest_attack() -> Result<Vec<KnowledgeDoc>> {
    let client = http_client()?;
    let bytes = client
        .get(ATTACK_URL)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .context("failed to fetch ATT&CK STIX bundle")?
        .error_for_status()
        .context("ATT&CK STIX request failed")?
        .bytes()
        .await
        .context("failed to read ATT&CK STIX bytes")?;
    ensure!(
        bytes.len() < 100 * 1024 * 1024,
        "ATT&CK STIX too large: {} bytes",
        bytes.len()
    );

    let retrieved_at = Utc::now();
    let version = attack_source_version(&bytes);
    let mut docs = parse_attack_stix(&bytes)?;
    apply_provenance(&mut docs, "attack", &version, retrieved_at);
    record_source_snapshot(&docs, bytes.as_ref(), "json", "attack", retrieved_at);
    Ok(docs)
}

#[derive(Debug, Deserialize)]
struct StixBundle {
    objects: Vec<StixObject>,
}

#[derive(Debug, Deserialize)]
struct StixObject {
    #[serde(rename = "type")]
    obj_type: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    revoked: bool,
    #[serde(default)]
    x_mitre_deprecated: bool,
    #[serde(default)]
    external_references: Vec<ExternalReference>,
    #[serde(default)]
    kill_chain_phases: Vec<KillChainPhase>,
    #[serde(default)]
    x_mitre_platforms: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExternalReference {
    #[serde(default)]
    source_name: String,
    #[serde(default)]
    external_id: String,
    #[serde(default)]
    url: String,
}

#[derive(Debug, Deserialize)]
struct KillChainPhase {
    #[serde(default)]
    phase_name: String,
}

fn parse_attack_stix(json: &[u8]) -> Result<Vec<KnowledgeDoc>> {
    let bundle: StixBundle =
        serde_json::from_slice(json).context("failed to parse ATT&CK STIX JSON")?;

    let mut docs = Vec::new();

    for obj in &bundle.objects {
        if obj.obj_type != "attack-pattern" || obj.revoked || obj.x_mitre_deprecated {
            continue;
        }
        if obj.description.is_empty() {
            continue;
        }

        // Find the ATT&CK external reference (T#### identifier).
        let ext = obj
            .external_references
            .iter()
            .find(|r| r.source_name == "mitre-attack");
        let (ext_id, url) = match ext {
            Some(r) => (r.external_id.clone(), r.url.clone()),
            None => continue,
        };

        let tactics: Vec<&str> = obj
            .kill_chain_phases
            .iter()
            .map(|k| k.phase_name.as_str())
            .collect();
        let platforms = obj.x_mitre_platforms.join(", ");
        let tags = format!("tactics: {} | platforms: {}", tactics.join(", "), platforms);

        docs.push(KnowledgeDoc {
            id: format!("ATT&CK-{ext_id}"),
            name: obj.name.clone(),
            description: obj.description.clone(),
            source: "attack".into(),
            url,
            tags,
            provenance: Provenance::default(),
        });
    }

    Ok(docs)
}
