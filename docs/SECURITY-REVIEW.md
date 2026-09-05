# Security Review — Zagros v0.2

**Date:** 2026-08-16
**Reviewer:** OpenCode (cybersecurity-expert skill)
**Scope:** Full codebase review — `src/`, `docker/`, `Dockerfile`, `Cargo.toml`
**Phase:** 2 (structured ingestion and provenance)

---

## Executive Summary

The codebase is well-structured for a Phase 2 learning project. The retrieval
logic, MCP server design, and container hardening are all above the baseline for
a project at this stage. Two issues require prompt attention: the HelixDB port
is bound to all interfaces with no authentication, and the CVE ingestion path
deserializes JSON from external URLs without input size guards. The remaining
findings are medium-to-low priority and map directly onto the existing Phase 2–3
roadmap.

---

## Findings

### F-01 — HelixDB bound to all interfaces with no authentication

**Severity:** High
**File:** `docker/compose.yml:27`
**Weakness:** ATT&CK-T1133 (External Remote Services) — flagged in `VISION.md`

HelixDB is mapped as `47474:8080`, which binds to `0.0.0.0` on the host. Any
process or user on the same host or LAN can read, write, or delete the entire
knowledge corpus without credentials. HelixDB has no built-in authentication
layer, so the only available control is network exposure.

**Fix:**

```yaml
# docker/compose.yml
ports:
  - "127.0.0.1:47474:8080"   # restrict to loopback
```

This one-character change prevents network-adjacent access at no operational
cost. The MCP server container reaches HelixDB over the internal `zagros-net`
bridge network (`http://helix:8080`) and is unaffected by this change.

---

### F-02 — CVE ingestion deserializes external JSON without URL validation or size cap

**Severity:** High
**File:** `src/lib.rs:141–153` (`fetch_latest_cves`), `src/lib.rs:231–254` (`backfill_from_history`)
**Weakness:** CWE-502 (Deserialization of Untrusted Data) — flagged in `VISION.md`

`github_link` values from the CVE delta feed are followed and deserialized
without validating the URL scheme or origin, and without a payload size limit.
A poisoned feed or MITM response could deliver an arbitrarily large or
malformed payload to the `serde` deserializer.

Note: Rust's `serde` deserializer is not vulnerable to the class of object
instantiation attacks seen in Java. The practical risk here is denial of service
via memory exhaustion, not remote code execution.

**Fix:**

```rust
// In fetch_latest_cves and backfill_from_history — replace .json() with:
let bytes = response.bytes().await?;
anyhow::ensure!(
    bytes.len() < 10 * 1024 * 1024,
    "CVE record too large: {} bytes", bytes.len()
);
let cve: CveRecord = serde_json::from_slice(&bytes)?;
```

Additionally, validate that each `github_link` starts with the expected origin
before following it:

```rust
anyhow::ensure!(
    entry.github_link.starts_with(
        "https://raw.githubusercontent.com/CVEProject/cvelistV5/"
    ),
    "unexpected CVE link origin: {}", entry.github_link
);
```

---

### F-03 — `upsert_document` is not atomic

**Severity:** Medium
**File:** `src/db.rs:56–101`
**Weakness:** CWE-362 (Race Condition) — partial; non-concurrent but non-atomic

The upsert strategy is delete-then-insert. If the HelixDB connection drops or
the process is killed between the two operations, the record is permanently lost.
HelixDB v3 does not expose native transactions, so a fully atomic upsert is not
currently possible.

**Fix:** Wrap the pair with a retry on insert failure and log a warning when the
delete succeeded but the insert did not:

```rust
pub async fn upsert_document(client: &Client, doc: &CveDocument) -> Result<()> {
    let deleted = delete_cve_node(client, &doc.cve_id).await;
    for attempt in 0..2 {
        match insert_cve_node(client, doc).await {
            Ok(_) => return Ok(()),
            Err(e) if attempt == 0 => {
                eprintln!("insert attempt 1 failed for {}: {e}; retrying", doc.cve_id);
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

Document the residual risk in the module comment: a record can be temporarily
absent from the index between a failed upsert and the next successful run.

---

### F-04 — `sync_cves` MCP tool has no rate limit

**Severity:** Medium
**File:** `src/bin/zagros-mcp.rs:175–192`
**Weakness:** CWE-770 (Allocation of Resources Without Limits or Throttling)

The `sync_cves` tool description says it "requires user approval" but that
constraint is enforced by the agent client, not by the server. Any client that
ignores the convention — or any future tool that chains `sync_cves` automatically
— can trigger unlimited outbound HTTP requests to GitHub.

**Fix:** Add a last-sync timestamp guard on the server struct:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;

#[derive(Clone)]
struct CveMcpServer {
    last_sync: Arc<Mutex<Option<Instant>>>,
}

// In sync_cves:
let mut guard = self.last_sync.lock().await;
if let Some(last) = *guard {
    if last.elapsed().as_secs() < 300 {
        return Err(McpError::invalid_params(
            "sync_cves was called less than 5 minutes ago; wait before retrying",
            None,
        ));
    }
}
*guard = Some(Instant::now());
drop(guard);
```

---

### F-05 — `load_all` performs a full table scan on every search

**Severity:** Medium (performance; becomes a reliability issue at scale)
**File:** `src/db.rs:134–150`, `src/bin/zagros-mcp.rs:99–100`

Every `search_cves` and `index_status` call fetches all nodes from HelixDB
over HTTP. At 499 records this is fast. The `backfill` command supports up to
10,000 records; at that scale, each MCP search call loads ~5–10 MB of data
from HelixDB before scoring.

**Fix:** Add an in-process document cache with a TTL on the MCP server:

```rust
use std::time::{Duration, Instant};

struct DocCache {
    docs: Vec<CveDocument>,
    loaded_at: Instant,
}

#[derive(Clone)]
struct CveMcpServer {
    cache: Arc<Mutex<Option<DocCache>>>,
    last_sync: Arc<Mutex<Option<Instant>>>,
}

async fn get_docs(server: &CveMcpServer) -> Result<Vec<CveDocument>, McpError> {
    let mut guard = server.cache.lock().await;
    if let Some(ref c) = *guard {
        if c.loaded_at.elapsed() < Duration::from_secs(300) {
            return Ok(c.docs.clone());
        }
    }
    let helix = db::client().map_err(internal_error)?;
    let docs = db::load_all(&helix).await.map_err(internal_error)?;
    *guard = Some(DocCache { docs: docs.clone(), loaded_at: Instant::now() });
    Ok(docs)
}
```

`sync_cves` should invalidate the cache on success.

---

### F-06 — BM25 minimum score threshold not set

**Severity:** Low (latent; relevant at corpus sizes above ~5K)
**File:** `src/lib.rs:445–451`

Any document that shares a single low-IDF term with the query receives a
non-zero score and enters the result set. At `top_k=10` and 499 records the
effect is negligible. Above ~5K records, the bottom of a top-K result set can
contain loosely related documents that dilute signal.

**Fix:** Apply a minimum threshold before pushing hits:

```rust
const MIN_SCORE: f64 = 1.0;

if score > MIN_SCORE {
    hits.push(SearchHit { document: doc, score });
}
```

Adjust the constant empirically once the corpus grows. The same threshold
applies to `rank_knowledge`.

---

## What Is Working Well

| Area | Observation |
|---|---|
| MCP tool description | `search_cves` includes an explicit "untrusted reference data" warning in its tool description. Agent consumers receive this at tool-call time, not just in documentation. |
| `sync_cves` description | The tool description states it "makes network requests" and "clients should request user approval." This is the correct pattern for write/network tools. |
| CVE ID validation | `valid_cve_id` in `zagros-mcp.rs` correctly rejects path traversal inputs (tested), malformed IDs, and short year strings. |
| Container hardening | Dockerfile creates a non-root system user (`zagros`, uid 10001), copies only the compiled binary into the final image, and uses `debian:bookworm-slim`. |
| Idempotent ingestion | All `source` and `ingest` commands upsert rather than append. Re-running any ingestion command is safe. |
| Tokenizer design | Hyphens are preserved in tokenization, which is required for CVE IDs (`CVE-2026-17061`) to match as a single token rather than three unrelated terms. |
| BM25 field weights | ID weight ×8, title ×3, description ×1 — ensures an exact CVE ID lookup always outranks a partial description match. The `+100.0` exact-match bonus guarantees the correct record is ranked first. |
| Prompt injection awareness | The `warning` field in `SearchResponse` is included in every result payload. Agents that log or display the response will surface this warning automatically. |

---

## Phase 2 Roadmap Items That Address Security Gaps

The following items from `VISION.md` directly reduce security and auditability
risk and should be prioritized within Phase 2:

| Gap | Risk it closes |
|---|---|
| SHA-256 content hash per chunk | Detects silent corruption or substitution of ingested data |
| `retrieved_at` timestamp per node | Enables staleness detection and audit of when data entered the index |
| Source version field per node | Prevents a rule from an old standard version being applied to a newer artifact |
| License field per node | Required for redistribution compliance (MITRE terms, CC BY-SA) |
| `sync_source` MCP tool | Formalizes the approval requirement for knowledge source updates, same as `sync_cves` |

---

## Summary Table

| ID | Severity | File | Status |
|---|---|---|---|
| F-01 | High | `docker/compose.yml:27` | Open |
| F-02 | High | `src/lib.rs:141–254` | Open |
| F-03 | Medium | `src/db.rs:56–101` | Open |
| F-04 | Medium | `src/bin/zagros-mcp.rs:175` | Open |
| F-05 | Medium | `src/db.rs:134`, `zagros-mcp.rs:99` | Open |
| F-06 | Low | `src/lib.rs:445` | Open — defer until corpus > 5K |
