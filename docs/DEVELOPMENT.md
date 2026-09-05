# Development

## Prerequisites

| Tool | Version | Purpose |
|---|---|---|
| Rust | 1.85+ | Compile the project |
| Docker Desktop | Current | Run HelixDB sidecar |
| PowerShell | 7+ | Setup and registration scripts |

Install Rust via [rustup](https://rustup.rs/).

---

## Build

### Debug build (fast, for development)

```powershell
cargo build
```

Produces:
- `target\debug\cve-rag.exe` — CLI
- `target\debug\cve-rag-mcp.exe` — MCP server

### Release build (optimized, for Docker image)

```powershell
cargo build --release
```

Produces:
- `target\release\cve-rag.exe`
- `target\release\cve-rag-mcp.exe`

### Build only one binary

```powershell
cargo build --bin cve-rag
cargo build --bin cve-rag-mcp
```

### Check without producing binaries

```powershell
cargo check
```

---

## Tests

```powershell
cargo test
```

Current tests (4 total):

| Test | File | What it covers |
|---|---|---|
| `exact_cve_id_is_ranked_first` | `src/lib.rs` | Exact CVE ID match outranks all partial matches |
| `phrase_and_title_matches_receive_higher_score` | `src/lib.rs` | Title phrase match outscores description-only match |
| `unrelated_documents_are_not_returned` | `src/lib.rs` | No results when no term overlap |
| `validates_cve_ids` | `src/bin/cve-rag-mcp.rs` | `valid_cve_id` accepts well-formed IDs and rejects malformed and path-traversal inputs |

Run a single test:

```powershell
cargo test exact_cve_id
```

Run tests with output:

```powershell
cargo test -- --nocapture
```

---

## Code structure

| File | Responsibility |
|---|---|
| `src/lib.rs` | Public API: `CveDocument`, `KnowledgeDoc`, HTTP ingestion, BM25 engine, flat-file I/O |
| `src/db.rs` | HelixDB CRUD: upsert, load, count for `Cve` and `Knowledge` nodes |
| `src/sources.rs` | Knowledge source parsers: CWE XML ZIP, ASVS CSV, CAPEC XML, ATT&CK STIX JSON |
| `src/main.rs` | CLI entry point: clap command dispatch |
| `src/bin/cve-rag-mcp.rs` | MCP server: tool handlers, DocCache, rate-limit guard, CVE ID validation |

### Adding a new CLI subcommand

1. Add a variant to the `Commands` enum in `src/main.rs`.
2. Add a struct with `#[derive(Args)]` if the command has options.
3. Match the variant in `main()` and call the appropriate library function.
4. Add any new domain logic to `src/lib.rs` or `src/db.rs`.

### Adding a new knowledge source

1. Add a public `ingest_<source>()` function to `src/sources.rs` that:
   - Downloads the source.
   - Parses it into `Vec<KnowledgeDoc>`.
   - Calls `db::upsert_knowledge_batch()`.
   - Returns `Result<(usize, usize)>` (ingested, total).
2. Add a wrapper in `src/lib.rs` that calls your parser.
3. Add the source name to the `source` CLI command match arms in `src/main.rs`.

### Adding a new MCP tool

1. Define an input params struct deriving `Deserialize` and `JsonSchema`.
2. Define an output struct deriving `Serialize`.
3. Add a method on `CveMcpServer` with `#[tool(description = "...")]`.
4. Register the method with `tool_router!` in the `ServerHandler` impl.

---

## Dependencies

Key crates and their roles:

| Crate | Version | Role |
|---|---|---|
| `helix-db` | `=3.0.0` (pinned) | HelixDB client and DSL |
| `rmcp` | `3.1.2` | MCP server protocol (stdio transport, schemars for tool schemas) |
| `reqwest` | `0.13` | Async HTTP (rustls TLS — no OpenSSL dependency) |
| `clap` | `4` | CLI argument parsing (derive macros) |
| `serde` + `serde_json` | `1` | JSON serialization/deserialization |
| `sonic-rs` | `0.5` | Fast JSON value type for raw HelixDB responses |
| `quick-xml` | `0.41` | SAX-style XML parsing for CWE and CAPEC |
| `csv` | `1` | CSV parsing for ASVS |
| `zip` | `2` | ZIP decompression (deflate only) for CWE XML archive |
| `chrono` | `0.4` | DateTime parsing and formatting |
| `tokio` | `1` | Async runtime |
| `anyhow` | `1` | Error context and propagation |
| `thiserror` | `2` | Custom error type derivation |

`helix-db` is pinned to an exact version (`=3.0.0`) to prevent unintentional API
drift from the database client.

---

## Linting and formatting

```powershell
cargo fmt                  # format all source files
cargo fmt -- --check       # check formatting without modifying files
cargo clippy               # lint
cargo clippy -- -D warnings # treat warnings as errors
```

---

## Building the Docker image

```powershell
docker build -t cve-rag-mcp:0.1.0 .
```

The Dockerfile uses a two-stage build. Both base images are pinned to SHA256
digests. The build uses `--locked` to ensure the exact dependency versions in
`Cargo.lock` are used.

To rebuild from scratch (ignore Docker layer cache):

```powershell
docker build --no-cache -t cve-rag-mcp:0.1.0 .
```

---

## Common development tasks

### Wipe and re-seed HelixDB

```powershell
docker compose -f docker/compose.yml down -v          # delete volume
docker compose -f docker/compose.yml up -d helix       # fresh start
.\target\debug\cve-rag.exe backfill --limit 500
.\target\debug\cve-rag.exe source all
.\target\debug\cve-rag.exe status
```

### Watch for compilation errors

```powershell
cargo watch -x check       # requires cargo-watch: cargo install cargo-watch
```

### Inspect HelixDB directly

HelixDB exposes a raw HTTP query API on `http://localhost:47474`. Refer to the
`helix-db` crate documentation for DSL query syntax.

### Profile a search query

Because BM25 runs in-process, the standard Rust profiling tools apply.
For a quick timing measurement:

```powershell
Measure-Command { .\target\release\cve-rag.exe search "remote code execution" }
```

---

## Known limitations

- HelixDB `load_all()` performs a full table scan on every CLI search call.
  This is fast at current corpus sizes (~500 CVEs, ~2600 knowledge records)
  but will require caching or indexed queries as the corpus grows.
- The MCP server cache is in-process and is lost when the subprocess exits.
  The cache is rebuilt on the next search call after restart.
- HelixDB has no native upsert or transactions. The delete-then-insert upsert
  has a brief window where a record is absent if the process is interrupted.
  See [SECURITY-REVIEW.md](SECURITY-REVIEW.md) F-03 for details.
