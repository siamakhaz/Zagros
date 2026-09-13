# Interface Stability Status

Zagros uses three lifecycle labels:

- **Stable** — intended for compatibility across compatible releases; breaking changes require explicit release notes/versioning.
- **Beta** — usable and supported, but details may change before a stable compatibility promise is made.
- **Experimental** — exploratory or roadmap capability; no compatibility promise.

## Current interfaces

| Interface / capability | Status | Notes |
|---|---|---|
| MCP `search_cves` | Stable | Read-only retrieval |
| MCP `get_cve` | Stable | Read-only exact lookup |
| MCP `index_status` | Stable | Read-only status/provenance metadata |
| MCP `search_knowledge` | Stable | Read-only knowledge retrieval |
| MCP `sync_cves` | Stable | Mutation/network tool; approval-gated |
| MCP `backfill_cves` | Stable | Mutation/network tool; approval-gated |
| MCP `sync_knowledge_source` | Stable | Mutation/network tool; approval-gated |
| CLI current commands | Stable | Compatibility changes follow release notes/SemVer |
| Skill Contract v1 | Stable | Versioned independently from Core |
| `cybersecurity-expert` skill package | Beta | Current package status is beta |
| Web UI | Beta | Useful interface without stable API compatibility promise |
| Graph relationships | Experimental / planned | Phase 3; no compatibility promise |
| Vector/hybrid retrieval | Experimental / planned | Phase 3; must pass retrieval benchmarks before stabilization |
| LLM reasoning layer | Experimental / planned | Phase 4; evidence boundaries remain mandatory |

## Policy

New externally visible interfaces start as Beta or Experimental unless explicitly documented as Stable. A Stable interface may still receive additive, backward-compatible changes. Breaking Stable changes require a versioned migration path or a major-version change where applicable.
