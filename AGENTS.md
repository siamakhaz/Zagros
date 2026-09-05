# AGENTS.md — cve-rag

## What this repo is

Rust CLI + MCP server for ingesting and BM25-searching CVE records and four security knowledge corpora (CWE, ASVS, CAPEC, ATT&CK), backed by a local HelixDB graph-vector store. Three binaries share one library crate.

---

## Binaries

| Binary | Source | Purpose |
|---|---|---|
| `cve-rag` | `src/main.rs` | CLI (clap derive) |
| `cve-rag-mcp` | `src/bin/cve-rag-mcp.rs` | MCP server (stdio transport) |
| `cve-ui` | `src/bin/cve-ui.rs` | Optional web UI (axum, port 8788) |

All three link to `src/lib.rs` (BM25 engine, HTTP ingestion, data types) and `src/db.rs` / `src/sources.rs`.

---

## Developer commands

```powershell
# Check / build
cargo check
cargo build                          # debug: target\debug\*.exe
cargo build --release                # release: target\release\*.exe
cargo build --bin cve-rag            # single binary

# Test (4 unit tests, all offline, no HelixDB required)
cargo test
cargo test exact_cve_id              # single test by name prefix
cargo test -- --nocapture            # show println! output

# Lint / format
cargo fmt
cargo fmt -- --check
cargo clippy
cargo clippy -- -D warnings          # CI-equivalent: fail on any warning
```

There is no CI config in the repo yet. The implied check order is: `fmt --check` → `clippy -D warnings` → `test`.

---

## HelixDB is a required sidecar — start it first

Every command that reads or writes data requires HelixDB running at `http://localhost:47474` (default). Tests do **not** require it.

```powershell
docker compose -f docker/compose.yml up -d helix
docker compose -f docker/compose.yml ps   # verify healthy
```

Override the URL: `$env:HELIX_URL = "http://localhost:47474"`

HelixDB uses two node labels: `Cve` (key: `cve_id`) and `Knowledge` (key: `doc_id`). There is no native upsert — all writes are delete-then-insert (see F-03 in `docs/SECURITY-REVIEW.md`).

---

## First-run seeding sequence

```powershell
.\target\debug\cve-rag.exe backfill --limit 500   # ~499 CVEs from deltaLog history
.\target\debug\cve-rag.exe source all              # CWE + ASVS + CAPEC + ATT&CK
.\target\debug\cve-rag.exe status                  # verify counts
```

`backfill` and `source` are idempotent (upsert). Re-running is safe.

Wipe and re-seed:
```powershell
docker compose -f docker/compose.yml down -v
docker compose -f docker/compose.yml up -d helix
# then repeat seeding sequence above
```

---

## MCP server setup

```powershell
docker build -t cve-rag-mcp:0.1.0 .   # two-stage build, uses Cargo.lock --locked
.\docker\register.ps1                  # registers with Docker Desktop MCP Toolkit, profile "profile"
.\scripts\setup.ps1 -Seed -SeedLimit 500  # full automated setup (build + register + seed)
```

The MCP server runs as stdio transport. It has an in-process doc cache (5-minute TTL) and a rate-limit guard on `sync_cves` (5-minute cooldown). Both guards reset on process restart.

MCP tools: `search_cves`, `get_cve`, `index_status`, `sync_cves`.

---

## Code structure — what lives where

| File | Owns |
|---|---|
| `src/lib.rs` | `CveDocument`, `KnowledgeDoc`, BM25 (`rank_documents`, `rank_knowledge`), HTTP fetch, flat-file I/O |
| `src/db.rs` | HelixDB CRUD for `Cve` and `Knowledge` nodes |
| `src/sources.rs` | Parsers for CWE (XML ZIP), ASVS (CSV), CAPEC (XML), ATT&CK (STIX JSON) |
| `src/main.rs` | CLI command dispatch |
| `src/bin/cve-rag-mcp.rs` | MCP tool handlers, `DocCache`, rate-limit guard, `valid_cve_id` |
| `src/bin/cve-ui.rs` | Axum web UI |

New domain logic goes in `src/lib.rs` or `src/db.rs`, not in the CLI or MCP binaries.

---

## Key quirks

- **`helix-db` is pinned to `=3.0.0`** (exact version). Do not bump without checking the DSL API.
- **BM25 runs in-process** — `load_all()` fetches every node from HelixDB on every CLI search call. Fast now; will need caching for corpora > ~5K nodes.
- **Tokenizer preserves hyphens** — required for CVE IDs like `CVE-2026-17061` to match as one token. Do not "fix" the tokenizer without re-running the BM25 tests.
- **`reqwest` uses rustls** — no OpenSSL dependency. Do not add features that pull in native-tls.
- **`MIN_SCORE = 1.0`** is set in `src/lib.rs`. Documents scoring below this are excluded. Adjust empirically if the corpus grows past ~5K.
- **Dockerfile uses `--locked`** and pinned SHA256 base image digests. Always commit `Cargo.lock`.

---

## Open security findings (from `docs/SECURITY-REVIEW.md`)

| ID | Severity | What it is |
|---|---|---|
| F-01 | High | HelixDB port `47474` binds to `0.0.0.0`; fix: `127.0.0.1:47474:8080` in `docker/compose.yml:27` |
| F-02 | High | CVE ingestion has no URL-origin validation or payload size cap on individual CVE records — **already partially fixed** in `src/lib.rs` (`bytes.len() < 10 MB` check and origin prefix check present in current code) |
| F-03 | Medium | `upsert_document` is delete-then-insert, not atomic; a record can vanish if the process dies mid-upsert |
| F-04 | Medium | `sync_cves` rate-limit guard **already implemented** in current `src/bin/cve-rag-mcp.rs` |
| F-05 | Medium | `load_all()` full table scan; MCP server cache **already implemented** |
| F-06 | Low | BM25 `MIN_SCORE` threshold **already set** at 1.0 in `src/lib.rs` |

F-01 (HelixDB exposed on all interfaces) is the only High-severity finding that remains unaddressed in the current code.

---

## Extension patterns

**New CLI subcommand:** add variant to `Commands` enum in `src/main.rs`, match it in `main()`, implement logic in `src/lib.rs`.

**New knowledge source:** implement `ingest_<source>() -> Result<Vec<KnowledgeDoc>>` in `src/sources.rs`, add wrapper in `src/lib.rs`, add match arm in `src/main.rs` `run_source()`.

**New MCP tool:** add params struct (`Deserialize + JsonSchema`), output struct (`Serialize`), `#[tool(...)]` method on `CveMcpServer`, register in `tool_router!`.

---

## graphify knowledge graph

A knowledge graph of this repository is stored in `graphify-out/` (201 nodes, 360 edges, 18 communities). Use it to answer structural questions without re-reading the full codebase.

### Prerequisites

```powershell
py -m pip install graphifyy   # Python 3.12+ required; already installed if graphify-out/ exists
```

### Query the graph

```powershell
# Check a specific concept and everything connected to it
/graphify explain "BM25 retrieval engine"

# Trace how two concepts are connected
/graphify path "sync_cves" "rate limit"

# Ask a broad question (BFS traversal)
/graphify query "how does the MCP server handle caching"

# Narrow path-tracing (DFS)
/graphify query "how does ingest reach HelixDB" --dfs
```

### Rebuild after code changes

```powershell
# Incremental update (re-extracts only changed files, code-only = no LLM needed)
/graphify . --update

# Full rebuild from scratch
/graphify .
```

### What the graph contains

- **18 communities** including: Web UI Layer, CVE Data Models, Search and Ingestion Architecture, MCP Server Implementation, CLI Commands and Configuration, Security Knowledge Sources, Multi-Binary Architecture, Runtime Configuration.
- **God nodes** (highest betweenness): `Ok()` (38 edges, bridges 5 communities), `client()` (20 edges), `load_all_knowledge()` (12 edges).
- **Hyperedges** capturing group relationships: Tier-1 ingestion pipeline, CVE MCP tool suite, Phase 3 graph-rich search vision.
- **Audit trail**: every edge is tagged EXTRACTED, INFERRED, or AMBIGUOUS with a confidence score.

Graph outputs:
- `graphify-out/graph.html` — interactive visualization, open in browser
- `graphify-out/graph.json` — machine-readable graph for programmatic queries
- `graphify-out/GRAPH_REPORT.md` — full audit report with god nodes, surprising connections, and suggested questions
