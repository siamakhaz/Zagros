# Architecture

## Overview

Zagros is a four-binary Rust project:

| Binary | Source | Role |
|---|---|---|
| `zagros` | `src/main.rs` | CLI — ingestion, search, interactive REPL, status |
| `zagros-mcp` | `src/bin/zagros-mcp.rs` | MCP stdio server — AI agent integration |
| `zagros-mcp-http` | `src/bin/zagros-mcp-http.rs` | MCP Streamable HTTP server — remote agent integration |
| `zagros-ui` | `src/bin/zagros-ui.rs` | Web UI — browse/search CVEs and knowledge docs |

All four binaries share the library crate defined in `src/lib.rs`, `src/db.rs`, and `src/sources.rs`.

---

## Component diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                     AI Agent Clients                                  │
│           (OpenCode, Claude Desktop, VS Code, custom)                 │
└─────────────────────────┬────────────────────────────────────────────┘
                          │  stdio (JSON-RPC / MCP protocol)
                  Docker MCP Toolkit
                          │  spawns on demand
┌─────────────────────────▼────────────────────────────────────────────┐
│  zagros-mcp[-http]  (src/bin/zagros-mcp{,-http}.rs)                 │
│  search_cves  get_cve  index_status  sync_cves                        │
│  DocCache (5-min TTL)  last_sync rate-limit guard                     │
└──────────┬───────────────────────────────────────────────────────────┘
            │                              │ sync_cves only
            │                    raw.githubusercontent.com
            │                    delta.json / CVE JSON files
┌──────────▼───────────────────────────────────────────────────────────┐
│  BM25 Retrieval Engine  (src/lib.rs)                                  │
│  rank_documents(Cve)   rank_knowledge(Knowledge)                      │
│  field weights: id×8  title×3  description×1                          │
│  exact-match bonus +100   phrase bonus +8/+4                          │
└──────────┬───────────────────────────────────────────────────────────┘
           │  HTTP  (helix-db crate)
┌──────────▼───────────────────────────────────────────────────────────┐
│  HelixDB  (docker/compose.yml, port 47474)                            │
│  Cve nodes           Knowledge nodes                                  │
│  499+ CVE records    2,624 CWE/ASVS/CAPEC/ATT&CK records             │
│  No graph edges yet  No vector embeddings yet                         │
│  Volume: zagros_helix-data                                           │
└──────────────────────────────────────────────────────────────────────┘

Web UI Binary (src/bin/zagros-ui.rs)
  /api/status  /api/cves  /api/knowledge  /api/search
  Default port 8788 via UI_PORT

CLI Binary  (src/main.rs)
  ingest  backfill  source  search  know  interactive  status
  Uses same db.rs + lib.rs path; no MCP layer; no in-process cache
```

---

## Source layout

```
zagros/
├── Cargo.toml                  # dependencies, edition 2024
├── Cargo.lock                  # pinned dependency graph
├── Dockerfile                  # two-stage build → debian:bookworm-slim
├── assets/
│   └── zagros.png
├── data/
│   └── cves.json               # legacy flat-file cache (gitignored)
├── docker/
│   ├── compose.yml             # HelixDB sidecar, port 127.0.0.1:47474:8080
│   ├── server.yaml             # Docker MCP Toolkit registration
│   ├── zagros-server.yaml     # alternate registration with HELIX_URL env
│   ├── tools.json              # machine-readable tool catalog
│   └── register.ps1            # idempotent MCP catalog + profile registration
├── docs/                       # documentation (you are here)
├── scripts/
│   └── setup.ps1               # end-to-end machine setup
└── src/
    ├── lib.rs                  # public API: ingestion, BM25, type definitions
    ├── db.rs                   # HelixDB CRUD for Cve and Knowledge nodes
    ├── sources.rs              # KnowledgeDoc + four source parsers
    ├── main.rs                 # CLI entry point
    └── bin/
        ├── zagros-mcp.rs      # MCP server entry point (stdio)
        ├── zagros-mcp-http.rs # MCP server entry point (Streamable HTTP)
        └── zagros-ui.rs       # web UI entry point
```

---

## HelixDB node schemas

### `Cve` node

| Field | Type | Notes |
|---|---|---|
| `cve_id` | String | Primary key pattern — e.g. `CVE-2026-17061` |
| `title` | String | First English description sentence from the CVE record |
| `description` | String | Full description text |
| `published_at` | String | RFC 3339 timestamp or empty string |
| `updated_at` | String | RFC 3339 timestamp or empty string |

### `Knowledge` node

| Field | Type | Notes |
|---|---|---|
| `doc_id` | String | Primary key — e.g. `CWE-89`, `ASVS-v5.0.0-1.2.5`, `CAPEC-66`, `ATT&CK-T1190` |
| `name` | String | Short human-readable title |
| `description` | String | Full text used for BM25 scoring |
| `source` | String | `cwe` \| `asvs` \| `capec` \| `attack` |
| `url` | String | Canonical source URL |
| `tags` | String | Tactics, platforms, chapter, level (source-specific) |

HelixDB has no built-in authentication. The port is restricted to loopback
(`127.0.0.1:47474`) in `docker/compose.yml`.

---

## BM25 retrieval engine

The engine lives entirely in `src/lib.rs` and runs in-process — no round-trip
to HelixDB for search, only for the initial data load.

### Tokenization

- Split on any character that is not alphanumeric or a hyphen.
- Discard tokens with length ≤ 1.
- Lowercase all tokens.
- Hyphens are preserved so `CVE-2026-17061` tokenizes as a single token, not three.

### Scoring formula

For each document, BM25 scores are computed per field with individual field weights:

```
score = Σ_term [ idf(term) × tf_norm(term, field) × field_weight ]
```

Field weights:
- `cve_id` / `doc_id`: **8**
- `title` / `name`: **3**
- `description`: **1**

Length normalization (within each field):

```
tf_norm = (tf × (k1 + 1)) / (tf + k1 × (0.25 + 0.75 × doc_len / avg_len))
```

k1 = 2.2

### Bonus scoring

| Condition | Bonus |
|---|---|
| Query exactly equals document ID | +100.0 |
| Document ID contains full query | +20.0 |
| Multi-word query is substring of title/name | +8.0 |
| Multi-word query is substring of description | +4.0 |

### Filtering and tie-breaking

- Documents with score ≤ `MIN_SCORE` (1.0) are excluded entirely.
- Results are sorted by score descending, then by `updated_at` descending.
- Truncated to `top_k`.

### In-process document cache (MCP server only)

The MCP server holds a `DocCache` with a 5-minute TTL. On cache hit, search
runs without any HelixDB network call. `sync_cves` invalidates the cache on
success. The CLI does not use a cache.

---

## Upsert strategy

HelixDB v3 has no native upsert or transaction. The implemented strategy is
delete-then-insert with one retry:

1. `DELETE` the node matching the primary key.
2. `INSERT` the new node.
3. On insert failure: retry once.
4. On second failure: return error.

**Residual risk:** if the process is killed between step 1 and step 2, the
record is absent from the index until the next successful upsert of the same ID.

---

## MCP server internals

`zagros-mcp` is a stdio-transport MCP server built with `rmcp` 3.1.2.
Docker Desktop MCP Toolkit spawns it as a subprocess when an agent client
connects.

### Tool schemas

Input parameter types (`SearchParams`, `GetCveParams`, `SyncParams`) derive
`JsonSchema` via `schemars`. Tool schemas are generated at compile time and
published to MCP clients during the `initialize` handshake.

### Rate limiting

`sync_cves` enforces a minimum 5-minute interval between calls using an
`Arc<Mutex<Option<Instant>>>` on the server struct. Calls within the window
return `McpError::invalid_params`.

### CVE ID validation

`get_cve` validates the `cve_id` parameter before any HelixDB lookup:
- Must match `CVE-{4-digit year}-{4+ digit sequence}`.
- Rejects path traversal strings (e.g. `../../secret`).
- Rejects malformed IDs (e.g. `CVE-26-1`).

---

## Docker image

The `Dockerfile` uses a two-stage build:

**Stage 1 — builder** (`rust:1.95-bookworm`, pinned SHA256)
- Compiles the `zagros-mcp` and `zagros-mcp-http` binaries with `--release --locked`.

**Stage 2 — runtime** (`debian:bookworm-slim`, pinned SHA256)
- Installs only `ca-certificates`.
- Creates system user `zagros` (uid 10001, no shell, no home).
- Copies binary as root-owned 755.
- Sets `ZAGROS_DATA_DIR=/data` and declares `/data` as a Docker volume.
- Runs as `USER zagros`.

The CLI binary (`zagros`) is not included in the Docker image.

---

## Roadmap

See [VISION.md](VISION.md) for the full phased roadmap. Key upcoming phases:

| Phase | Focus |
|---|---|
| 2 (current) | SHA-256 content hashes, `retrieved_at` timestamps, `sync_source` MCP tool, OWASP Cheat Sheets |
| 3 | HelixDB graph edges (CWE → ATT&CK, CWE → ASVS), dense vector embeddings, hybrid BM25 + vector + graph with RRF |
| 4 | LLM reasoning layer with citation enforcement, `analyze_code` and `check_config` MCP tools |
| 5 | AST-based code analysis, Dockerfile/IaC parser, SBOM |
| 6 | Evaluation harness, precision/recall per CWE, confidence calibration |
