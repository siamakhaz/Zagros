# Architecture Decision Records

Zagros uses Architecture Decision Records (ADRs) for durable architecture, security-boundary, compatibility, and maintenance decisions.

## Status values

- **Accepted** — current project decision.
- **Superseded** — replaced by a newer ADR.
- **Deprecated** — retained for history but no longer recommended.

## Current ADRs

- [ADR-001](ADR-001-helixdb-storage.md) — HelixDB as the primary storage backend
- [ADR-002](ADR-002-bm25-first-retrieval.md) — BM25-first retrieval before vector/graph search
- [ADR-003](ADR-003-local-first-deployment.md) — Local-first deployment architecture
- [ADR-004](ADR-004-core-skills-separation.md) — Separate Zagros Core and Zagros Skills
- [ADR-005](ADR-005-authoritative-sources-provenance.md) — Authoritative sources with explicit provenance
- [ADR-006](ADR-006-mcp-mutation-approval.md) — Approval-gated MCP mutation/network tools
- [ADR-007](ADR-007-rebuildable-corpus.md) — Rebuildable corpus; backups are optional
- [ADR-008](ADR-008-harness-neutral-skill-contract.md) — Harness-neutral Skill Contract

Use [ADR-TEMPLATE.md](ADR-TEMPLATE.md) for future decisions.
