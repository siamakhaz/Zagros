# Zagros — Project Vision

## Purpose

Zagros is an open-source, self-hosted security knowledge service for teams and their AI agents.
Its purpose is to give an organization or technical team a **shared, inspectable, controlled
security reference** that multiple humans, agents, IDE integrations, and internal tools can use
consistently without sending the security corpus to a hosted RAG provider.

The repository has two complementary parts:

1. **Zagros Core** — ingestion, HelixDB persistence, retrieval, CLI, MCP servers,
   and UI. Teams can run one instance on infrastructure they control and connect
   multiple MCP-compatible agent harnesses and internal tools to the same shared corpus.
2. **Zagros Skills** — independently authored agent guidance that teaches an AI
   agent how to investigate security questions, use Zagros evidence correctly,
   distinguish evidence from interpretation, and follow safe investigation
   practices.

Zagros favors authoritative upstream sources, explicit provenance, organizational control,
and evidence-backed conclusions. Local single-user deployment remains supported, but the primary
product model is a shared team service. The project is not tied to one agent harness, one model
provider, or one deployment environment.

---

## Current State (public pre-release baseline; example snapshot)

### What is working right now

The counts below are a captured example baseline, not live website telemetry. Verify any running instance with `zagros status` or MCP `index_status`.

| Component | Current implementation |
|---|---|
| Storage | HelixDB running in Docker (`ghcr.io/helixdb/helixdb:latest`), port `47474→8080` |
| CVE nodes | 499+ CVE records from CVEProject delta feed, `Cve` label in HelixDB |
| Knowledge nodes | 2,624 records across four sources, `Knowledge` label in HelixDB |
| Ingestion — CVE | `ingest` (latest delta) and `backfill` (deltaLog history, up to 10,000) |
| Ingestion — CWE | 969 MITRE CWE weaknesses from the versioned ZIP (v4.20, XML) |
| Ingestion — ASVS | 345 OWASP ASVS 5.0.0 requirements from the pinned CSV |
| Ingestion — CAPEC | 613 MITRE CAPEC attack patterns from the XML feed |
| Ingestion — ATT&CK | 697 MITRE ATT&CK Enterprise techniques from the STIX 2.1 bundle |
| Retrieval | BM25 lexical scoring with field weights — in-process, no round-trip |
| CLI | `ingest`, `backfill`, `source <name\|all>`, `search`, `know`, `interactive`, `status` |
| MCP server | Seven tools over stdio and Streamable HTTP: `search_cves`, `get_cve`, `index_status`, `sync_cves`, `search_knowledge`, `sync_knowledge_source`, `backfill_cves` |
| Docker Compose | `docker/compose.yml` starts HelixDB + MCP server as a stack |
| Docker MCP | `docker/zagros-server.yaml` registers the MCP server with Docker MCP Toolkit |
| Tests | Hosted CI covers formatting, clippy, Rust tests, website build, Compose validation, three Docker builds, security scanning, and deterministic HelixDB -> CLI -> MCP E2E |

### Example seeded corpus snapshot (captured 2026-09-08)

```
HelixDB URL  : http://localhost:47474
CVE nodes    : 499
  asvs        : 345
  attack      : 697
  capec       : 613
  cwe         : 969
  total       : 2,624 knowledge nodes
```

### Verified live against this repository

The `know` command was run against patterns present in this codebase:

| Pattern in code | Top knowledge result | Source |
|---|---|---|
| Hardcoded secrets / API keys | `ASVS-v5.0.0-V13.3.1` — secrets management vault requirement | ASVS |
| HTTP requests without timeout (some paths) | `CWE-1088` — synchronous remote call without timeout | CWE |
| Deserialization of untrusted CVE JSON from GitHub | `CWE-502` — deserialization of untrusted data | CWE |
| Unauthenticated HelixDB port `47474` | `ATT&CK-T1133` — external remote services | ATT&CK |
| TLS certificate validation | `CWE-295` — improper certificate validation | CWE |

### What is NOT yet implemented

| Gap | Phase it belongs to |
|---|---|
| HelixDB graph edges (ChildOf, MapsToTechnique, MitigatedBy, …) | Phase 3 |
| Dense vector embeddings and semantic search | Phase 3 |
| Hybrid retrieval with Reciprocal Rank Fusion | Phase 3 |
| Exact/graph-oriented knowledge tools: `get_rule`, `explain_weakness`, `map_to_attack`, `list_sources` | Phase 3 |
| Finding schema with evidence chains and confidence levels | Phase 4 |
| LLM reasoning layer with citation enforcement | Phase 4 |
| AST-based code analysis, IaC parser, SBOM | Phase 5 |
| Evaluation harness and calibration | Phase 6 |
| OWASP Cheat Sheets, NIST NCP, DISA STIGs, SEI CERT | Phase 3+ source expansion |

---

## Architecture

### Current layer diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                     AI Agent / LLM Client                        │
│           (OpenCode, Claude Desktop, VS Code, custom)            │
└─────────────────────────┬────────────────────────────────────────┘
                          │  Docker MCP Toolkit (stdio)
┌─────────────────────────▼────────────────────────────────────────┐
│                MCP Server (Rust / rmcp)                           │
│ search_cves  get_cve  index_status  sync_cves  search_knowledge  │
│ sync_knowledge_source  backfill_cves                              │
└──────┬──────────────────────────────────────────────────────────-┘
       │
┌──────▼──────────────────────────────────────────────────────────┐
│              BM25 Retrieval (in-process, lib.rs)                 │
│   rank_documents (Cve)    rank_knowledge (Knowledge)             │
└──────┬───────────────────────────────────────────────────────────┘
       │  HTTP  (helix-db crate → QueryRequest::read/write)
┌──────▼───────────────────────────────────────────────────────────┐
│                       HelixDB  :47474                            │
│  Cve nodes (499)    Knowledge nodes (2,624)                      │
│  No edges yet       No embeddings yet                            │
└──────────────────────────────────────────────────────────────────┘
```

### Target layer diagram (full vision)

```
┌──────────────────────────────────────────────────────────────────┐
│                     AI Agent / LLM Client                        │
└─────────────────────────┬────────────────────────────────────────┘
                          │  Docker MCP Toolkit (stdio / SSE)
┌─────────────────────────▼────────────────────────────────────────┐
│                    MCP Server (Rust / rmcp)                       │
│  search_knowledge  analyze_code  check_config  get_finding       │
│  sync_knowledge_source  index_status  list_sources  explain_rule  │
└──────┬──────────────────┬──────────────────────┬─────────────────┘
       │                  │                       │
┌──────▼──────┐  ┌────────▼────────┐  ┌──────────▼──────────────┐
│  Retrieval  │  │ Deterministic   │  │  Evidence Assembler     │
│  BM25       │  │ Analyzer        │  │  + Confidence Scorer    │
│  + Vectors  │  │ (pattern rules, │  │  (citation chains,      │
│  + Graph    │  │  AST, OPA)      │  │   trust tiers)          │
└──────┬──────┘  └────────┬────────┘  └──────────┬──────────────┘
       │                  │                       │
┌──────▼──────────────────▼───────────────────────▼──────────────┐
│                      HelixDB                                     │
│  Graph edges   →  CWE / CAPEC / ATT&CK / ASVS relationships    │
│  Vector store  →  Semantic embeddings of standard chunks        │
│  Document store→  Chunks with full provenance envelopes         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Source Catalog

### Tier 1 — Authoritative foundations

| Source | Records | Status | Format | License |
|---|---|---|---|---|
| [MITRE CWE 4.20](https://cwe.mitre.org/data/downloads.html) | 969 | **ingested** | XML ZIP | MITRE terms |
| [OWASP ASVS 5.0.0](https://github.com/OWASP/ASVS/tree/v5.0.0) | 345 | **ingested** | CSV | CC BY-SA 4.0 |
| [MITRE CAPEC 3.9](https://capec.mitre.org/data/downloads.html) | 613 | **ingested** | XML | MITRE terms |
| [MITRE ATT&CK Enterprise](https://attack.mitre.org/resources/attack-data-and-tools/) | 697 | **ingested** | STIX 2.1 | ATT&CK terms |
| [CVE Project delta feed](https://github.com/CVEProject/cvelistV5) | 499+ | **ingested** | JSON | per-CVE |
| [OWASP Cheat Sheets](https://cheatsheetseries.owasp.org/) | 0 | planned Phase 3+ | Markdown | CC BY-SA 4.0 |
| [NIST NCP](https://ncp.nist.gov/data-feeds) | 0 | planned Phase 3 | SCAP/XCCDF | Public |
| [DISA STIGs](https://public.cyber.mil/stigs/downloads/) | 0 | planned Phase 3 | XCCDF/XML | Public |
| [Kubernetes PSS](https://kubernetes.io/docs/concepts/security/pod-security-standards/) | 0 | planned Phase 3 | Structured docs | CC BY 4.0 |

### Tier 2 — Planned

| Source | Scope |
|---|---|
| SEI CERT Coding Standards | Language-specific secure coding rules (C, C++, Java, Perl) |
| OSV / GitHub Advisory Database | Dependency vulnerability lookup |
| Official language security docs | Python, Rust, Go, Java, .NET |

---

## Document Schema

### Current: `KnowledgeDoc` (`src/sources.rs`)

```rust
pub struct KnowledgeDoc {
    pub id: String,          // e.g. "CWE-89", "ASVS-v5.0.0-1.2.5", "ATT&CK-T1190"
    pub name: String,        // short human-readable title
    pub description: String, // full text used for BM25 search
    pub source: String,      // "cwe" | "asvs" | "capec" | "attack"
    pub url: String,         // canonical source URL
    pub tags: String,        // tactic, platform, chapter, level, etc.
    pub provenance: Provenance, // publisher/version/URLs/license/timestamps/hash/trust tier
}
```

### Current provenance envelope

```json
{
  "publisher": "OWASP Foundation",
  "source_name": "OWASP ASVS",
  "source_version": "5.0.0",
  "canonical_url": "https://github.com/OWASP/ASVS/tree/v5.0.0",
  "retrieval_url": "https://raw.githubusercontent.com/OWASP/ASVS/v5.0.0/...",
  "license": "CC BY-SA 4.0",
  "retrieved_at": "2026-09-14T00:00:00Z",
  "upstream_updated_at": null,
  "content_sha256": "<64-hex SHA-256>",
  "trust_tier": "authoritative"
}
```

---

## MCP Tools

### Currently exposed

| Tool | Description |
|---|---|
| `search_cves` | BM25 search over CVE nodes in HelixDB |
| `get_cve` | Exact lookup by CVE ID |
| `index_status` | CVE count, newest update, and knowledge counts by source |
| `sync_cves` | Download and upsert the latest CVE delta — requires user approval and is rate-limited |
| `search_knowledge` | BM25 search over CWE, ASVS, CAPEC, and ATT&CK knowledge |
| `sync_knowledge_source` | Re-ingest one knowledge source or all supported sources — requires user approval and is rate-limited |
| `backfill_cves` | Walk CVE delta history and upsert historical records — requires user approval and is rate-limited |

### Target (Phase 3+)

| Tool | Description | Phase |
|---|---|---|
| `get_rule` | Exact lookup by rule ID (`CWE-89`, `ASVS-v5.0.0-1.2.5`, etc.) | 3 |
| `explain_weakness` | Full CWE entry with related CAPEC and ATT&CK | 3 |
| `map_to_attack` | ATT&CK techniques for a given CWE or finding | 3 |
| `analyze_code` | Submit code snippet; returns findings with citations | 4 |
| `check_config` | Submit config fragment; returns findings | 4 |
| `get_remediation` | Remediation steps for a rule ID | 4 |
| `list_sources` | Indexed sources with version and record counts | 3 |

---

## Learning Path

### Phase 1 — Foundations (COMPLETE)

**Learning goal:** understand the retrieval problem before adding AI complexity.

- Rust async HTTP with `reqwest` + `tokio`
- `serde` JSON serialization
- BM25 from scratch (IDF, TF, field weights, length normalization)
- CLI with `clap`, MCP server with `rmcp`
- HelixDB Docker sidecar, Docker MCP Toolkit integration

**Key insight:** RAG is retrieval first. Good retrieval without an LLM already
provides value. An LLM on bad retrieval produces confident hallucinations.

### Phase 2 — Structured Ingestion and Provenance (COMPLETE)

**Learning goal:** understand why provenance, versioning, and schema discipline
matter from the start. Mistakes here cascade to every downstream component.

**Done:**
- Four authoritative sources parsed and stored in HelixDB: CWE, ASVS, CAPEC, ATT&CK
- Shared `KnowledgeDoc` type with structured `Provenance`
- Provenance stores publisher/source, source version, canonical/retrieval URLs, license, retrieval/upstream timestamps, SHA-256 content hash, and trust tier
- Source manifests record raw-source SHA-256 hashes; optional raw snapshots are preserved with `ZAGROS_PRESERVE_RAW_SOURCES`
- `source <name|all>` CLI command with progress reporting
- `know <query>` command for BM25 search over the knowledge corpus
- Idempotent upsert (safe to re-run any source ingestion)

**Deferred source expansion:**
- Add OWASP Cheat Sheets and other authoritative sources during Phase 3+

**Key insight:** A knowledge chunk without provenance is noise. You cannot
trust a finding if you cannot explain which version of which standard produced it.

### Phase 3 — HelixDB Graph + Vectors (NEXT)

**Learning goal:** understand when graph traversal outperforms vector similarity.

- Create CWE → ATT&CK technique edges using the official CWE/ATT&CK crosswalk
- Create CWE → ASVS requirement edges
- Generate embeddings for each chunk using a local or API-based model
- Store embeddings in HelixDB vector index
- Implement hybrid retrieval: BM25 + dense vectors + graph hops
- Fuse with Reciprocal Rank Fusion (RRF)
- Extend the current `search_knowledge` MCP capability with `get_rule`, `explain_weakness`, and `map_to_attack`

**Key insight:** "What controls address CWE-89?" is a graph traversal. "Find
chunks about injection in user-supplied data" is a vector query. Combining both
with RRF beats either alone.

### Phase 4 — LLM Reasoning Layer

**Learning goal:** the correct role of an LLM — reasoning over retrieved
evidence, never generating security claims from memory.

- LLM call that receives top-K cited chunks as structured context
- Structured output enforced with JSON schema validation in Rust
- Every LLM claim must include a cited evidence ID
- Quote anchoring: verify that an LLM-quoted phrase exists verbatim in the source
- `analyze_code` and `check_config` MCP tools

**Key insight:** the LLM explains and contextualizes findings that the retrieval
system already found. It does not produce findings and it does not adjudicate
compliance.

### Phase 5 — Security Scanner Integration

**Learning goal:** build the evidence collection layer.

- AST-based code pattern extraction for Rust
- Kubernetes and Dockerfile manifest parser
- Terraform/HCL IaC parser
- SBOM generation and dependency lookup via OSV / GitHub Advisory Database
- Connect scan output to MCP tools

### Phase 6 — Evaluation and Confidence Calibration

**Learning goal:** measure the system honestly and improve it systematically.

- OWASP Benchmark as evaluation harness
- Precision/recall per CWE/source
- Calibrate confidence scores against known-true findings
- Track false-positive rate as a first-class metric

---

## Repository Layout

### Current

```
zagros/
├── Cargo.toml                     # helix-db, quick-xml, csv, zip, rmcp, reqwest, …
├── Dockerfile                     # builds zagros-mcp image
├── docker/
│   ├── compose.yml                # HelixDB + MCP server stack
│   ├── zagros-server.yaml        # Docker MCP server registration
│   └── tools.json
├── src/
│   ├── lib.rs                     # Public API: ingest_*, rank_*, sync_*, backfill_*
│   ├── main.rs                    # CLI: ingest backfill source search know status
│   ├── db.rs                      # HelixDB client: Cve + Knowledge node CRUD
│   ├── sources.rs                 # KnowledgeDoc type + CWE/ASVS/CAPEC/ATT&CK parsers
│   └── bin/
│       ├── zagros-mcp.rs         # MCP stdio server
│       ├── zagros-mcp-http.rs    # MCP Streamable HTTP server
│       └── zagros-ui.rs          # Web UI/API
└── docs/
    ├── VISION.md                  # This document
    ├── README.md
    ├── CLI.md
    └── MCP.md
```

### Target (Phase 3+)

The flat `src/` layout will grow into subdirectories (`ingestion/`, `retrieval/`,
`analysis/`, `mcp/`) as each phase introduces clearly bounded components. The
current flat layout is intentional — it stays legible for learning without
premature abstraction.

---

## Design Constraints

**Evidence before claims.** No finding is reported without a retrieved chunk
that directly supports it. Semantic resemblance alone is `insufficient-evidence`.

**Version matching is a hard filter.** A rule inapplicable to the artifact's
language, platform, or version is excluded before retrieval. This is a
correctness requirement, not a performance hint.

**Source identity is preserved.** CIS, STIG, OWASP, and NIST controls that
address the same concern are never merged into an unlabeled generic rule. Each
finding traces to exactly one source and version.

**Provenance is immutable.** Raw downloads are stored as immutable snapshots
with content hashes. Derived chunks reference the snapshot. Updates produce new
records with new provenance; they do not overwrite old ones.

**Scanned code is untrusted.** Code comments, string literals, and documentation
in scanned repositories must never reach the system prompt as instructions.

**The LLM does not pass or fail.** Compliance verdicts come from the rule engine
or are marked `manual-review`. The LLM explains; it does not adjudicate.

**License compliance is structural.** Sources with redistribution restrictions
store only rule IDs and metadata in the index. Full text is linked, not embedded.

---

## Technology Stack

| Component | Choice | Status |
|---|---|---|
| Language | Rust 1.85+ | active |
| Database | HelixDB (Docker, `helix-db` crate) | active |
| MCP protocol | `rmcp` 3.1 | active |
| Agent connection | Docker MCP Toolkit | active |
| XML parsing | `quick-xml` 0.41 | active |
| CSV parsing | `csv` 1.4 | active |
| ZIP decompression | `zip` 2 | active |
| HTTP client | `reqwest` 0.12 | active |
| Embeddings | planned — ONNX local or API | Phase 3 |

---

## Non-Goals

- Does not produce CVSS scores.
- Does not automatically remediate findings.
- Does not replace Semgrep or CodeQL — it provides knowledge context for their output.
- Does not claim compliance certification for any standard.
- Is not a runtime intrusion detection system.

---

## Success Criteria

### Phase 2 (complete baseline)

- `source all` ingests all four Tier-1 sources without error and stores correct counts.
- `know "SQL injection"` returns CWE-89 and CAPEC-66 in the top three results.
- `know "hardcoded credentials"` returns ASVS-v5.0.0-V13.3.1 in the top five results.
- Provenance is attached to newly ingested CVE and knowledge records and exposed through MCP responses.
- Re-running any `source` command is idempotent (no duplicates).

### Phase 3 (next milestone)

- `know "what controls address SQL injection"` traverses CWE-89 → ATT&CK T1190 → ASVS requirements via graph edges.
- Graph/vector results preserve the existing provenance envelope through retrieval.
- Retrieval evaluation expands to larger source-diverse corpora before scanner-level false-positive targets are introduced.

### Full vision

- A query for "SQL injection in Rust" retrieves CWE-89, the relevant ASVS requirement,
  and the OWASP Injection Cheat Sheet within the top three results, each with exact
  source, version, and a supporting quotation.
- A submitted code snippet with a hardcoded credential is identified with a confirmed
  finding citing ASVS v5.0.0-2.10.1 and CWE-798.
- A Kubernetes manifest with `securityContext.privileged: true` is flagged citing
  Kubernetes Pod Security Standards Baseline.
- A finding without a retrieved evidence chain is returned as `insufficient-evidence`,
  never silently omitted or fabricated.
