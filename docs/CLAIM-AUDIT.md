# Documentation claim audit — issue #27

Audit date: 2026-10-11. Scope: public README, docs landing page, deployment/testing guides, and first-query demo. Classification refers to repository evidence, not an assertion that every deployment behaves identically.

| Public claim | Classification | Evidence / limitation | Disposition |
| --- | --- | --- | --- |
| Zagros is self-hosted and shares a security corpus via CLI, MCP and web UI | Implemented, partly runtime-verified | `docker/compose.yml`, `src/main.rs`, `src/mcp.rs`, `src/bin/zagros-ui.rs`; connected MCP queries passed 2026-10-11 | Retain as product capability, not security certification |
| CVE data originates in CVEProject delta feeds | Implemented, source-claim not independently revalidated live | `src/sources.rs`, `docs/DATA-SOURCES.md`; review upstream authority before making freshness claims | Retain with provenance qualification |
| CWE, ASVS, CAPEC and ATT&CK ingestion available | Implemented, runtime-verified on connected instance | `src/sources.rs`, `src/mcp.rs`; `index_status` returned counts for all four 2026-10-11 | Retain; counts vary by deployment and snapshot |
| Persistent HelixDB-backed Cve/Knowledge storage | Implemented, runtime-verified through MCP queries | `src/db.rs`, `src/lib.rs`; `index_status` and exact lookup returned data | Retain |
| Seven MCP tools over stdio and HTTP | Implemented, transports not both tested end-to-end in this audit | `src/mcp.rs`, `src/bin/zagros-mcp.rs`, `src/bin/zagros-mcp-http.rs`; `docs/MCP.md` | State implementation; #32 verifies client setup |
| Ranked search returns source links | Runtime-verified on connected instance | `search_cves('remote code execution')`, `search_knowledge('SQL injection')` returned ranked results and URLs 2026-10-11; `src/mcp.rs` | Retain; search results are untrusted data |
| CVE exact lookup works | Runtime-verified for indexed item | `get_cve('CVE-2026-62819')` succeeded; `get_cve('CVE-2024-3094')` failed because not indexed | State conditional on record present; flag demo #32 |
| Fixed corpus counts (CWE 969, ASVS 345, CAPEC 613, ATT&CK 697) | Historical snapshot only | README identifies 2026-09-08 seed baseline. Connected instance 2026-10-11: CVE 5604, CWE 969, ASVS 345, CAPEC 556, ATT&CK 697 | Never advertise as live guarantees; replace unqualified example counts |
| Freshness/health are guaranteed | Not verified | `index_status` returned newest_update 2026-09-11; no successful live refresh demonstrated | Do not claim always current; document operator responsibility |
| MCP write operations require approval | Client-policy requirement, not server-side authorization guarantee | `src/mcp.rs`, `docs/MCP.md`, `docs/THREAT-MODEL.md` | Clarify host/client enforcement; never promise server enforces human approval |
| Automatic remediation or compliance certification | Not offered | `docs/README.md`, `docs/DEMO.md` | Keep explicit non-goals |
| Clean new-user onboarding is reproducible | Unverified | `docs/DEPLOYMENT.md`, `docs/TESTING.md`; end-to-end seed fixture skipped on 2026-10-11 (requires HelixDB) | Gate on #30/#32 |

## Verification record

- Local git revision examined: `a4592a1a6d1a56a6cfc20609915b4735d5b3295e`.
- `python skills/validate.py` passed (Skill Contract v1; 12 evals).
- `cargo fmt --all -- --check` passed.
- `cargo test --test retrieval_benchmark` passed (1 test).
- `cargo test --test e2e_seed` compiled, but fixture ignored without HelixDB.
- MCP `index_status`: CVE=5604, knowledge=2567 (CWE=969, ASVS=345, CAPEC=556, ATT&CK=697); newest_update=2026-09-11.
- MCP `search_knowledge('SQL injection')`, `search_cves('remote code execution')`, `get_cve('CVE-2026-62819')` succeeded.
- MCP `get_cve('CVE-2024-3094')` failed (not in local index), despite that ID being used by `docs/DEMO.md`.
- CI and both clean onboarding personas are NOT established by these results; see #30 and #32.

## Reviewer decision

Issue #27's claim inventory and source-traceability are prepared, and snapshot qualifications must be kept visible. Explicit reviewer confirmation is still required for acceptance. Do not conflate this audit with complete release readiness.
