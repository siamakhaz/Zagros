# CLI Reference

The `zagros` binary is built from `src/main.rs`. It provides seven subcommands
covering CVE ingestion, knowledge ingestion, search, and status reporting.

```
zagros.exe <COMMAND>

Commands:
  ingest       Fetch latest changed CVEs and store them in HelixDB
  backfill     Walk deltaLog.json history and load up to N CVEs into HelixDB
  source       Ingest a security knowledge source into HelixDB
  search       Search CVE records in HelixDB
  know         Search the security knowledge base (CWE, ASVS, CAPEC, ATT&CK)
  interactive  Open an interactive CVE search prompt
  status       Show HelixDB status
  help         Print help for a subcommand
```

---

## `ingest`

Fetch the latest changed CVEs from the official CVE Project delta feed and
upsert them into HelixDB.

```powershell
zagros.exe ingest [OPTIONS]
```

**Options:**

| Flag | Default | Range | Description |
|---|---|---|---|
| `--limit <N>` | `50` | 1–1000 | Number of latest changed CVE records to download |

**Example:**

```powershell
.\target\debug\zagros.exe ingest --limit 100
# ingested 97 changed CVEs; 596 total in HelixDB
```

**Behavior:**
- Downloads `delta.json` from the CVE Project GitHub repository.
- Sorts delta entries by `dateUpdated` descending (most recent first).
- Takes the first `--limit` entries.
- For each entry, fetches the individual CVE JSON file.
- Validates the URL origin and enforces a 10 MB per-file size limit.
- Skips CVEs that are not in `PUBLISHED` state.
- Upserts each record into HelixDB (delete-then-insert; safe to re-run).

---

## `backfill`

Walk the full `deltaLog.json` history and load up to N unique CVEs into HelixDB.
Use this on first setup to build a useful starting corpus.

```powershell
zagros.exe backfill [OPTIONS]
```

**Options:**

| Flag | Default | Range | Description |
|---|---|---|---|
| `--limit <N>` | `500` | 1–10000 | Maximum number of unique CVEs to load |
| `--verbose` | false | — | Print per-URL skip messages |

**Example:**

```powershell
.\target\debug\zagros.exe backfill --limit 500
# fetched 499 CVEs; 499 total in HelixDB

.\target\debug\zagros.exe backfill --limit 1000 --verbose
# skipping already-seen https://raw.githubusercontent.com/...
```

**Behavior:**
- Downloads `deltaLog.json` (~300 KB, ~1000 hourly snapshots, newest first).
- Walks entries from newest to oldest, collecting unique `githubLink` values.
- Stops when `--limit` unique links have been collected.
- Fetches and upserts each CVE record (same validation as `ingest`).

---

## `source`

Ingest one or all authoritative security knowledge sources into HelixDB.

```powershell
zagros.exe source <NAME>
```

**Argument:**

| Value | Source | Records | Format |
|---|---|---|---|
| `cwe` | MITRE CWE (latest XML ZIP) | ~969 | XML in ZIP |
| `asvs` | OWASP ASVS 5.0.0 (CSV) | 345 | CSV |
| `capec` | MITRE CAPEC (latest XML) | ~613 | XML |
| `attack` | MITRE ATT&CK Enterprise (STIX 2.1) | ~697 | JSON |
| `all` | All four sources in sequence | ~2624 | — |

**Examples:**

```powershell
.\target\debug\zagros.exe source all
# ingested 969 CWE records; 969 total
# ingested 345 ASVS records; 345 total
# ingested 613 CAPEC records; 613 total
# ingested 697 ATT&CK records; 697 total

.\target\debug\zagros.exe source cwe
.\target\debug\zagros.exe source asvs
```

**Behavior:**
- Downloads source data from the canonical upstream URL.
- Parses and upserts all valid records (idempotent; safe to re-run).
- CAPEC and ATT&CK records with `Deprecated`, `Obsolete`, or `revoked` status are skipped.
- Prints count of ingested records and total in HelixDB on completion.

See [DATA-SOURCES.md](DATA-SOURCES.md) for source URLs, formats, and record ID schemes.

---

## `search`

Search CVE records stored in HelixDB using ranked lexical retrieval.

```powershell
zagros.exe search <QUERY> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|---|---|
| `<QUERY>` | Search phrase, product name, vulnerability class, or CVE ID |

**Options:**

| Flag | Default | Range | Description |
|---|---|---|---|
| `--top-k <N>` | `10` | 1–500 | Maximum number of results to return |
| `--json` | false | — | Emit results as JSON instead of formatted text |

**Examples:**

```powershell
.\target\debug\zagros.exe search "remote code execution" --top-k 5

.\target\debug\zagros.exe search CVE-2024-3094

.\target\debug\zagros.exe search "buffer overflow in nginx" --top-k 20

.\target\debug\zagros.exe search "authentication bypass" --json
```

**Text output format:**

```
1. CVE-2024-3094 | score 128.342 | updated <timestamp>
   XZ Utils supply-chain compromise
2. CVE-2025-12345 | score 42.100 | updated 2025-11-15
   ...
```

**JSON output format** (`--json`):

```json
[
  {
    "cve_id": "CVE-2024-3094",
    "title": "XZ Utils supply-chain compromise",
    "description": "...",
    "published_at": "<RFC3339 timestamp>",
    "updated_at": "<RFC3339 timestamp>",
    "source_url": "https://www.cve.org/CVERecord?id=CVE-2024-3094",
    "score": 128.342
  }
]
```

**Ranking behavior:**
- An exact CVE ID match scores +100.0 bonus and always ranks first.
- Query terms are matched against CVE ID (weight 8×), title (3×), and description (1×).
- Multi-word phrases that appear verbatim in the title receive a +8.0 bonus.
- Documents scoring ≤ 1.0 are excluded from results.

---

## `know`

Search the security knowledge base (CWE, ASVS, CAPEC, ATT&CK) using ranked
lexical retrieval.

```powershell
zagros.exe know <QUERY> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|---|---|
| `<QUERY>` | Search phrase, weakness name, technique name, or rule ID |

**Options:**

| Flag | Default | Range | Description |
|---|---|---|---|
| `--top-k <N>` | `10` | 1–500 | Maximum number of results to return |

**Examples:**

```powershell
.\target\debug\zagros.exe know "SQL injection"
.\target\debug\zagros.exe know "hardcoded credentials"
.\target\debug\zagros.exe know "lateral movement"
.\target\debug\zagros.exe know CWE-89
.\target\debug\zagros.exe know "ASVS-v5.0.0-1.2.5"
```

**Output format:**

```
1. CWE-89 [cwe] | score 94.200
   Improper Neutralization of Special Elements used in an SQL Command ('SQL Injection')
   SQL injection occurs when user-supplied input is incorporated into a SQL query...
   https://cwe.mitre.org/data/definitions/89.html

2. CAPEC-66 [capec] | score 72.100
   SQL Injection
   ...
```

**Ranking behavior:** same BM25 algorithm as `search`, applied to the
`Knowledge` corpus. Fields are `doc_id` (8×), `name` (3×), `description` (1×).

---

## `interactive`

Open an interactive search REPL that keeps HelixDB data loaded in memory
across queries.

```powershell
zagros.exe interactive [OPTIONS]
```

**Options:**

| Flag | Default | Range | Description |
|---|---|---|---|
| `--top-k <N>` | `10` | 1–500 | Initial result count |

**Session commands:**

| Input | Effect |
|---|---|
| Any search phrase | Run BM25 search; print top-k results |
| `:limit N` | Change result count for subsequent queries (1–500) |
| `:help` | Show usage hint |
| `:quit`, `:q`, `quit`, `exit` | Exit the REPL |

**Example session:**

```
> remote code execution
1. CVE-2024-3094 | score 128.3 | updated <timestamp>
   ...

> :limit 5

> buffer overflow in libpng
1. CVE-2025-98765 | score 44.1 | updated 2025-10-30
   ...

> :quit
```

**Note:** `interactive` searches CVE records only. Use `know` for the security
knowledge base.

---

## `status`

Show HelixDB connection details and record counts.

```powershell
zagros.exe status
```

No options.

**Example output:**

```
HelixDB URL  : http://localhost:47474
CVE nodes    : 499

Knowledge nodes by source:
  asvs        : 345
  attack      : 697
  capec       : 613
  cwe         : 969
  total       : 2,624
```

---

## Global options

| Flag | Description |
|---|---|
| `-h`, `--help` | Print help for the current command |
| `-V`, `--version` | Print version |

---

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Error (HelixDB unreachable, network error, parse failure, invalid argument) |

Error messages are printed to stderr. All normal output goes to stdout.

---

## Environment variables

| Variable | Default | Effect |
|---|---|---|
| `HELIX_URL` | `http://localhost:47474` | HelixDB instance used by all commands |
| `ZAGROS_DATA_DIR` | `data/` relative to CWD | Directory for legacy flat-file cache (`cves.json`) |

See [CONFIGURATION.md](CONFIGURATION.md) for full configuration reference.
