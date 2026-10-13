# Architecture

## Core and Skills boundary

**Core** comprises the Rust service binaries, shared ingestion/retrieval code, and HelixDB storage. The CLI, MCP servers, and web UI expose retrieved source-attributed evidence. Core can function without an agent skill. Server-side access restrictions for a shared deployment must be supplied by an authenticated gateway; MCP client approval guidance alone is not authorization.

**Skills** are separately versioned, optional agent guidance packaged under `skills/` and governed by [the Skill Contract](SKILL-CONTRACT.md). They specify how a compatible agent should distinguish evidence from analysis, cite sources, and seek approval for writes. They neither execute ingestion on their own nor establish that any target is vulnerable. Retrieved source text is untrusted data, not agent instructions.

Current capabilities and limitations must be verified against source and tests; see [claim audit](CLAIM-AUDIT.md) and [deployment security](DEPLOYMENT.md).

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

## Component and data flow

For a short user-facing explanation see [How Zagros works](HOW-IT-WORKS.md).

```text
Upstream source feeds -> fetch / validate / normalize -> HelixDB (Cve, Knowledge)
                                     |                        |
                              provenance metadata            v
                                               application retrieval + BM25
                                                         |
                                               CLI / MCP / Web UI
```

CLI and the three MCP write tools can initiate ingestion. The retrieval logic is in
the shared Rust library; MCP provides both stdio and Streamable HTTP transports.
The UI is a separate HTTP server. All retrieved upstream content is untrusted
reference material, never instructions to agents. HelixDB stores nodes; it is not
an autonomous vulnerability scanner or a server-side authorization layer.

Corpus counts are installation-dependent. Inspect live status instead of relying
on historical seed examples.

---

## Source layout

```
zagros/
├── Cargo.toml                  # dependencies, edition 2024
├── Cargo.lock                  # pinned dependency graph
├── Dockerfile                  # multi-stage build → debian:bookworm-slim
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
| `cve_id` | String | Primary key pattern — e.g. `CVE-2024-3094` |
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


### Structured provenance

Both CVE and knowledge records carry source provenance used by MCP responses and health checks:

| Field | Purpose |
|---|---|
| `publisher` | authoritative publisher/maintainer |
| `source` / `source_version` | corpus identity and version |
| `canonical_url` | canonical evidence location |
| `retrieval_url` | exact retrieval origin |
| `license` | upstream licensing metadata |
| `retrieved_at` | successful retrieval timestamp |
| `upstream_updated_at` | upstream update timestamp when available |
| `content_sha256` | normalized content integrity hash |
| `trust_tier` | source trust classification |

Legacy rows without complete provenance are identified as `legacy_unknown` until refreshed.

---

## BM25 retrieval engine

The engine lives entirely in `src/lib.rs` and runs in-process — no round-trip
to HelixDB for search, only for the initial data load.

### Tokenization

- Split on any character that is not alphanumeric or a hyphen.
- Discard tokens with length ≤ 1.
- Lowercase all tokens.
- Hyphens are preserved so `CVE-2024-3094` tokenizes as a single token, not three.

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

`zagros-mcp` is a stdio-transport MCP server built with the repository-locked `rmcp` dependency.
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

## Docker images

The MCP `Dockerfile` uses a Go builder for Supercronic, a Rust builder for
the CLI and both MCP binaries, and a Debian Bookworm runtime image. The runtime
runs as an unprivileged `zagros` user and includes `zagros`,
`zagros-mcp`, and `zagros-mcp-http`. Its daily refresh mechanism executes
through the bundled scheduler and entrypoint. Image dependencies are updated
during building; security scans remain required.

`Dockerfile.cli` builds the standalone CLI image, and `Dockerfile.ui`
builds the UI service image. See [deployment](DEPLOYMENT.md) for networking,
mount permissions, and access controls.

---

## Roadmap

The authoritative roadmap is [../ROADMAP.md](../ROADMAP.md). This architecture document intentionally does not duplicate phase status because implementation and release-readiness work evolve independently.

Current stable foundations include BM25 retrieval, structured provenance, corpus health metrics, CI-gated retrieval regression tests, Core/Skills separation, and seven MCP tools. Graph relationships, vector/hybrid retrieval, and assisted reasoning remain experimental/planned and must satisfy the interface and evidence-boundary policies before stabilization.
