# How Zagros works

Zagros is a self-hosted reference service for security investigators and AI agents. It retrieves source-attributed information; it does **not** inspect a target, prove exploitability, certify compliance or remediate vulnerabilities. The **Core** service works independently of optional [agent Skills](SKILL-CONTRACT.md).

## The evidence flow

```text
CVE Project / MITRE CWE / OWASP ASVS / MITRE CAPEC / MITRE ATT&CK
                            |
                 CLI or approved MCP sync
                            |
        fetch -> parse/validate -> normalize + provenance
                            |
               HelixDB: Cve / Knowledge records
                            |
              load -> rank using in-process BM25
                            |
              CLI      MCP stdio / HTTP      Web UI
                            |
          source-linked records for human investigation
```

1. **Collect:** The CLI's `backfill`, `ingest`, and `source` commands, or the MCP write tools, retrieve upstream datasets. Source metadata and provenance are retained, including canonical/retrieval URLs when supplied. A successful retrieval is not proof that an upstream claim is correct; external text is untrusted.
2. **Store:** Normalized CVE and knowledge records are persisted as typed nodes in HelixDB. The contents and freshness depend on what the operator has successfully indexed. Counts shown in documentation are not live guarantees.
3. **Retrieve:** CLI, MCP and Web UI use shared Rust data-access/retrieval logic. Lexical BM25 ranking and bonuses are implemented by the application, not a HelixDB vector index. The MCP server additionally maintains in-process caches; see [architecture](ARCHITECTURE.md).
4. **Investigate:** Search results include record identifiers and source information for follow-up. The investigator or AI agent must check the original advisory and whether it applies to the actual system.

## Choose an interface

| Interface | Where it runs | Intended use | Relevant reference |
| --- | --- | --- | --- |
| CLI (`zagros`) | Local shell or Docker Compose `cli` profile | Seed, sync, check index status, search CVEs/knowledge | [CLI](CLI.md) |
| MCP stdio (`zagros-mcp`) | Spawned by a compatible agent client | Agent tool calls over local standard input/output | [MCP](MCP.md) |
| MCP Streamable HTTP (`zagros-mcp-http`) | Long-running service, `/mcp` endpoint | Agent tools over HTTP for connected clients | [MCP](MCP.md) |
| Web UI (`zagros-ui`) | HTTP service, normally port 8788 | Human browsing, filtering, record inspection and search | [Interface status](INTERFACE-STATUS.md) |

The two MCP transports expose the **same seven tools**, implemented in `src/mcp.rs`:

| Read-only tools | Network/write tools |
| --- | --- |
| `search_cves`, `get_cve`, `index_status`, `search_knowledge` | `sync_cves`, `backfill_cves`, `sync_knowledge_source` |

**Approval and trust boundary:** The write tools fetch external data and change the index. Clients should seek operator approval before invoking them; guidance inside a Skill does **not** enforce server authorization. The HTTP endpoint has host-header controls but does not provide end-user authentication. For shared use, put MCP and UI behind authenticated, authorized ingress and keep HelixDB private. A health endpoint or successful search is not an attestment of freshness, data completeness, or exposure safety. See [deployment security](DEPLOYMENT.md) and [source-trust policy](SOURCE-TRUST-POLICY.md).

## Example investigation

An agent reviewing possible SQL injection can call `search_knowledge` with `SQL injection`, inspect a CWE result and canonical reference, and present **published evidence** separately from **its inference** about the code under review. A human should confirm relevance; Zagros does not inspect that code on its own.

For an operator's first query, follow [administrator onboarding](ONBOARD-ADMIN.md). For an agent, follow [developer onboarding](ONBOARD-DEVELOPER.md). Fresh-machine onboarding coverage is tracked separately in [issue #32](https://github.com/siamakhaz/Zagros/issues/32).

## Implementation and verification map

| Claim | Repository evidence | Verification level |
| --- | --- | --- |
| Upstream ingestion and provenance | `src/sources.rs`, `src/lib.rs`, `src/provenance.rs` | Code implemented; upstream freshness requires runtime check |
| Typed persistence | `src/db.rs` | Code implemented; E2E smoke exercises fixture storage |
| Ranking and search | `src/lib.rs`, `tests/retrieval_benchmark.rs` | Code implemented and regression-tested |
| Seven MCP tools, both transports | `src/mcp.rs`, `src/bin/zagros-mcp.rs`, `src/bin/zagros-mcp-http.rs` | Tool contracts implemented; E2E tests use MCP |
| CLI and Web UI | `src/main.rs`, `src/bin/zagros-ui.rs` | Code implemented; do not infer a complete fresh-install walkthrough |

See [claim audit](CLAIM-AUDIT.md) for explicit limitations and [testing](TESTING.md) for test scope.
