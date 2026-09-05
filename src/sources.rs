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
use anyhow::{Context, Result, ensure};
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

    parse_cwe_xml(&xml_bytes)
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

    parse_asvs_csv(&text)
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

    parse_capec_xml(&bytes)
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

    parse_attack_stix(&bytes)
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
        });
    }

    Ok(docs)
}
