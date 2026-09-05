# cve-rag Documentation

cve-rag is a Rust CLI and MCP server for ingesting, storing, and searching official
CVE records and authoritative security knowledge standards, backed by a local
HelixDB graph-vector store.

## Quick links

- [README](../README.md) — Quick start, setup, MCP tools summary
- [CLI.md](CLI.md) — All CLI subcommands, flags, options, and output formats
- [MCP.md](MCP.md) — MCP server tools, input/output schemas, safety guidance
- [ARCHITECTURE.md](ARCHITECTURE.md) — Component diagram, BM25 engine, HelixDB schema, Docker image
- [DATA-SOURCES.md](DATA-SOURCES.md) — Ingestion pipeline for each of the five sources
- [CONFIGURATION.md](CONFIGURATION.md) — Environment variables, Docker Compose, setup script
- [DEVELOPMENT.md](DEVELOPMENT.md) — Build, test, code structure, common tasks
- [SECURITY-REVIEW.md](SECURITY-REVIEW.md) — Security findings F-01 through F-06 and their status
- [VISION.md](VISION.md) — Project goals, phased roadmap, design constraints

## Current state (v0.2)

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
| MCP server | Active — 4 tools (CVE only) |
| Graph edges | Planned Phase 3 |
| Vector embeddings | Planned Phase 3 |
| Knowledge MCP tools | Planned Phase 3 |
| LLM reasoning layer | Planned Phase 4 |
