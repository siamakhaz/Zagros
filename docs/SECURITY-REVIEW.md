# Security Review — Zagros Pre-Release

**Date:** 2026-09-12
**Scope:** Current source, Docker configuration, MCP layer, ingestion paths, retrieval behavior, and release-facing security controls.
**Purpose:** Revalidate the previous F-01 through F-06 findings before open-source release.

## Executive Summary

The previous six findings were rechecked against the current implementation.

- F-01, F-02, F-04, F-05, and F-06 are fixed.
- F-03 is mitigated as far as the current HelixDB v3 API permits; the residual non-atomic window is documented.
- No unresolved High-severity finding remains from the previous review.

The current code also retains positive controls including non-root runtime containers, MCP host allowlisting, explicit approval semantics for mutating/network tools, server-side rate limiting, cache invalidation after writes, and warnings that retrieved security text is untrusted reference data.

This review closes the previous P0 security-review blocker. Broader hardening work remains tracked in the open-source release TODO.

## Finding Status

| ID | Previous severity | Current status | Result |
|---|---:|---|---|
| F-01 | High | **Fixed** | HelixDB host port is loopback-only |
| F-02 | High | **Fixed** | CVE URLs are origin-validated and responses are size-limited |
| F-03 | Medium | **Mitigated** | Insert retry and residual-risk documentation are present |
| F-04 | Medium | **Fixed** | MCP mutation tools are server-side rate-limited |
| F-05 | Medium | **Fixed** | CVE and knowledge caches avoid full reload per search |
| F-06 | Low | **Fixed** | Minimum BM25 score threshold is enforced |
## F-01 — HelixDB network exposure

**Status:** Fixed.

`docker/compose.yml` publishes HelixDB only on `127.0.0.1:47474:8080`.

This prevents direct LAN/internet access through the host interface while allowing Zagros containers to reach HelixDB on the internal Docker network.

**Residual requirement:** operators must not change this to an all-interface binding on an untrusted network without an appropriate protective layer.

## F-02 — Untrusted CVE feed URLs and oversized payloads

**Status:** Fixed.

Current CVE ingestion validates individual CVE URLs against the expected `raw.githubusercontent.com/CVEProject/cvelistV5/` origin, reads response bytes before deserialization, rejects CVE records at or above the 10 MB guard, and only then deserializes with Serde.

This materially reduces origin-abuse and memory-exhaustion risk from unexpectedly large records.

## F-03 — Non-atomic HelixDB upsert

**Status:** Mitigated / accepted residual risk.

HelixDB v3 does not expose the transactional upsert needed to eliminate the delete-then-insert window. Current code documents the temporary-absence risk, retries insertion once, and reports persistent insertion failure.

This is acceptable for the current release. A future native atomic upsert should replace this path when available.
## F-04 — MCP synchronization rate limiting

**Status:** Fixed.

The MCP server tracks `last_sync`, `last_backfill`, and `last_knowledge_sync`. Mutating/network operations enforce an approximately five-minute cooldown server-side.

The client must still obtain explicit user approval because these operations make network requests and modify persistent data.

## F-05 — Full-table load on every MCP search

**Status:** Fixed.

The MCP layer now maintains `DocCache` for CVE records and `KnowledgeCache` for knowledge records. Both use a five-minute TTL, and successful synchronization/backfill invalidates the relevant cache.

## F-06 — Low-relevance BM25 results

**Status:** Fixed.

The retrieval implementation defines `MIN_SCORE = 1.0` and applies it to both CVE and security-knowledge ranking before results are returned.

The threshold should remain subject to regression testing as corpus size grows.

## Additional Security Controls Verified

- MCP HTTP uses a host allowlist through `MCP_ALLOWED_HOSTS`.
- HelixDB is not intended to be exposed directly.
- Docker runtime images use a non-root Zagros user where applicable.
- CVE identifiers are validated before exact lookup.
- Retrieval responses tell downstream agents to treat retrieved content as untrusted reference data.
- Write/network MCP tools are distinct from read/search tools.
- Synchronization invalidates in-process caches after successful writes.
## Post-review hardening status

Several hardening items identified during this dated review were completed during P1/P2:

1. Internet-facing MCP authentication guidance is documented in `DEPLOYMENT.md`.
2. Rust, npm, GitHub Actions, CodeQL, and container security automation is present in repository workflows.
3. Release automation produces checksums, SBOM/provenance metadata, signed container images, and immutable digests.
4. Structured provenance records publisher, source version, canonical/retrieval URLs, license, retrieval/upstream timestamps, content SHA-256, and trust tier.
5. `THREAT-MODEL.md` covers local, LAN, and public/cloud deployments.
6. Retrieval-quality and corpus-health regression tests are checked in and CI-gated.
7. Retrieved external security content remains explicitly classified as untrusted data, never agent instructions.

Additional ingestion/transport security regression coverage can continue to expand over time; it is not a blocker identified by F-01 through F-06.

## Release Assessment

**Previous P0 security findings:** no unresolved High finding remains.

**Release posture:** the reviewed F-01 through F-06 findings do not block release. Subsequent P1/P2 hardening is tracked in `OPEN-SOURCE-RELEASE-TODO.md`; remaining repository-side blockers are GitHub settings that depend on repository visibility/account capabilities.