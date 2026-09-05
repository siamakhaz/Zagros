# Graph Report - .  (2026-08-17)

## Corpus Check
- 19 files · ~82,688 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 201 nodes · 360 edges · 16 communities detected
- Extraction: 66% EXTRACTED · 34% INFERRED · 0% AMBIGUOUS · INFERRED: 122 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Web UI Layer|Web UI Layer]]
- [[_COMMUNITY_CVE Data Models|CVE Data Models]]
- [[_COMMUNITY_Search and Ingestion Architecture|Search and Ingestion Architecture]]
- [[_COMMUNITY_MCP Server Implementation|MCP Server Implementation]]
- [[_COMMUNITY_CLI Commands and Configuration|CLI Commands and Configuration]]
- [[_COMMUNITY_MCP Tool Internals|MCP Tool Internals]]
- [[_COMMUNITY_Security Knowledge Sources|Security Knowledge Sources]]
- [[_COMMUNITY_Multi-Binary Architecture|Multi-Binary Architecture]]
- [[_COMMUNITY_Runtime Configuration|Runtime Configuration]]
- [[_COMMUNITY_CVE Schema|CVE Schema]]
- [[_COMMUNITY_Status Command|Status Command]]
- [[_COMMUNITY_Dev Prerequisites|Dev Prerequisites]]
- [[_COMMUNITY_Vision Architecture|Vision Architecture]]
- [[_COMMUNITY_Repository Layout|Repository Layout]]
- [[_COMMUNITY_Technology Stack|Technology Stack]]
- [[_COMMUNITY_Non-Goals|Non-Goals]]

## God Nodes (most connected - your core abstractions)
1. `Ok()` - 38 edges
2. `client()` - 20 edges
3. `load_all_knowledge()` - 12 edges
4. `CVE RAG Security Knowledge MCP Server` - 10 edges
5. `load_all()` - 9 edges
6. `sync_cves_to_helix()` - 9 edges
7. `rank_documents()` - 9 edges
8. `main()` - 9 edges
9. `search()` - 9 edges
10. `http_client()` - 8 edges

## Surprising Connections (you probably didn't know these)
- `row_to_doc()` --calls--> `Ok()`  [INFERRED]
  C:\Projects\RAG\cve-rag\src\db.rs → C:\Projects\RAG\cve-rag\scripts\setup.ps1
- `main()` --calls--> `Ok()`  [INFERRED]
  C:\Projects\RAG\cve-rag\src\bin\cve-rag-mcp.rs → C:\Projects\RAG\cve-rag\scripts\setup.ps1
- `main()` --calls--> `Ok()`  [INFERRED]
  C:\Projects\RAG\cve-rag\src\bin\cve-ui.rs → C:\Projects\RAG\cve-rag\scripts\setup.ps1
- `cve-rag logo image` --conceptually_related_to--> `CVE RAG Security Knowledge MCP Server`  [INFERRED]
  assets/cve-rag.png → README.md
- `cve-rag logo image` --conceptually_related_to--> `HelixDB graph-vector store`  [INFERRED]
  assets/cve-rag.png → README.md

## Hyperedges (group relationships)
- **Tier-1 ingestion pipeline** — readme_cve_project_delta_feed, readme_tier1_security_corpora, cli_ingest_command [EXTRACTED 0.95]
- **CVE MCP tool suite** — mcp_search_cves, mcp_get_cve, mcp_index_status, mcp_sync_cves [EXTRACTED 1.00]
- **Phase 3 graph-rich search vision** — vision_phase3_next_milestone, architecture_roadmap, architecture_bm25_retrieval_engine [EXTRACTED 0.90]

## Communities

### Community 0 - "Web UI Layer"
Cohesion: 0.1
Nodes (33): api_error(), AppState, cve(), cve_view(), cves(), CveView, knowledge(), knowledge_doc() (+25 more)

### Community 1 - "CVE Data Models"
Cohesion: 0.1
Nodes (22): CnaContainer, CveContainers, CveDelta, CveDeltaEntry, CveDocument, CveMetadata, CveRecord, data_file() (+14 more)

### Community 2 - "Search and Ingestion Architecture"
Cohesion: 0.11
Nodes (25): BM25 retrieval engine, DocCache, Docker image, MCP server internals, Delete-then-insert upsert strategy, interactive command, know command, search command (+17 more)

### Community 3 - "MCP Server Implementation"
Cohesion: 0.18
Nodes (20): CveMcpServer, get_docs(), count(), upsert_document(), upsert_documents(), backfill_from_history(), fetch_latest_cves(), http_client() (+12 more)

### Community 4 - "CLI Commands and Configuration"
Cohesion: 0.16
Nodes (22): cve-rag logo image, backfill command, ingest command, source command, Docker MCP Toolkit, Automated setup script, CVE Project delta feed (DATA-SOURCES), MITRE ATT&CK Enterprise (+14 more)

### Community 5 - "MCP Tool Internals"
Cohesion: 0.12
Nodes (11): CveToolRecord, DocCache, GetCveParams, IndexStatusResponse, main(), SearchParams, SearchResponse, SyncParams (+3 more)

### Community 6 - "Security Knowledge Sources"
Cohesion: 0.17
Nodes (15): AsvsRow, build_capec_doc(), ExternalReference, ingest_asvs(), ingest_attack(), ingest_capec(), ingest_cwe(), KillChainPhase (+7 more)

### Community 7 - "Multi-Binary Architecture"
Cohesion: 0.13
Nodes (16): cve-rag CLI, cve-rag-mcp MCP server, cve-ui web UI, HelixDB node schemas, Knowledge node, Roadmap, Shared library crate, Three-binary Rust project (+8 more)

### Community 8 - "Runtime Configuration"
Cohesion: 0.4
Nodes (5): CVE_RAG_DATA_DIR, Docker Compose configuration, Environment variables, HELIX_URL, UI_PORT

### Community 10 - "CVE Schema"
Cohesion: 1.0
Nodes (2): Cve node, CVE node schema

### Community 12 - "Status Command"
Cohesion: 1.0
Nodes (1): status command

### Community 13 - "Dev Prerequisites"
Cohesion: 1.0
Nodes (1): Prerequisites

### Community 14 - "Vision Architecture"
Cohesion: 1.0
Nodes (1): Vision Architecture

### Community 15 - "Repository Layout"
Cohesion: 1.0
Nodes (1): Repository Layout

### Community 16 - "Technology Stack"
Cohesion: 1.0
Nodes (1): Technology Stack

### Community 17 - "Non-Goals"
Cohesion: 1.0
Nodes (1): Non-Goals

## Knowledge Gaps
- **68 isolated node(s):** `CveRow`, `NodesResponse`, `KnowledgeRow`, `KnowledgeResponse`, `CveDelta` (+63 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `CVE Schema`** (2 nodes): `Cve node`, `CVE node schema`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Status Command`** (1 nodes): `status command`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dev Prerequisites`** (1 nodes): `Prerequisites`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Vision Architecture`** (1 nodes): `Vision Architecture`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Repository Layout`** (1 nodes): `Repository Layout`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Technology Stack`** (1 nodes): `Technology Stack`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Non-Goals`** (1 nodes): `Non-Goals`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Ok()` connect `MCP Server Implementation` to `Web UI Layer`, `CVE Data Models`, `MCP Tool Internals`, `Security Knowledge Sources`, `Setup Scripts`?**
  _High betweenness centrality (0.155) - this node is a cross-community bridge._
- **Why does `Three-binary Rust project` connect `Multi-Binary Architecture` to `Search and Ingestion Architecture`, `CLI Commands and Configuration`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Why does `BM25 retrieval engine` connect `Search and Ingestion Architecture` to `Multi-Binary Architecture`?**
  _High betweenness centrality (0.054) - this node is a cross-community bridge._
- **Are the 37 inferred relationships involving `Ok()` (e.g. with `upsert_document()` and `upsert_documents()`) actually correct?**
  _`Ok()` has 37 INFERRED edges - model-reasoned connections that need verification._
- **Are the 18 inferred relationships involving `client()` (e.g. with `backfill_from_history()` and `sync_cves_to_helix()`) actually correct?**
  _`client()` has 18 INFERRED edges - model-reasoned connections that need verification._
- **Are the 10 inferred relationships involving `load_all_knowledge()` (e.g. with `Ok()` and `ingest_cwe()`) actually correct?**
  _`load_all_knowledge()` has 10 INFERRED edges - model-reasoned connections that need verification._
- **Are the 7 inferred relationships involving `load_all()` (e.g. with `Ok()` and `search_cve()`) actually correct?**
  _`load_all()` has 7 INFERRED edges - model-reasoned connections that need verification._