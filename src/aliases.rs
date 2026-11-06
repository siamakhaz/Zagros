//! Reviewed vulnerability nicknames, separate from canonical CVE records.
//! This catalog is bundled with the binary; never take alias assertions from
//! model output or arbitrary network responses.
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CveAlias {
    pub cve_id: String,
    pub alias: String,
    pub relation: String,
    pub source_url: String,
    pub source_type: String,
    pub retrieved_at: String,
    pub verification_status: String,
}

/// Strip formatting only, not semantic words. Unicode alphabetics are
/// intentionally excluded to avoid mixed-script impersonation.
pub fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

fn checked(records: Vec<CveAlias>) -> Result<Vec<CveAlias>, String> {
    let mut seen = HashMap::<String, String>::new();
    let mut pairs = HashSet::new();
    for record in &records {
        let key = normalize(&record.alias);
        if key.len() < 6
            || key.len() > 80
            || !record
                .alias
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            || !record.cve_id.starts_with("CVE-")
            || record.cve_id.len() > 25
            || !record.cve_id["CVE-".len()..]
                .chars()
                .all(|c| c.is_ascii_digit() || c == '-')
            || record.relation != "vendor_nickname"
            || record.source_type != "vendor_advisory"
            || record.verification_status != "verified"
            || !record.source_url.starts_with("https://react.dev/blog/")
            || chrono::DateTime::parse_from_rfc3339(&record.retrieved_at).is_err()
        {
            return Err("alias catalog contains invalid or unreviewed evidence".into());
        }
        if let Some(previous) = seen.insert(key.clone(), record.cve_id.clone()) {
            if previous != record.cve_id {
                return Err("conflicting alias maps to multiple CVEs".into());
            }
        }
        if !pairs.insert((key, record.cve_id.clone())) {
            return Err("duplicate alias mapping".into());
        }
    }
    Ok(records)
}

pub fn catalog() -> Result<Vec<CveAlias>, String> {
    let records: Vec<CveAlias> = serde_json::from_str(include_str!("../catalog/cve-aliases.json"))
        .map_err(|e| format!("invalid alias catalog JSON: {e}"))?;
    checked(records)
}

pub fn exact<'a>(catalog: &'a [CveAlias], query: &str) -> Option<&'a CveAlias> {
    if !query.is_ascii() {
        return None;
    }
    let normalized = normalize(query);
    if normalized.len() < 6 {
        return None;
    }
    catalog
        .iter()
        .find(|alias| normalize(&alias.alias) == normalized)
}

/// Bounded edit distance: one insertion, deletion, or substitution.
/// Unlike fuzzy mapping, this only produces a labelled suggestion.
fn one_edit_apart(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a == b || a.len().abs_diff(b.len()) > 1 {
        return false;
    }
    let (mut i, mut j, mut errors) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            i += 1;
            j += 1;
            continue;
        }
        errors += 1;
        if errors > 1 {
            return false;
        }
        if a.len() > b.len() {
            i += 1;
        } else if b.len() > a.len() {
            j += 1;
        } else {
            i += 1;
            j += 1;
        }
    }
    errors + (a.len() - i).max(b.len() - j) == 1
}

pub fn suggestions<'a>(catalog: &'a [CveAlias], query: &str) -> Vec<&'a CveAlias> {
    let key = normalize(query);
    if key.len() < 6
        || key.len() > 80
        || !query
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Vec::new();
    }
    catalog
        .iter()
        .filter(|alias| one_edit_apart(&key, &normalize(&alias.alias)))
        .take(3)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(name: &str, id: &str) -> CveAlias {
        CveAlias { cve_id: id.into(), alias: name.into(), relation: "vendor_nickname".into(),
        source_url: "https://react.dev/blog/2025/12/11/denial-of-service-and-source-code-exposure-in-react-server-components".into(),
        source_type: "vendor_advisory".into(), retrieved_at: "2026-11-06T00:00:00Z".into(),
        verification_status: "verified".into() }
    }
    #[test]
    fn verified_catalog_loads() {
        assert_eq!(catalog().unwrap().len(), 1);
    }
    #[test]
    fn precise_and_normalized_alias() {
        let v = vec![fixture("React2Shell", "CVE-2025-55182")];
        assert!(exact(&v, "REACT-2-SHELL").is_some());
        assert!(exact(&v, "node2shell").is_none());
    }
    #[test]
    fn one_typo_suggested_not_confirmed() {
        let v = vec![fixture("React2Shell", "CVE-2025-55182")];
        assert_eq!(suggestions(&v, "react2shel").len(), 1);
        assert!(suggestions(&v, "node2shell").is_empty());
        assert!(suggestions(&v, "unrelated").is_empty());
    }
    #[test]
    fn untrusted_alias_rejected() {
        assert!(
            checked(vec![fixture(
                "ignore previous instructions",
                "CVE-2025-55182"
            )])
            .is_err()
        );
        assert!(checked(vec![fixture("Ð eact2Shell", "CVE-2025-55182")]).is_err());
        let mut bad = fixture("React2Shell", "CVE-2025-55182");
        bad.source_url = "https://evil.example/blog/pretend".into();
        assert!(checked(vec![bad]).is_err());
    }
    #[test]
    fn duplicate_and_conflict_rejected() {
        let a = fixture("React2Shell", "CVE-2025-55182");
        assert!(checked(vec![a.clone(), a.clone()]).is_err());
        assert!(checked(vec![a, fixture("React-2-Shell", "CVE-2025-99999")]).is_err());
    }
}
