# Zagros Documentation

Zagros is an open-source, local-first security knowledge system for AI agents.
Zagros Core provides ingestion, HelixDB-backed retrieval, CLI, MCP servers, and
UI; Zagros Skills provide harness-neutral investigation guidance that teaches
agents how to use retrieved evidence safely and correctly.

## Quick links

- [README](../README.md) — Quick start, setup, MCP tools summary
- [CLI.md](CLI.md) — All CLI subcommands, flags, options, and output formats
- [MCP.md](MCP.md) — MCP server tools, input/output schemas, safety guidance
- [ARCHITECTURE.md](ARCHITECTURE.md) — Component diagram, BM25 engine, HelixDB schema, Docker image
- [DATA-SOURCES.md](DATA-SOURCES.md) — Ingestion pipeline for each of the five sources
- [CONFIGURATION.md](CONFIGURATION.md) — Environment variables, Docker Compose, refresh scheduling, resource limits
- [DEPLOYMENT.md](DEPLOYMENT.md) — Local/cloud deployment, authentication boundary, recovery, upgrade, uninstall
- [DEVELOPMENT.md](DEVELOPMENT.md) — Build, test, code structure, common tasks
- [SECURITY-REVIEW.md](SECURITY-REVIEW.md) — Security findings F-01 through F-06 and their status
- [VISION.md](VISION.md) — Project goals, Core + Skills model, phased roadmap, design constraints
- [OPEN-SOURCE-RELEASE-TODO.md](OPEN-SOURCE-RELEASE-TODO.md) — Public-release readiness checklist and review status
- [../THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) — Upstream data-source licenses, attribution, and trademark notices

## Current pre-release state

| Component | Status |
|---|---|
| HelixDB storage | Active — `Cve` and `Knowledge` node labels |
| CVE ingestion | Active — delta feed + full deltaLog backfill |
| CWE ingestion | Active — 969 records |
| ASVS ingestion | Active — 345 records (v5.0.0 pinned) |
| CAPEC ingestion | Active — 613 records |
| ATT&CK ingestion | Active — 697 records |
| BM25 retrieval | Active — field weights, exact-match bonus, phrase bonus |
| CLI | Active — 7 subcommands |
| MCP servers | Active — 7 tools over stdio and Streamable HTTP |
| Knowledge MCP search/sync | Active — `search_knowledge`, `sync_knowledge_source` |
| Graph edges | Planned Phase 3 |
| Vector embeddings | Planned Phase 3 |
| Exact/graph knowledge tools | Planned Phase 3 — `get_rule`, `explain_weakness`, `map_to_attack`, `list_sources` |
| LLM reasoning layer | Planned Phase 4 |
