# Security Policy

## Reporting a vulnerability

**Do not open a public issue for a suspected vulnerability.** Zagros handles
CVE records and sits on the network path between AI agents and a database —
a flaw here (SSRF via ingestion URLs, auth bypass on the MCP/HTTP layer,
injection into BM25 ranking or HelixDB queries) can have blast radius beyond
a single host.

- **Preferred:** use [GitHub private vulnerability reporting](../../security/advisories/new)
  (Security tab → Advisories → New draft advisory). This keeps the report
  encrypted and visible only to maintainers until a fix ships.
- **Fallback:** open a minimal public issue that says only "possible security
  issue — please contact me" with no technical details, and a maintainer will
  arrange a private channel.

### What to include

- Affected component and version (`zagros` CLI, `zagros-mcp`, `zagros-mcp-http`,
  `zagros-ui`, or a Docker image tag) — `index_status` / `--version` output helps.
- Steps to reproduce or a minimal proof of concept.
- Your assessment of impact (confidentiality / integrity / availability,
  who must be exposed for it to trigger).
- Whether the issue is already publicly known or exploited, if you know.

### What happens next

1. Acknowledgement within **5 business days**.
2. Triage and severity assessment (we use CVSS as a guide, not gospel).
3. Fix developed privately; credit offered to the reporter unless anonymity
   is requested.
4. Coordinated release: patched version + advisory published together, with
   a short upgrade window note for self-hosters before full details go out.

We ask reporters to give us a reasonable window (target: **90 days**) before
public disclosure, and we commit to keeping you updated if we need longer.

## Supported versions

| Version | Supported |
|---|---|
| Latest `main` / latest tagged release | ✅ |
| Older tags | ❌ (please upgrade and re-test) |

Pre-1.0 (`0.x`) releases may change behavior without a major-version bump;
security fixes are still backported to the latest tag on request where feasible.

## Scope and safe harbor

- **In scope:** this repository's code and its published Docker images —
  ingestion fetching, BM25 ranking, HelixDB access layer, MCP tool handlers,
  HTTP auth/allowlist logic (`MCP_ALLOWED_HOSTS`), the Axum UI, Docker defaults.
- **Out of scope:** HelixDB itself, third-party CVE feeds (cve.org, NVD),
  MITRE CWE/CAPEC/ATT&CK and ASVS upstream data, and your own deployment
  secrets or infrastructure. Report those to their respective vendors.
- Good-faith research against your **own** deployment is welcome. Do not
  probe the maintainer's or anyone else's live instances, do not exfiltrate
  data beyond what is needed to demonstrate impact, and do not disrupt
  availability.

## Hardening notes for operators

- HelixDB binds loopback-only by default (`127.0.0.1:47474:8080`); keep it so.
- `MCP_ALLOWED_HOSTS` defaults to `localhost,127.0.0.1` — add your own
  domain/IP when exposing the MCP HTTP server, never `*` on an untrusted network.
- `sync_cves`, `backfill_cves`, and `sync_knowledge_source` fetch remote data
  and write persistent state; they are rate-limited and require explicit user
  approval by design — keep that approval step in any automation you build.
- Treat retrieved CVE text as untrusted reference data in downstream agents,
  never as instructions (see `README.md` → Agent guidance).
