# Data Sources

cve-rag ingests five data sources into HelixDB. CVE records use the `Cve` node label.
All security knowledge sources use the `Knowledge` node label with a `source` field
distinguishing them.

---

## Summary

| Source | Command | Records | Node label | `source` field |
|---|---|---|---|---|
| CVE Project delta feed | `ingest` / `backfill` | 499+ | `Cve` | — |
| MITRE CWE | `source cwe` | ~969 | `Knowledge` | `cwe` |
| OWASP ASVS 5.0.0 | `source asvs` | 345 | `Knowledge` | `asvs` |
| MITRE CAPEC | `source capec` | ~613 | `Knowledge` | `capec` |
| MITRE ATT&CK Enterprise | `source attack` | ~697 | `Knowledge` | `attack` |

---

## CVE Project delta feed

**Command:** `cve-rag ingest` / `cve-rag backfill`

**Source:**
- Delta URL: `https://raw.githubusercontent.com/CVEProject/cvelistV5/main/cves/delta.json`
- DeltaLog URL: `https://raw.githubusercontent.com/CVEProject/cvelistV5/main/cves/deltaLog.json`
- Individual CVE files: fetched from `githubLink` values in the delta (always under `https://raw.githubusercontent.com/CVEProject/cvelistV5/`)

**Format:** JSON (CVE 5.0 schema)

**Record ID scheme:** `CVE-{4-digit year}-{4+ digit sequence}` — e.g. `CVE-2026-17061`

**Ingestion flow:**

```
GET delta.json
  → deserialize CveDelta { new: [...], updated: [...] }
  → sort by dateUpdated descending
  → take first --limit entries

For each entry:
  validate URL origin (must start with https://raw.githubusercontent.com/CVEProject/cvelistV5/)
  GET {githubLink}
    validate payload < 10 MB
    deserialize CveRecord
    filter: state == "PUBLISHED"
    extract first English description → title
    build CveDocument { cve_id, title, description, published_at, updated_at }
    db::upsert_document(helix, doc)
```

**Backfill difference:** downloads `deltaLog.json` (~300 KB, ~1000 hourly snapshots),
walks entries newest-to-oldest, collects up to `--limit` unique `githubLink` values
before fetching. Allows loading a much larger historical corpus.

**Security controls:**
- URL prefix validation: rejects any `githubLink` not starting with the expected origin.
- Payload size cap: rejects responses larger than 10 MB.
- Only `PUBLISHED` CVEs are stored; draft/rejected records are skipped.

**Node schema:**

| Field | Example |
|---|---|
| `cve_id` | `CVE-2026-17061` |
| `title` | `Remote code execution in ExampleLib via crafted input` |
| `description` | Full CVE description text |
| `published_at` | `2026-07-01T00:00:00Z` |
| `updated_at` | `2026-08-16T14:30:00Z` |

**Source URL** (constructed at query time): `https://www.cve.org/CVERecord?id={cve_id}`

---

## MITRE CWE

**Command:** `cve-rag source cwe`

**Source URL:** `https://cwe.mitre.org/data/xml/cwec_latest.xml.zip`

**Format:** ZIP archive containing one XML file (MITRE CWE catalog schema v4.20+)

**Record ID scheme:** `CWE-{numeric ID}` — e.g. `CWE-89`

**Ingestion flow:**

```
GET cwec_latest.xml.zip (size limit: 50 MB, timeout: 120 s)
  → ZipArchive::new → read first entry → XML bytes

SAX parse (quick-xml):
  <Weakness ID="N" Name="...">
    <Description>text</Description>   ← first <Description> child only
  </Weakness>

  → skip if description is empty
  → XML-unescape entities in description text
  → build KnowledgeDoc {
      id: "CWE-{N}",
      name: weakness name,
      description: description text,
      source: "cwe",
      url: "https://cwe.mitre.org/data/definitions/{N}.html",
      tags: ""
    }

db::upsert_knowledge_batch(helix, docs)
```

**Skipped records:** weaknesses with no description text.

**Tags field:** empty for CWE (no tactic or platform metadata in this corpus).

**Example node:**

| Field | Value |
|---|---|
| `doc_id` | `CWE-89` |
| `name` | `Improper Neutralization of Special Elements used in an SQL Command ('SQL Injection')` |
| `source` | `cwe` |
| `url` | `https://cwe.mitre.org/data/definitions/89.html` |

---

## OWASP ASVS 5.0.0

**Command:** `cve-rag source asvs`

**Source URL:** `https://raw.githubusercontent.com/OWASP/ASVS/v5.0.0/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.csv`

**Format:** CSV — columns: `chapter_id`, `chapter_name`, `section_id`, `section_name`, `req_id`, `req_description`, `L`

**Record ID scheme:** `ASVS-v5.0.0-{req_id}` — e.g. `ASVS-v5.0.0-1.2.5`

The source URL is pinned to the `v5.0.0` Git tag. When OWASP releases a new
major version, the URL and ID prefix must be updated manually in `src/sources.rs`.

**Ingestion flow:**

```
GET OWASP_ASVS_5.0.0_en.csv (size limit: 10 MB)
  → csv::Reader::deserialize into Vec<AsvsRow>

For each row:
  skip if req_id is empty
  build KnowledgeDoc {
      id: "ASVS-v5.0.0-{req_id}",
      name: req_id (e.g. "1.2.5"),
      description: req_description,
      source: "asvs",
      url: "https://github.com/OWASP/ASVS/tree/v5.0.0",
      tags: "{chapter_name} | {section_name} | Level {L}"
    }

db::upsert_knowledge_batch(helix, docs)
```

**Tags field:** `"{chapter_name} | {section_name} | Level {L}"` — e.g.
`"Encoding and Sanitization | Input Sanitization | Level 1"`.

**Example node:**

| Field | Value |
|---|---|
| `doc_id` | `ASVS-v5.0.0-13.3.1` |
| `name` | `13.3.1` |
| `description` | `Verify that application secrets, API keys... are stored in a secrets vault` |
| `source` | `asvs` |
| `tags` | `Secrets Management | Secrets Management | Level 1` |

---

## MITRE CAPEC

**Command:** `cve-rag source capec`

**Source URL:** `https://capec.mitre.org/data/xml/capec_latest.xml`

**Format:** XML (CAPEC schema 3.9+)

**Record ID scheme:** `CAPEC-{numeric ID}` — e.g. `CAPEC-66`

**Ingestion flow:**

```
GET capec_latest.xml (size limit: 50 MB, timeout: 60 s)

SAX parse (quick-xml):
  <Attack_Pattern ID="N" Name="..." Status="...">
    <Description>text</Description>   ← top-level <Description> only
  </Attack_Pattern>

  → skip if Status == "Deprecated" or "Obsolete"
  → skip nested <Description> elements (inside sub-elements)
  → build KnowledgeDoc {
      id: "CAPEC-{N}",
      name: attack pattern name,
      description: description text,
      source: "capec",
      url: "https://capec.mitre.org/data/definitions/{N}.html",
      tags: ""
    }

db::upsert_knowledge_batch(helix, docs)
```

**Skipped records:** patterns with status `Deprecated` or `Obsolete`.

**Tags field:** empty for CAPEC.

**Example node:**

| Field | Value |
|---|---|
| `doc_id` | `CAPEC-66` |
| `name` | `SQL Injection` |
| `source` | `capec` |
| `url` | `https://capec.mitre.org/data/definitions/66.html` |

---

## MITRE ATT&CK Enterprise

**Command:** `cve-rag source attack`

**Source URL:** `https://raw.githubusercontent.com/mitre-attack/attack-stix-data/master/enterprise-attack/enterprise-attack.json`

**Format:** STIX 2.1 JSON bundle (`StixBundle { objects: Vec<StixObject> }`)

**Record ID scheme:** `ATT&CK-T{nnnn}` or `ATT&CK-T{nnnn}.{nnn}` (subtechniques) — e.g. `ATT&CK-T1190`, `ATT&CK-T1059.001`

**Ingestion flow:**

```
GET enterprise-attack.json (size limit: 50 MB, timeout: 120 s)
  → serde_json::from_slice::<StixBundle>

For each StixObject:
  skip if type != "attack-pattern"
  skip if revoked == true
  skip if x_mitre_deprecated == true
  skip if description is empty

  find external_reference where source_name == "mitre-attack"
  extract external_id (T#### or T####.###)
  skip if no external_id found

  join kill_chain_phases[*].phase_name → tactics string
  join x_mitre_platforms → platforms string
  tags = "{tactics} | {platforms}"

  build KnowledgeDoc {
      id: "ATT&CK-{external_id}",
      name: object name,
      description: description,
      source: "attack",
      url: external_reference url,
      tags: tags
    }

db::upsert_knowledge_batch(helix, docs)
```

**Skipped records:** revoked techniques, deprecated techniques, techniques without
a `mitre-attack` external reference, and techniques with empty descriptions.

**Tags field:** `"{tactic1} {tactic2} | {platform1} {platform2}"` — e.g.
`"initial-access | Windows Linux macOS"`.

**Example node:**

| Field | Value |
|---|---|
| `doc_id` | `ATT&CK-T1190` |
| `name` | `Exploit Public-Facing Application` |
| `source` | `attack` |
| `tags` | `initial-access | Windows Linux macOS Android iOS` |

---

## Data flow summary

```
Remote source                  Parser (src/sources.rs)          HelixDB
─────────────────────────────  ───────────────────────────────  ────────────────────
CVE Project delta.json  ─────► fetch_latest_cves()          ►  Cve nodes
CVE Project deltaLog.json ───► backfill_from_history()       ►  Cve nodes
MITRE CWE XML ZIP       ─────► ingest_cwe() / parse_cwe_xml() ► Knowledge (source=cwe)
OWASP ASVS CSV          ─────► ingest_asvs() / parse_asvs_csv()► Knowledge (source=asvs)
MITRE CAPEC XML         ─────► ingest_capec() / parse_capec_xml()► Knowledge (source=capec)
MITRE ATT&CK STIX JSON  ─────► ingest_attack() / parse_attack_stix()► Knowledge (source=attack)
```

All ingestion functions are idempotent: re-running any source command upserts
(delete-then-insert) existing records rather than creating duplicates.

---

## Planned sources (Phase 2–3)

| Source | Format | Status |
|---|---|---|
| OWASP Cheat Sheets | Markdown | Planned Phase 2 |
| NIST NCP | SCAP/XCCDF | Planned Phase 3 |
| DISA STIGs | XCCDF/XML | Planned Phase 3 |
| Kubernetes Pod Security Standards | Structured docs | Planned Phase 3 |
| SEI CERT Coding Standards | HTML | Planned Phase 3+ |
| OSV / GitHub Advisory Database | JSON | Planned Phase 5 |
