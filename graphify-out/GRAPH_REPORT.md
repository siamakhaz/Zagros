# Graph Report - Zagros  (2026-09-11)

## Corpus Check
- 83 files · ~290,850 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1739 nodes · 2169 edges · 186 communities (174 shown, 12 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 105 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `3364586a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- client
- cve-rag/src/lib.rs
- Three-binary Rust project
- rank_documents
- CVE RAG Security Knowledge MCP Server
- cve-rag-mcp.rs
- cve-rag/src/sources.rs
- src/lib.rs
- Environment variables
- Cve node
- status command
- Prerequisites
- Vision Architecture
- Repository Layout
- Technology Stack
- Non-Goals
- mcp.rs
- Module 2.2: Disaster Recovery
- 4.1 Computer Networking
- zagros-ui.rs
- Module 2.1: Business Continuity
- KnowledgeDoc
- Docker MCP Gateway on Linux
- Docker MCP Gateway on Linux
- content.d.ts
- Docker MCP Gateway on Windows (Without Docker Desktop)
- Docker MCP Gateway on Windows (Without Docker Desktop)
- Module 1.1: Information Assurance Concepts
- Module 1.3: Security Controls
- Module 1.5: Governance Processes
- Development
- MCP Server Reference
- Zagros — Security Knowledge MCP Server
- Cybersecurity Expert Skill
- AGENTS.md — Zagros
- cve-rag/src/db.rs
- properties
- demo.astro
- package.json
- Testing the Deployed Application
- 2. Common Attack Types
- Testing the Deployed Application
- The Six IR Phases
- Module 2.3: Incident Response
- 4.3 Network Security Infrastructure
- Architecture
- Module 1.2: Risk Management Process
- Monitoring
- Data Sources
- CLI Reference
- Security RAG — Project Vision
- On-Premises Infrastructure
- Network Design
- 5.1 Data Security and Cryptography
- Module 1.4: ISC2 Code of Ethics
- Physical Security Controls
- ISC2 CC Domain 3.2 Logical Access Controls
- 4.2 Network Threats and Attacks
- Common MITM methods
- 2. Symmetric Encryption
- 5.3 Security Policies
- 5.4 Security Awareness Training
- setup.sh
- Security Policy
- ISC2 CC Domain 3.1 Physical Access Controls
- Authorized vs Unauthorized Personnel
- Mandatory Access Control (MAC)
- Denial of Service Attacks
- Malware Types
- Cloud Security Fundamentals
- 5.2 System Hardening
- zagros-skill — cybersecurity skill + Zagros MCP wiring
- Configuration
- Locks
- Account Management Lifecycle
- Discretionary Access Control (DAC)
- 3. Asymmetric Encryption
- 5. Hashing
- 6. Change Management Policy
- Security Review — Zagros v0.2
- Findings
- Learning Path
- tsconfig.json
- The ISC2 Code of Ethics
- Ethical Scenarios (Exam-Style)
- Evidence Handling and Chain of Custody
- Real-World Breach Examples
- Communication During Incidents
- Gate Entry Controls
- CCTV
- AI in Access Controls
- Principle of Least Privilege
- Separation of Duties
- Role-Based Access Control (RBAC)
- Prevention Technologies
- VPNs
- 12. Logging and SIEM
- 7. PKI and Certificates
- 3. Password Policy
- 5. BYOD Policy
- 8. AI Workspace Security
- Zagros MCP appendix (concrete workflow)
- BM25 retrieval engine
- HelixDB
- Risk Treatment Options
- Risk Management Lifecycle
- AI in Incident Response
- Incident Classification
- Notification Requirements
- Detection Technologies
- Deployment models
- 13. Hardening Across the Lifecycle
- 4. Patch Management
- 6. Operating System Hardening
- Environment variables
- Current State (v0.2)
- Risk Identification Methods
- Risk Assessment: Qualitative vs Quantitative
- Biometrics
- Access Control Fundamentals
- MFA Rules
- Phishing Variants
- Supply Chain Attacks
- AI-Enhanced Network Security
- SLA and Uptime
- 11. Data Destruction
- 4. Hybrid Encryption and TLS
- 6. Digital Signatures
- 8. Key Management
- 9. Data Classification
- 11. Secure Configuration Practices
- 14. Verification and Validation
- 1. Configuration Management
- 7. Application Hardening
- 8. Network Hardening
- 13. Policy Communication
- 2. Data Handling Policy
- 4. Acceptable Use Policy
- 7. Privacy Policy
- 1. Social Engineering Basics
- 3. Password Protection Awareness
- 4. Clean Desk and Clean Screen
- 5. Reporting Culture
- 6. Training Program Design
- 7. Measuring Effectiveness
- INSTALL-PROMPT — paste this to your agent (any harness)
- zagros-mcp.rs
- zagros-mcp-http.rs
- MCP server internals
- Success Criteria
- 1. Preparation
- 5. Recovery
- 6. Lessons learned
- Threat Actors
- Side-Channel Attacks
- 10. Change Management and Hardening
- 3. Hardening Steps
- 5. CVSS and Risk Prioritization
- 9. Database Hardening
- 10. Policy Enforcement
- 11. Policy Lifecycle and Maintenance
- 12. Writing Better Policies
- 1. Policy in the Security Program
- 11. Security Champions and Peer Influence
- 12. Incident Response for User-Reported Events
- 13. Microlearning and Repetition
- 9. Compliance Training Requirements
- Architecture
- MCP Tools
- Source Catalog
- install.sh script
- zagros
- zagros-skill

## God Nodes (most connected - your core abstractions)
1. `CveDocument` - 23 edges
2. `KnowledgeDoc` - 22 edges
3. `client()` - 20 edges
4. `CveMcpServer` - 19 edges
5. `5.2 System Hardening` - 19 edges
6. `5.3 Security Policies` - 19 edges
7. `5.4 Security Awareness Training` - 19 edges
8. `4.2 Network Threats and Attacks` - 17 edges
9. `5.1 Data Security and Cryptography` - 17 edges
10. `ISC2 CC Domain 3.2 Logical Access Controls` - 16 edges

## Surprising Connections (you probably didn't know these)
- `CVE Project delta feed` --semantically_similar_to--> `CVE Project delta feed (DATA-SOURCES)`  [INFERRED] [semantically similar]
  README.md → docs/DATA-SOURCES.md
- `Tier-1 security knowledge corpora` --semantically_similar_to--> `Source Catalog`  [INFERRED] [semantically similar]
  README.md → docs/VISION.md
- `CVE RAG Security Knowledge MCP Server` --references--> `cve-rag Documentation Overview`  [EXTRACTED]
  README.md → docs/README.md
- `CVE Project delta feed` --references--> `backfill command`  [EXTRACTED]
  README.md → docs/CLI.md
- `CVE Project delta feed` --references--> `ingest command`  [EXTRACTED]
  README.md → docs/CLI.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Tier-1 ingestion pipeline** — readme_cve_project_delta_feed, readme_tier1_security_corpora, docs_cli_ingest_command [EXTRACTED 0.95]
- **CVE MCP tool suite** — docs_mcp_search_cves, docs_mcp_get_cve, docs_mcp_index_status, docs_mcp_sync_cves [EXTRACTED 1.00]
- **Phase 3 graph-rich search vision** — docs_vision_phase3_next_milestone, docs_architecture_roadmap, docs_architecture_bm25_retrieval_engine [EXTRACTED 0.90]

## Communities (186 total, 12 thin omitted)

### Community 0 - "client"
Cohesion: 0.17
Nodes (19): api_error(), AppState, cve(), cve_view(), cves(), CveView, knowledge(), knowledge_doc() (+11 more)

### Community 1 - "cve-rag/src/lib.rs"
Cohesion: 0.12
Nodes (18): CnaContainer, CveContainers, CveDelta, CveDeltaEntry, CveDocument, CveMetadata, CveRecord, data_file() (+10 more)

### Community 2 - "Three-binary Rust project"
Cohesion: 0.06
Nodes (42): BM25 retrieval engine, cve-rag CLI, cve-rag-mcp MCP server, cve-ui web UI, DocCache, Docker image, HelixDB node schemas, Knowledge node (+34 more)

### Community 3 - "rank_documents"
Cohesion: 0.21
Nodes (14): load_all(), exact_cve_id_is_ranked_first(), phrase_and_title_matches_receive_higher_score(), rank_documents(), rank_knowledge(), term_counts(), tokenize(), Cli (+6 more)

### Community 4 - "CVE RAG Security Knowledge MCP Server"
Cohesion: 0.16
Nodes (20): backfill command, ingest command, source command, Docker MCP Toolkit, Automated setup script, CVE Project delta feed (DATA-SOURCES), MITRE ATT&CK Enterprise, MITRE CAPEC (+12 more)

### Community 5 - "cve-rag-mcp.rs"
Cohesion: 0.11
Nodes (12): CveMcpServer, CveToolRecord, DocCache, get_docs(), GetCveParams, IndexStatusResponse, SearchParams, SearchResponse (+4 more)

### Community 6 - "cve-rag/src/sources.rs"
Cohesion: 0.13
Nodes (22): upsert_knowledge(), upsert_knowledge_batch(), http_client(), ingest_asvs(), ingest_attack(), ingest_capec(), ingest_cwe(), AsvsRow (+14 more)

### Community 7 - "src/lib.rs"
Cohesion: 0.09
Nodes (53): Client, DateTime, HashMap, PathBuf, backfill_from_history(), CnaContainer, CveContainers, CveDelta (+45 more)

### Community 8 - "Environment variables"
Cohesion: 0.40
Nodes (5): CVE_RAG_DATA_DIR, Docker Compose configuration, Environment variables, HELIX_URL, UI_PORT

### Community 18 - "mcp.rs"
Cohesion: 0.10
Nodes (37): Default, Error, Instant, McpError, Mutex, Parameters, Schema, SchemaGenerator (+29 more)

### Community 19 - "Module 2.2: Disaster Recovery"
Cohesion: 0.04
Nodes (48): Analogy, Archive bit behavior, Backup Strategies, Backup vs RAID, Choosing the strategy, Cloud outage scenario, Cloud recovery, Cold site (+40 more)

### Community 20 - "4.1 Computer Networking"
Cohesion: 0.04
Nodes (48): 4.1 Computer Networking, Answers, Core port table, Exam-Relevant Ports, Exam Tips, Fast memorization chart, IPv4, IPv4 basics (+40 more)

### Community 21 - "zagros-ui.rs"
Cohesion: 0.11
Nodes (42): BTreeMap, Display, Html, Json, Path, Query, Response, main() (+34 more)

### Community 22 - "Module 2.1: Business Continuity"
Cohesion: 0.05
Nodes (40): AI in Business Continuity, BC Plan Components, BC plan quality checklist, BC planning for AI systems, BC Testing Types, BCP Lifecycle, BIA and risk, BIA example (+32 more)

### Community 23 - "KnowledgeDoc"
Cohesion: 0.15
Nodes (39): client(), count(), count_knowledge_by_source(), CveRow, get_by_id(), helix_url(), KnowledgeResponse, KnowledgeRow (+31 more)

### Community 24 - "Docker MCP Gateway on Linux"
Cohesion: 0.05
Nodes (38): 1. Build the Image, 1. Clone and Build the CLI Plugin, 2. Create a Local Server Definition, 2. Set the Required Environment Variable, 3. Add to a Profile, 3. Enable Profiles, 4. Run the Gateway, Browse Available Servers (+30 more)

### Community 25 - "Docker MCP Gateway on Linux"
Cohesion: 0.05
Nodes (38): 1. Build the Image, 1. Clone and Build the CLI Plugin, 2. Create a Local Server Definition, 2. Set the Required Environment Variable, 3. Add to a Profile, 3. Enable Profiles, 4. Run the Gateway, Browse Available Servers (+30 more)

### Community 26 - "content.d.ts"
Cohesion: 0.05
Nodes (32): AllValuesOf, AnyEntryMap, astro:content, CollectionEntry, CollectionKey, ContentCollectionKey, ContentConfig, ContentEntryMap (+24 more)

### Community 27 - "Docker MCP Gateway on Windows (Without Docker Desktop)"
Cohesion: 0.06
Nodes (33): Comparison: WSL2 vs Native Windows, Connecting to opencode on Windows, Connecting to Other Clients on Windows, Container cannot reach HelixDB on the host, `Docker Desktop is not running`, `docker mcp` command not found, Docker MCP Gateway on Windows (Without Docker Desktop), If using Native Windows (Path B) (+25 more)

### Community 28 - "Docker MCP Gateway on Windows (Without Docker Desktop)"
Cohesion: 0.06
Nodes (33): Comparison: WSL2 vs Native Windows, Connecting to opencode on Windows, Connecting to Other Clients on Windows, Container cannot reach HelixDB on the host, `Docker Desktop is not running`, `docker mcp` command not found, Docker MCP Gateway on Windows (Without Docker Desktop), If using Native Windows (Path B) (+25 more)

### Community 29 - "Module 1.1: Information Assurance Concepts"
Cohesion: 0.08
Nodes (25): AI and the CIA Triad, AI Security Fundamentals (New for 2025), Authentication Deep Dive, Authentication Factors, Authentication vs Non-Repudiation, Availability, Biometrics Key Terms, CIA Conflicts and Trade-offs (+17 more)

### Community 30 - "Module 1.3: Security Controls"
Cohesion: 0.09
Nodes (23): Administrative Controls (Managerial Controls), Compensating Controls, Control Classification Matrix, Control Selection Criteria, Controls by Function, Controls by Implementation Type, Corporate Defense in Depth Example, Corrective Controls (+15 more)

### Community 31 - "Module 1.5: Governance Processes"
Cohesion: 0.10
Nodes (21): Compliance vs Security, GDPR (General Data Protection Regulation), Governance vs Management, GRC (Governance, Risk, and Compliance), Guidelines, HIPAA (Health Insurance Portability and Accountability Act), Key Regulations (Awareness Level for CC), Learning Objectives (+13 more)

### Community 32 - "Development"
Cohesion: 0.10
Nodes (21): Adding a new CLI subcommand, Adding a new knowledge source, Adding a new MCP tool, Build, Build only one binary, Building the Docker image, Check without producing binaries, Code structure (+13 more)

### Community 33 - "MCP Server Reference"
Cohesion: 0.10
Nodes (21): `backfill_cves`, Build the image, Connect a client, Docker image hardening, Docker server registration (`docker/server.yaml`), `get_cve`, `index_status`, MCP Server Reference (+13 more)

### Community 34 - "Zagros — Security Knowledge MCP Server"
Cohesion: 0.10
Nodes (20): 1. Start the stack, 2. Build (host), 3. Populate CVE records, 4. Populate the security knowledge base, 5. Search, Agent guidance, Configuration, Documentation (+12 more)

### Community 35 - "Cybersecurity Expert Skill"
Cohesion: 0.11
Nodes (17): Core Knowledge Domains, CVE Research and Tool Selection, Cybersecurity Expert Skill, Domain 1: Security Principles & Risk Management, Domain 2: Resilience & Response, Domain 3: Access Control Design, Domain 4: Network Defense, Domain 5: Security Operations (+9 more)

### Community 36 - "AGENTS.md — Zagros"
Cohesion: 0.12
Nodes (16): AGENTS.md — Zagros, Binaries, Code structure — what lives where, Developer commands, Extension patterns, First-run seeding sequence, graphify knowledge graph, HelixDB is a required sidecar — start it first (+8 more)

### Community 37 - "cve-rag/src/db.rs"
Cohesion: 0.16
Nodes (12): count(), count_knowledge_by_source(), CveRow, helix_url(), KnowledgeResponse, KnowledgeRow, NodesResponse, upsert_document() (+4 more)

### Community 38 - "properties"
Cohesion: 0.12
Nodes (15): definitions, docs, type, additionalProperties, properties, type, type, description (+7 more)

### Community 39 - "demo.astro"
Cohesion: 0.15
Nodes (7): box, go, payload, q, search(), tok(), all

### Community 40 - "package.json"
Cohesion: 0.13
Nodes (14): typescript, dependencies, astro, devDependencies, typescript, astro, name, private (+6 more)

### Community 41 - "Testing the Deployed Application"
Cohesion: 0.14
Nodes (13): Check gateway starts and lists tools, Check index status, Layer 1 — HelixDB, Layer 2 — CLI, Layer 3 — MCP gateway, Layer 4 — opencode MCP, Layer 5 — End-to-end via opencode headless, List connected MCP servers (+5 more)

### Community 42 - "2. Common Attack Types"
Cohesion: 0.14
Nodes (14): 2. Common Attack Types, Baiting, Business Email Compromise, Comparison table, Deepfakes, Phishing, Pretexting, Quid Pro Quo (+6 more)

### Community 43 - "Testing the Deployed Application"
Cohesion: 0.14
Nodes (13): Check gateway starts and lists tools, Check index status, Layer 1 — HelixDB, Layer 2 — CLI, Layer 3 — MCP gateway, Layer 4 — opencode MCP, Layer 5 — End-to-end via opencode headless, List connected MCP servers (+5 more)

### Community 44 - "The Six IR Phases"
Cohesion: 0.17
Nodes (12): 2. Detection and analysis, 3. Containment, 4. Eradication, Activities, Activities, Example, Example, Long-term containment (+4 more)

### Community 45 - "Module 2.3: Incident Response"
Cohesion: 0.17
Nodes (12): Common Mistakes and Exam Pointers, Event vs incident, Example, Incident Response Team Roles, Learning Objectives, Memory aid, Module 2.3: Incident Response, Practice Questions (+4 more)

### Community 46 - "4.3 Network Security Infrastructure"
Cohesion: 0.17
Nodes (9): 4.3 Network Security Infrastructure, Answers, Difference, Exam Tips, Infrastructure Example, Learning goals, MSP and MSSP, Practice Questions (+1 more)

### Community 47 - "Architecture"
Cohesion: 0.17
Nodes (10): Architecture, Component diagram, `Cve` node, Docker image, HelixDB node schemas, `Knowledge` node, Overview, Roadmap (+2 more)

### Community 48 - "Module 1.2: Risk Management Process"
Cohesion: 0.18
Nodes (11): Key Risk Terminology, Learning Objectives, Module 1.2: Risk Management Process, NIST Risk Management Framework (RMF), Practice Questions, Risk Appetite, Tolerance, and Threshold, The Building Analogy, The Risk Equation Visualized (+3 more)

### Community 49 - "Monitoring"
Cohesion: 0.18
Nodes (11): Advantages, Alarm Systems, Exam tip, Exam tip, Limitations, Logs and Audit Trails, Monitoring, Real-world example (+3 more)

### Community 50 - "Data Sources"
Cohesion: 0.18
Nodes (9): CVE Project delta feed, Data flow summary, Data Sources, MITRE ATT&CK Enterprise, MITRE CAPEC, MITRE CWE, OWASP ASVS 5.0.0, Planned sources (Phase 2–3) (+1 more)

### Community 51 - "CLI Reference"
Cohesion: 0.18
Nodes (11): `backfill`, CLI Reference, Environment variables, Exit codes, Global options, `ingest`, `interactive`, `know` (+3 more)

### Community 52 - "Security RAG — Project Vision"
Cohesion: 0.18
Nodes (11): Current, Current: `KnowledgeDoc` (`src/sources.rs`), Design Constraints, Document Schema, Non-Goals, Purpose, Repository Layout, Security RAG — Project Vision (+3 more)

### Community 53 - "On-Premises Infrastructure"
Cohesion: 0.20
Nodes (10): Data centers, Fire suppression, Fire suppression caution, HVAC, MOU and MOA, On-Premises Infrastructure, Power protection, RAID (+2 more)

### Community 54 - "Network Design"
Cohesion: 0.20
Nodes (10): Defense in depth, DMZ, Dual-firewall architecture, IoT security, Micro-segmentation, NAC and 802.1X, Network Design, Segmentation (+2 more)

### Community 55 - "5.1 Data Security and Cryptography"
Cohesion: 0.20
Nodes (10): 10. Data States and Lifecycle, 1. Cryptography Fundamentals, 5.1 Data Security and Cryptography, Exam Tips, Lifecycle view, Navigation, Plaintext, Ciphertext, Algorithm, and Key, Practice Questions (+2 more)

### Community 56 - "Module 1.4: ISC2 Code of Ethics"
Cohesion: 0.22
Nodes (9): Consequences of Ethics Violations, DO:, DON'T:, Exam Strategy for Ethics Questions, Learning Objectives, Module 1.4: ISC2 Code of Ethics, Practice Questions, Professional Conduct Requirements (+1 more)

### Community 57 - "Physical Security Controls"
Cohesion: 0.22
Nodes (9): Badge Systems, CPTED, Exam tip, Exam tip, Exam tip, Fencing, Physical Security Controls, Real-world example (+1 more)

### Community 58 - "ISC2 CC Domain 3.2 Logical Access Controls"
Cohesion: 0.22
Nodes (8): Exam Tips, How to Remember the Differences, ISC2 CC Domain 3.2 Logical Access Controls, Learning Objectives, Master Comparison Table, Practice Questions, Putting It All Together, Summary

### Community 59 - "4.2 Network Threats and Attacks"
Cohesion: 0.22
Nodes (9): 4.2 Network Threats and Attacks, Answers, Defense ideas, Exam Tips, Learning goals, Practice Questions, Putting It Together, Why this module matters (+1 more)

### Community 60 - "Common MITM methods"
Cohesion: 0.22
Nodes (9): ARP spoofing, Common MITM methods, Evil twin, Man-in-the-Middle Attacks, MITM analogy, MITM prevention, Rogue access point, Session hijacking (+1 more)

### Community 61 - "2. Symmetric Encryption"
Cohesion: 0.22
Nodes (9): 2. Symmetric Encryption, 3DES, AES, Blowfish, Common Symmetric Algorithms, DES, Example, RC4 (+1 more)

### Community 62 - "5.3 Security Policies"
Cohesion: 0.22
Nodes (9): 14. Policy Case Study, 5.3 Security Policies, 8. How Policies Work Together, 9. Common Exam Traps, Exam Tips, Example package, Navigation, Practice Questions (+1 more)

### Community 63 - "5.4 Security Awareness Training"
Cohesion: 0.22
Nodes (9): 10. Building a Strong Awareness Program, 14. Specialized Audience Training, 5.4 Security Awareness Training, Exam Tips, Navigation, Practice Questions, What good looks like, Why Awareness Matters (+1 more)

### Community 64 - "setup.sh"
Cohesion: 0.39
Nodes (7): DOCKER_MCP_IN_CONTAINER, setup.sh script, fail(), ok(), step(), warn(), ZAGROS_DATA_DIR

### Community 65 - "Security Policy"
Cohesion: 0.25
Nodes (7): Hardening notes for operators, Reporting a vulnerability, Scope and safe harbor, Security Policy, Supported versions, What happens next, What to include

### Community 66 - "ISC2 CC Domain 3.1 Physical Access Controls"
Cohesion: 0.25
Nodes (7): Exam Tips, ISC2 CC Domain 3.1 Physical Access Controls, Learning Objectives, Practice Questions, Putting It All Together, Summary, Why Physical Access Controls Matter

### Community 67 - "Authorized vs Unauthorized Personnel"
Cohesion: 0.25
Nodes (8): Authorized vs Unauthorized Personnel, Challenge Procedures, Exam tip, Real-world example, Real-world example, Tailgating and Piggybacking, Two-Person Rule and Dual Control, Visitor Management

### Community 68 - "Mandatory Access Control (MAC)"
Cohesion: 0.25
Nodes (8): Bell-LaPadula, Biba, Clearance Levels, Core Idea, Exam tip, Mandatory Access Control (MAC), Real-world example, Strengths and Weaknesses

### Community 69 - "Denial of Service Attacks"
Cohesion: 0.25
Nodes (8): Application attacks, Botnets, DDoS categories, Denial of Service Attacks, DoS scenario, DoS vs DDoS, Protocol attacks, Volumetric attacks

### Community 70 - "Malware Types"
Cohesion: 0.25
Nodes (8): Malware comparison table, Malware Types, Ransomware, Rootkit, Spyware, Trojan, Virus, Worm

### Community 71 - "Cloud Security Fundamentals"
Cohesion: 0.25
Nodes (8): Cloud model comparison, Cloud Security Fundamentals, IaaS, PaaS, SaaS, Service models, Shared responsibility model, Simple responsibility examples

### Community 72 - "5.2 System Hardening"
Cohesion: 0.25
Nodes (8): 12. Hardening and Monitoring Together, 2. Hardening Principles, 5.2 System Hardening, Exam-friendly logic, Exam Tips, Navigation, Practice Questions, What Hardening Means

### Community 73 - "zagros-skill — cybersecurity skill + Zagros MCP wiring"
Cohesion: 0.25
Nodes (7): Layout, Path 1 — URL install (modern one-liners), Path 2 — agent prompt install, Provenance, Publishing a release, Self-host Zagros (what the default MCP URL expects), zagros-skill — cybersecurity skill + Zagros MCP wiring

### Community 74 - "Configuration"
Cohesion: 0.25
Nodes (8): Allowed hosts, Automated setup (`scripts/setup.ps1`), Configuration, Docker MCP Toolkit, Dockerfile, Manual registration, Scripted registration (`docker/register.ps1`), Server registration files

### Community 75 - "Locks"
Cohesion: 0.29
Nodes (7): Biometric Locks, Cipher Locks, Electronic Locks, Fail-Safe vs Fail-Secure, Locks, Mechanical Locks, Real-world example

### Community 76 - "Account Management Lifecycle"
Cohesion: 0.29
Nodes (7): Account Management Lifecycle, Deprovision, Exam tip, Modify, Provision, Real-world example, Why It Matters

### Community 77 - "Discretionary Access Control (DAC)"
Cohesion: 0.29
Nodes (7): ACLs, Core Idea, Discretionary Access Control (DAC), Exam tip, Real-world example, Strengths and Weaknesses, Windows NTFS and Unix rwx

### Community 78 - "3. Asymmetric Encryption"
Cohesion: 0.29
Nodes (7): 3. Asymmetric Encryption, Common Asymmetric Algorithms, Diffie-Hellman, DSA, ECC, Exam comparison table, RSA

### Community 79 - "5. Hashing"
Cohesion: 0.29
Nodes (7): 5. Hashing, Common Hash Algorithms, Hashing example, Important distinction, Key properties of hashes, MD5 and SHA-1, SHA-256 and SHA-3

### Community 80 - "6. Change Management Policy"
Cohesion: 0.29
Nodes (7): 6. Change Management Policy, CAB, Change categories, Emergency changes, Example, Standard change request elements, Why it exists

### Community 81 - "Security Review — Zagros v0.2"
Cohesion: 0.29
Nodes (5): Executive Summary, Phase 2 Roadmap Items That Address Security Gaps, Security Review — Zagros v0.2, Summary Table, What Is Working Well

### Community 82 - "Findings"
Cohesion: 0.29
Nodes (7): F-01 — HelixDB bound to all interfaces with no authentication, F-02 — CVE ingestion deserializes external JSON without URL validation or size cap, F-03 — `upsert_document` is not atomic, F-04 — `sync_cves` MCP tool has no rate limit, F-05 — `load_all` performs a full table scan on every search, F-06 — BM25 minimum score threshold not set, Findings

### Community 83 - "Learning Path"
Cohesion: 0.29
Nodes (7): Learning Path, Phase 1 — Foundations (COMPLETE), Phase 2 — Structured Ingestion and Provenance (CURRENT), Phase 3 — HelixDB Graph + Vectors, Phase 4 — LLM Reasoning Layer, Phase 5 — Security Scanner Integration, Phase 6 — Evaluation and Confidence Calibration

### Community 85 - "tsconfig.json"
Cohesion: 0.33
Nodes (5): astro/client, astro/tsconfigs/strict, compilerOptions, types, extends

### Community 86 - "The ISC2 Code of Ethics"
Cohesion: 0.33
Nodes (6): Canon 1: Protect Society (HIGHEST PRIORITY), Canon 2: Act Honorably and Legally, Canon 3: Provide Competent Service, Canon 4: Advance the Profession (LOWEST PRIORITY), The Four Canons (MEMORIZE in this order), The ISC2 Code of Ethics

### Community 87 - "Ethical Scenarios (Exam-Style)"
Cohesion: 0.33
Nodes (6): Ethical Scenarios (Exam-Style), Scenario 1: Employer Breaking the Law, Scenario 2: Colleague Cover-Up, Scenario 3: Unauthorized Access for "Testing", Scenario 4: Vulnerability Outside Your Scope, Scenario 5: Employer vs Public Interest

### Community 88 - "Evidence Handling and Chain of Custody"
Cohesion: 0.33
Nodes (6): Chain of custody table, Evidence Handling and Chain of Custody, Evidence handling goals, Good evidence practices, What chain of custody means, Why this matters

### Community 89 - "Real-World Breach Examples"
Cohesion: 0.33
Nodes (6): Colonial Pipeline, Equifax, Real-World Breach Examples, SolarWinds, Target, WannaCry / NHS

### Community 90 - "Communication During Incidents"
Cohesion: 0.33
Nodes (6): Communication During Incidents, Communication matrix, Communication rules, Example, External communication, Internal communication

### Community 91 - "Gate Entry Controls"
Cohesion: 0.33
Nodes (6): Bollards, Exam tip, Gate Entry Controls, Mantraps and Vestibules, Sally Ports, Turnstiles

### Community 92 - "CCTV"
Cohesion: 0.33
Nodes (6): Camera types, CCTV, Exam tip, Placement, Real-world example, Recording vs monitoring

### Community 93 - "AI in Access Controls"
Cohesion: 0.33
Nodes (6): AI in Access Controls, Behavioral Analytics, Exam tip, Impossible Travel Detection, Real-world example, Service Account Management

### Community 94 - "Principle of Least Privilege"
Cohesion: 0.33
Nodes (6): Applies to Humans and Machines, Exam tip, Just-in-Time (JIT) Access, Need-to-Know, Principle of Least Privilege, Why It Matters

### Community 95 - "Separation of Duties"
Cohesion: 0.33
Nodes (6): Collusion, Common Examples, Job Rotation and Mandatory Vacations, Purpose, Real-world example, Separation of Duties

### Community 96 - "Role-Based Access Control (RBAC)"
Cohesion: 0.33
Nodes (6): Common Enterprise Use, Core Idea, Exam tip, Real-world example, Role-Based Access Control (RBAC), Why It Works Well

### Community 97 - "Prevention Technologies"
Cohesion: 0.33
Nodes (6): Antivirus and EDR, Firewall comparison, Firewall types explained, Network segmentation, Prevention Technologies, Real-world prevention example

### Community 98 - "VPNs"
Cohesion: 0.33
Nodes (6): IPSec, IPSec vs SSL VPN, Remote-access VPN, Site-to-site VPN, SSL VPN, VPNs

### Community 99 - "12. Logging and SIEM"
Cohesion: 0.33
Nodes (6): 12. Logging and SIEM, Log protection, SIEM, SIEM workflow, What should be logged, Why logs matter

### Community 100 - "7. PKI and Certificates"
Cohesion: 0.33
Nodes (6): 7. PKI and Certificates, Certificate contents, Certificate lifecycle, CRL vs OCSP, Main PKI components, Real-world analogy

### Community 101 - "3. Password Policy"
Cohesion: 0.33
Nodes (6): 3. Password Policy, Common policy mistakes, Exam-friendly rule, Good password policy elements, Modern password guidance, Passphrases

### Community 102 - "5. BYOD Policy"
Cohesion: 0.33
Nodes (6): 5. BYOD Policy, BYOD alternatives, Common BYOD controls, Example, Which model is safer?, Why BYOD is risky

### Community 103 - "8. AI Workspace Security"
Cohesion: 0.33
Nodes (6): 8. AI Workspace Security, Approved vs unapproved tools, Example, Good AI workspace practices, The problem, Why this matters

### Community 104 - "Zagros MCP appendix (concrete workflow)"
Cohesion: 0.33
Nodes (5): Connection, Rules (mirror `SKILL.md`, Zagros-bound), Self-host pointers, Tool workflow, Zagros MCP appendix (concrete workflow)

### Community 105 - "BM25 retrieval engine"
Cohesion: 0.33
Nodes (6): BM25 retrieval engine, Bonus scoring, Filtering and tie-breaking, In-process document cache (MCP server only), Scoring formula, Tokenization

### Community 106 - "HelixDB"
Cohesion: 0.33
Nodes (6): Checking health, Connecting from inside a Docker container, Docker Compose configuration (`docker/compose.yml`), HelixDB, Starting HelixDB, Stopping HelixDB

### Community 109 - "Risk Treatment Options"
Cohesion: 0.40
Nodes (5): 1. Risk Avoidance, 2. Risk Mitigation (Reduction), 3. Risk Transfer (Sharing), 4. Risk Acceptance, Risk Treatment Options

### Community 110 - "Risk Management Lifecycle"
Cohesion: 0.40
Nodes (5): Phase 1: Risk Identification, Phase 2: Risk Assessment, Phase 3: Risk Treatment, Phase 4: Risk Monitoring, Risk Management Lifecycle

### Community 111 - "AI in Incident Response"
Cohesion: 0.40
Nodes (5): AI in Incident Response, Benefits of AI and SOAR, Example, Risks of AI in IR, SOAR and automation

### Community 112 - "Incident Classification"
Cohesion: 0.40
Nodes (5): Classification questions, Common classification criteria, Incident Classification, Prioritization example, Severity examples

### Community 113 - "Notification Requirements"
Cohesion: 0.40
Nodes (5): Common notification triggers, Exam focus, Example, Notification Requirements, Why timing matters

### Community 114 - "Detection Technologies"
Cohesion: 0.40
Nodes (5): Detection methods, Detection Technologies, IDS vs IPS, NIDS vs HIDS, SIEM

### Community 115 - "Deployment models"
Cohesion: 0.40
Nodes (5): Deployment models, Hybrid cloud, Multi-cloud, Private cloud, Public cloud

### Community 116 - "13. Hardening Across the Lifecycle"
Cohesion: 0.40
Nodes (5): 13. Hardening Across the Lifecycle, After deployment, Before deployment, During deployment, Why this matters

### Community 117 - "4. Patch Management"
Cohesion: 0.40
Nodes (5): 4. Patch Management, Patch lifecycle, Patch strategy example, Patch types, Why testing matters

### Community 118 - "6. Operating System Hardening"
Cohesion: 0.40
Nodes (5): 6. Operating System Hardening, Common OS hardening actions, Linux-specific examples, OS hardening analogy, Windows-specific examples

### Community 119 - "Environment variables"
Cohesion: 0.40
Nodes (5): Environment variables, Running the UI, Running the UI with Docker Compose, Setting `HELIX_URL`, Setting `ZAGROS_DATA_DIR`

### Community 120 - "Current State (v0.2)"
Cohesion: 0.40
Nodes (5): Current State (v0.2), Knowledge corpus in HelixDB (verified 2026-08-16), Verified live against this repository, What is NOT yet implemented, What is working right now

### Community 121 - "Risk Identification Methods"
Cohesion: 0.50
Nodes (4): Penetration Testing, Risk Identification Methods, Threat Modeling, Vulnerability Assessment

### Community 122 - "Risk Assessment: Qualitative vs Quantitative"
Cohesion: 0.50
Nodes (4): Qualitative Risk Assessment, Quantitative Risk Assessment, Risk Assessment: Qualitative vs Quantitative, When to Use Each

### Community 123 - "Biometrics"
Cohesion: 0.50
Nodes (4): Biometrics, Exam tip, Key Metrics, Real-world example

### Community 124 - "Access Control Fundamentals"
Cohesion: 0.50
Nodes (4): Access Control Fundamentals, Exam tip, IAAA, Subject -> Object -> Action Model

### Community 125 - "MFA Rules"
Cohesion: 0.50
Nodes (4): Authentication Factors, Exam tip, MFA Rules, Real-world example

### Community 126 - "Phishing Variants"
Cohesion: 0.50
Nodes (4): Examples, Phishing types, Phishing Variants, Why phishing works

### Community 127 - "Supply Chain Attacks"
Cohesion: 0.50
Nodes (4): Examples, Supply Chain Attacks, Supply chain defense, Why supply chain attacks are powerful

### Community 128 - "AI-Enhanced Network Security"
Cohesion: 0.50
Nodes (4): AI-Enhanced Network Security, Example use case, Important caution, Where AI helps

### Community 129 - "SLA and Uptime"
Cohesion: 0.50
Nodes (4): How to reason about SLA questions, SLA and Uptime, Uptime calculations, Why uptime matters

### Community 131 - "11. Data Destruction"
Cohesion: 0.50
Nodes (4): 11. Data Destruction, Choosing the right destruction method, Important exam rule, Methods of destruction

### Community 132 - "4. Hybrid Encryption and TLS"
Cohesion: 0.50
Nodes (4): 4. Hybrid Encryption and TLS, How Hybrid Encryption Works, TLS handshake analogy, Why hybrid design matters

### Community 133 - "6. Digital Signatures"
Cohesion: 0.50
Nodes (4): 6. Digital Signatures, Example, Step-by-step process, What signatures do and do not do

### Community 134 - "8. Key Management"
Cohesion: 0.50
Nodes (4): 8. Key Management, Common controls, Example, Key management lifecycle

### Community 135 - "9. Data Classification"
Cohesion: 0.50
Nodes (4): 9. Data Classification, Commercial classification, Government classification, Labeling

### Community 136 - "11. Secure Configuration Practices"
Cohesion: 0.50
Nodes (4): 11. Secure Configuration Practices, Configuration drift, Drift example, Why drift is dangerous

### Community 137 - "14. Verification and Validation"
Cohesion: 0.50
Nodes (4): 14. Verification and Validation, Validation methods, Verification methods, Why both matter

### Community 138 - "1. Configuration Management"
Cohesion: 0.50
Nodes (4): 1. Configuration Management, CIS Benchmarks and DISA STIGs, Core terms, Why baselines matter

### Community 139 - "7. Application Hardening"
Cohesion: 0.50
Nodes (4): 7. Application Hardening, Application hardening steps, Example, Why it matters

### Community 140 - "8. Network Hardening"
Cohesion: 0.50
Nodes (4): 8. Network Hardening, Common network controls, Network hardening example, Typical insecure vs secure choices

### Community 141 - "13. Policy Communication"
Cohesion: 0.50
Nodes (4): 13. Policy Communication, Example, Good communication practices, Why this matters

### Community 142 - "2. Data Handling Policy"
Cohesion: 0.50
Nodes (4): 2. Data Handling Policy, Analogy, Classification and handling example, Main elements of a data handling policy

### Community 143 - "4. Acceptable Use Policy"
Cohesion: 0.50
Nodes (4): 4. Acceptable Use Policy, AUP usually covers, Example, Why signed acknowledgment matters

### Community 144 - "7. Privacy Policy"
Cohesion: 0.50
Nodes (4): 7. Privacy Policy, Common privacy principles, Example, Privacy policy topics

### Community 145 - "1. Social Engineering Basics"
Cohesion: 0.50
Nodes (4): 1. Social Engineering Basics, Core influence principles, Example, Key insight

### Community 146 - "3. Password Protection Awareness"
Cohesion: 0.50
Nodes (4): 3. Password Protection Awareness, Bad behavior, Good user behavior, Simple analogy

### Community 147 - "4. Clean Desk and Clean Screen"
Cohesion: 0.50
Nodes (4): 4. Clean Desk and Clean Screen, Clean desk expectations, Example, Why it matters

### Community 148 - "5. Reporting Culture"
Cohesion: 0.50
Nodes (4): 5. Reporting Culture, Best practice, What should be reported, Why culture matters

### Community 149 - "6. Training Program Design"
Cohesion: 0.50
Nodes (4): 6. Training Program Design, Delivery methods, Program design elements, Role-based training

### Community 150 - "7. Measuring Effectiveness"
Cohesion: 0.50
Nodes (4): 7. Measuring Effectiveness, Example, Good target behavior, Useful metrics

### Community 151 - "INSTALL-PROMPT — paste this to your agent (any harness)"
Cohesion: 0.50
Nodes (3): Fast path (preferred), INSTALL-PROMPT — paste this to your agent (any harness), Manual fallback (if running piped scripts is blocked)

### Community 154 - "MCP server internals"
Cohesion: 0.50
Nodes (4): CVE ID validation, MCP server internals, Rate limiting, Tool schemas

### Community 155 - "Success Criteria"
Cohesion: 0.50
Nodes (4): Full vision, Phase 2 (current target), Phase 3 (next milestone), Success Criteria

### Community 156 - "1. Preparation"
Cohesion: 0.67
Nodes (3): 1. Preparation, Activities, Example

### Community 157 - "5. Recovery"
Cohesion: 0.67
Nodes (3): 5. Recovery, Activities, Example

### Community 158 - "6. Lessons learned"
Cohesion: 0.67
Nodes (3): 6. Lessons learned, Activities, Why it matters

### Community 159 - "Threat Actors"
Cohesion: 0.67
Nodes (3): How to think about threat actors, Threat actor table, Threat Actors

### Community 160 - "Side-Channel Attacks"
Cohesion: 0.67
Nodes (3): Side-Channel Attacks, Side-channel types, Why this matters

### Community 161 - "10. Change Management and Hardening"
Cohesion: 0.67
Nodes (3): 10. Change Management and Hardening, Relationship to change management, Why this matters

### Community 162 - "3. Hardening Steps"
Cohesion: 0.67
Nodes (3): 3. Hardening Steps, Real-world example, Why these steps work

### Community 163 - "5. CVSS and Risk Prioritization"
Cohesion: 0.67
Nodes (3): 5. CVSS and Risk Prioritization, Example, Important note

### Community 164 - "9. Database Hardening"
Cohesion: 0.67
Nodes (3): 9. Database Hardening, Database hardening actions, Example

### Community 165 - "10. Policy Enforcement"
Cohesion: 0.67
Nodes (3): 10. Policy Enforcement, Enforcement methods, Why enforcement matters

### Community 166 - "11. Policy Lifecycle and Maintenance"
Cohesion: 0.67
Nodes (3): 11. Policy Lifecycle and Maintenance, Maintenance triggers, Why this matters

### Community 167 - "12. Writing Better Policies"
Cohesion: 0.67
Nodes (3): 12. Writing Better Policies, Better policy language examples, Key idea

### Community 168 - "1. Policy in the Security Program"
Cohesion: 0.67
Nodes (3): 1. Policy in the Security Program, Policy hierarchy, Why the exam cares

### Community 169 - "11. Security Champions and Peer Influence"
Cohesion: 0.67
Nodes (3): 11. Security Champions and Peer Influence, Example, Why they work

### Community 170 - "12. Incident Response for User-Reported Events"
Cohesion: 0.67
Nodes (3): 12. Incident Response for User-Reported Events, Example, Simple response model

### Community 171 - "13. Microlearning and Repetition"
Cohesion: 0.67
Nodes (3): 13. Microlearning and Repetition, Common microlearning topics, Why this matters

### Community 172 - "9. Compliance Training Requirements"
Cohesion: 0.67
Nodes (3): 9. Compliance Training Requirements, Common examples, Why compliance matters

### Community 173 - "Architecture"
Cohesion: 0.67
Nodes (3): Architecture, Current layer diagram, Target layer diagram (full vision)

### Community 174 - "MCP Tools"
Cohesion: 0.67
Nodes (3): Currently exposed (CVE only), MCP Tools, Target (Phase 3+)

### Community 175 - "Source Catalog"
Cohesion: 0.67
Nodes (3): Source Catalog, Tier 1 — Authoritative foundations, Tier 2 — Planned

## Knowledge Gaps
- **1051 isolated node(s):** `zagros`, `DOCKER_MCP_IN_CONTAINER`, `ZAGROS_DATA_DIR`, `zagros-skill`, `install.sh script` (+1046 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **12 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `5.4 Security Awareness Training` connect `5.4 Security Awareness Training` to `5.1-Data-Security.md`, `8. AI Workspace Security`, `11. Security Champions and Peer Influence`, `12. Incident Response for User-Reported Events`, `13. Microlearning and Repetition`, `2. Common Attack Types`, `9. Compliance Training Requirements`, `1. Social Engineering Basics`, `3. Password Protection Awareness`, `4. Clean Desk and Clean Screen`, `5. Reporting Culture`, `6. Training Program Design`, `7. Measuring Effectiveness`?**
  _High betweenness centrality (0.009) - this node is a cross-community bridge._
- **Why does `5.2 System Hardening` connect `5.2 System Hardening` to `10. Change Management and Hardening`, `5.1-Data-Security.md`, `3. Hardening Steps`, `5. CVSS and Risk Prioritization`, `9. Database Hardening`, `11. Secure Configuration Practices`, `14. Verification and Validation`, `1. Configuration Management`, `7. Application Hardening`, `8. Network Hardening`, `13. Hardening Across the Lifecycle`, `4. Patch Management`, `6. Operating System Hardening`?**
  _High betweenness centrality (0.008) - this node is a cross-community bridge._
- **Why does `5.3 Security Policies` connect `5.3 Security Policies` to `5.1-Data-Security.md`, `10. Policy Enforcement`, `11. Policy Lifecycle and Maintenance`, `12. Writing Better Policies`, `1. Policy in the Security Program`, `3. Password Policy`, `5. BYOD Policy`, `13. Policy Communication`, `2. Data Handling Policy`, `4. Acceptable Use Policy`, `6. Change Management Policy`, `7. Privacy Policy`?**
  _High betweenness centrality (0.006) - this node is a cross-community bridge._
- **Are the 18 inferred relationships involving `client()` (e.g. with `.get_cve()` and `get_docs()`) actually correct?**
  _`client()` has 18 INFERRED edges - model-reasoned connections that need verification._
- **What connects `zagros`, `DOCKER_MCP_IN_CONTAINER`, `ZAGROS_DATA_DIR` to the rest of the system?**
  _1051 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `cve-rag/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11688311688311688 - nodes in this community are weakly interconnected._
- **Should `Three-binary Rust project` be split into smaller, more focused modules?**
  _Cohesion score 0.06155632984901278 - nodes in this community are weakly interconnected._