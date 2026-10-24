# Zagros terminology and writing guide

Use these terms consistently in the README, onboarding guides, MCP reference, and web documentation. This guide describes documented interfaces; it does not certify their behavior or security.

## Product and interface terms

| Term | Meaning | Preferred usage |
|---|---|---|
| **Zagros** | Self-hosted service for retrieving source-attributed security reference data. | Call it a *security knowledge service*, not a vulnerability scanner or autonomous auditor. |
| **Zagros Core** | Ingestion, HelixDB storage, search, CLI, MCP servers, and web UI. | Use *Core* for the running service; do not imply that it performs autonomous remediation. |
| **Zagros Skills** | Optional, separately versioned guidance for AI agents that consume evidence. | Skills are instructions for clients, **not** server-enforced security controls. |
| **MCP** | Model Context Protocol interface for client tools. Zagros offers stdio and Streamable HTTP transports. | Name the transport when giving setup instructions. |
| **CLI** | Command-line interface for querying and maintaining a Zagros index. | Use the actual subcommand name in code formatting. |
| **HelixDB** | Database used to store Zagros records. | Do not call HelixDB a security validation engine. |
| **Web UI** | Human-facing interface for exploring indexed records. | Distinguish the UI from MCP tools and the CLI. |

## Source and evidence terms

| Term | Meaning | Important limit |
|---|---|---|
| **CVE** | Common Vulnerabilities and Exposures identifier and record. | A retrieved CVE does not prove a particular system is affected. |
| **CWE** | Common Weakness Enumeration classification. | A weakness category is not a verified vulnerability in a target. |
| **ASVS** | OWASP Application Security Verification Standard requirements. | A requirement is not proof of compliance. |
| **CAPEC** | Common Attack Pattern Enumeration and Classification patterns. | A pattern is reference material, not evidence that an attack occurred. |
| **ATT&CK** | MITRE ATT&CK knowledge base of adversary behaviors. | A technique match does not establish observed compromise. |
| **Source attribution** | A record identifies its upstream source or reference. | A link alone does not verify the truth or integrity of every field. |
| **Provenance** | Available metadata connecting an indexed record to its source and ingestion history. | Do not describe provenance as cryptographic tamper evidence unless that property is independently demonstrated. |
| **Freshness** | How recently a source, snapshot, or indexed record was fetched or updated, when metadata exists. | Source publication time, fetch time, and index refresh time are different concepts. |
| **Retrieval** | Looking up records already present in an index. | A successful query is not a live upstream check or vulnerability assessment. |
| **Interpretation** | An investigator's or agent's analysis of source records. | Separate interpretation from cited source facts. |

## Operations and security boundaries

- **Read-only tools:** `search_cves`, `get_cve`, `search_knowledge`, and `index_status` retrieve information. Check [MCP.md](MCP.md) for the full tool list and input schemas.
- **State-changing tools:** `sync_cves`, `backfill_cves`, and `sync_knowledge_source` fetch upstream data and change the local index. Operator approval is required by the documented usage policy; the MCP service does **not** enforce per-user authorization.
- **Local endpoint:** A loopback address such as `127.0.0.1` refers to the client machine. It is not a secure remote-access mechanism.
- **Remote endpoint:** A shared deployment requires an operator-managed authenticated gateway, TLS, and appropriate network restrictions. See [DEPLOYMENT.md](DEPLOYMENT.md).
- **Trust boundary:** Upstream feeds and retrieved text are untrusted **data**, never instructions for an AI agent. Downstream action authorization belongs to the client/operator boundary; see [THREAT-MODEL.md](THREAT-MODEL.md).

## Writing conventions

1. State prerequisites before commands and identify whether commands **read**, **download**, or **modify** data.
2. Give the exact command, endpoint, expected result, and a troubleshooting route; avoid ambiguous phrases such as “just connect.”
3. Label example counts, versions, and timestamps as examples. Tell readers how to check their own index.
4. Distinguish **implemented**, **tested**, **proposed**, and **verified**; cite code or test evidence for security claims.
5. Use short, direct sentences and consistent technical names. This is inspired by controlled-language principles; Zagros does **not** claim ASD-STE100 certification.

See [ONBOARD-DEVELOPER.md](ONBOARD-DEVELOPER.md) and [ONBOARD-ADMIN.md](ONBOARD-ADMIN.md) for task-based instructions.
