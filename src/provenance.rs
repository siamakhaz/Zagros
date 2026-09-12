use chrono::{DateTime, Utc};
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    Authoritative,
    Curated,
    Secondary,
    Community,
    #[default]
    LegacyUnknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Provenance {
    pub publisher: String,
    pub source_name: String,
    pub source_version: String,
    pub canonical_url: String,
    pub license: String,
    pub retrieved_at: String,
    pub upstream_updated_at: Option<String>,
    pub content_sha256: String,
    pub trust_tier: TrustTier,
}

#[derive(Debug, Clone)]
pub struct SourceDefinition {
    pub publisher: &'static str,
    pub source_name: &'static str,
    pub license: &'static str,
    pub trust_tier: TrustTier,
}

pub fn source_definition(source: &str) -> Option<SourceDefinition> {
    match source {
        "cve" => Some(SourceDefinition {
            publisher: "CVE Program",
            source_name: "CVE List V5",
            license: "CVE Program Terms of Use",
            trust_tier: TrustTier::Authoritative,
        }),
        "cwe" => Some(SourceDefinition {
            publisher: "The MITRE Corporation",
            source_name: "CWE",
            license: "MITRE CWE Terms of Use",
            trust_tier: TrustTier::Authoritative,
        }),
        "asvs" => Some(SourceDefinition {
            publisher: "OWASP Foundation",
            source_name: "OWASP ASVS",
            license: "CC BY-SA 4.0",
            trust_tier: TrustTier::Authoritative,
        }),
        "capec" => Some(SourceDefinition {
            publisher: "The MITRE Corporation",
            source_name: "CAPEC",
            license: "MITRE CAPEC Terms of Use",
            trust_tier: TrustTier::Authoritative,
        }),
        "attack" => Some(SourceDefinition {
            publisher: "The MITRE Corporation",
            source_name: "MITRE ATT&CK Enterprise",
            license: "MITRE ATT&CK Terms of Use",
            trust_tier: TrustTier::Authoritative,
        }),
        _ => None,
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
pub fn normalized_record_hash(id: &str, name: &str, description: &str, tags: &str) -> String {
    let normalized = format!(
        "{}\n{}\n{}\n{}",
        id.trim(),
        name.trim(),
        description.split_whitespace().collect::<Vec<_>>().join(" "),
        tags.split_whitespace().collect::<Vec<_>>().join(" ")
    );
    sha256_hex(normalized.as_bytes())
}

pub fn build_provenance(
    source: &str,
    source_version: &str,
    canonical_url: &str,
    retrieved_at: DateTime<Utc>,
    content_sha256: String,
    upstream_updated_at: Option<DateTime<Utc>>,
) -> Provenance {
    let definition = source_definition(source).unwrap_or(SourceDefinition {
        publisher: "Unknown",
        source_name: "Unknown",
        license: "Unknown",
        trust_tier: TrustTier::LegacyUnknown,
    });
    Provenance {
        publisher: definition.publisher.to_string(),
        source_name: definition.source_name.to_string(),
        source_version: source_version.to_string(),
        canonical_url: canonical_url.to_string(),
        license: definition.license.to_string(),
        retrieved_at: retrieved_at.to_rfc3339(),
        upstream_updated_at: upstream_updated_at.map(|v| v.to_rfc3339()),
        content_sha256,
        trust_tier: definition.trust_tier,
    }
}

pub fn legacy_provenance(source: &str, canonical_url: &str) -> Provenance {
    build_provenance(
        source,
        "legacy-unknown",
        canonical_url,
        Utc::now(),
        String::new(),
        None,
    )
}

pub fn append_source_manifest(provenance: &Provenance, raw_sha256: &str, record_count: usize) {
    let data_dir = std::env::var("ZAGROS_DATA_DIR").unwrap_or_else(|_| "data".to_string());
    let data_dir = PathBuf::from(data_dir);
    if let Err(error) = create_dir_all(&data_dir) {
        eprintln!("[zagros] failed to create manifest directory: {error}");
        return;
    }

    let event = serde_json::json!({
        "event": "source_snapshot",
        "publisher": provenance.publisher,
        "source": provenance.source_name,
        "source_version": provenance.source_version,
        "canonical_url": provenance.canonical_url,
        "license": provenance.license,
        "retrieved_at": provenance.retrieved_at,
        "raw_sha256": raw_sha256,
        "record_count": record_count,
        "trust_tier": provenance.trust_tier,
    });

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_dir.join("source-manifest.jsonl"))
    {
        Ok(mut file) => {
            if let Err(error) = writeln!(file, "{event}") {
                eprintln!("[zagros] failed to append source manifest: {error}");
            }
        }
        Err(error) => eprintln!("[zagros] failed to open source manifest: {error}"),
    }
}

pub fn preserve_raw_snapshot(
    source: &str,
    bytes: &[u8],
    extension: &str,
    retrieved_at: DateTime<Utc>,
) {
    let enabled = std::env::var("ZAGROS_PRESERVE_RAW_SOURCES")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);
    if !enabled {
        return;
    }

    let data_dir = std::env::var("ZAGROS_DATA_DIR").unwrap_or_else(|_| "data".to_string());
    let hash = sha256_hex(bytes);
    let stamp = retrieved_at.format("%Y%m%dT%H%M%SZ");
    let dir = PathBuf::from(data_dir).join("snapshots").join(source);
    if let Err(error) = create_dir_all(&dir) {
        eprintln!("[zagros] failed to create snapshot directory: {error}");
        return;
    }

    let path = dir.join(format!("{stamp}-{hash}.{extension}"));
    if let Err(error) = std::fs::write(path, bytes) {
        eprintln!("[zagros] failed to preserve raw source snapshot: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_hash_is_deterministic_across_whitespace() {
        let a = normalized_record_hash("CWE-89", "SQL Injection", "a   b\nc", "x y");
        let b = normalized_record_hash("CWE-89", "SQL Injection", "a b c", "x   y");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn current_sources_are_authoritative() {
        for source in ["cve", "cwe", "asvs", "capec", "attack"] {
            assert_eq!(
                source_definition(source).unwrap().trust_tier,
                TrustTier::Authoritative
            );
        }
    }
}
