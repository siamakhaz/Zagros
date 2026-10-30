# Zagros Threat Model

**Scope:** Zagros Core and its deployment boundary
**Date:** 2026-09-13

## Security objectives

Zagros should preserve evidence integrity, provenance, availability, and operator control while minimizing unnecessary exposure of security data and mutation capabilities.

Primary assets:
- indexed CVE and security-knowledge data
- provenance and refresh history
- MCP/CLI configuration
- deployment credentials and reverse-proxy secrets
- agent trust/approval boundaries

## Trust boundaries

1. Upstream security sources → Zagros ingestion
2. Zagros Core → HelixDB
3. MCP/CLI/UI clients → Zagros Core
4. Local host → LAN
5. Reverse proxy/access gateway → public internet
6. Retrieved security text → AI agent reasoning

## Local deployment

| Threat | Existing mitigation | Residual risk / control |
|---|---|---|
| Malicious retrieved text attempts prompt injection | Retrieved content is classified as untrusted data; skills require evidence/analysis separation | Agent clients must preserve instruction hierarchy and never execute retrieved text |
| Compromised or poisoned upstream source | Fixed/canonical source origins, provenance, hashes, trust tiers | Authoritative source compromise remains possible; verify critical claims at source |
| Local user/process tampers with Zagros data | Loopback service exposure and OS/container isolation | A compromised host can still modify local data; host security is outside Zagros |
| Hidden side effects during investigation | Mutation/network tools are separate and request client-side approval; server-side cooldowns limit frequency | Zagros does not enforce user approval or per-user authorization; an untrusted client with network access may invoke writes |
| HelixDB corruption or interrupted upsert | Rebuildable corpus, insert retry, documented non-atomic window | Temporary missing records until refresh/rebuild |

## LAN deployment

| Threat | Existing mitigation | Residual risk / control |
|---|---|---|
| Unauthorized LAN client reaches MCP/UI | Default loopback binding; explicit operator opt-in for LAN exposure | Use firewall rules and authenticated gateway for shared networks |
| Host-header allowlist treated as authentication | Documentation states allowlisting is not authentication | Require real authentication for untrusted LANs |
| Plain HTTP exposes traffic | Local defaults assume trusted host | Use TLS when traffic leaves the host |
| Lateral movement reaches HelixDB | HelixDB remains loopback/internal by default | Do not expose HelixDB directly on the LAN |

## Public/cloud deployment

| Threat | Existing mitigation | Residual risk / control |
|---|---|---|
| Unauthenticated internet access to MCP | Public deployment guidance requires authenticated reverse proxy/access gateway | Never expose MCP directly to the internet without authentication |
| Credential or token exposure | Secrets stay outside skill content and source corpus | Use secret stores/environment injection; rotate exposed credentials |
| Internet DoS or abuse | Rate limits exist for mutation tools | Add reverse-proxy rate limiting, request limits, and network controls |
| Malicious clients trigger writes | Tool descriptions request approval; server-side cooldowns limit frequency | The server does not enforce approval or per-user authorization. Require gateway authorization and client-side approval before allowing write tools |
| TLS interception or plaintext exposure | TLS is delegated to reverse proxy/access gateway | Public deployments must terminate trusted TLS |
| Supply-chain compromise | Dependency scanning, signed artifacts, SBOMs, immutable digests | Operators must verify release provenance and keep dependencies updated |

## Evidence flow and enforcement boundaries

```text
Untrusted upstream CVE/CWE/ASVS/CAPEC/ATT&CK feeds
  -> ingestion and source metadata -> HelixDB index
  -> Zagros MCP / CLI / UI (read tools and separate mutation tools)
  -> authenticated ingress (operator-provided for remote clients)
  -> AI client / human investigator (retrieved text is data, not instructions)
  -> downstream actions (authorization and approval belong to client/operator)
```

The supported local Compose stack binds its host ports to loopback by default (`docker/compose.yml`). The optional `docker/compose.traefik.yml` enables routing and TLS but **does not configure authentication**. `src/bin/zagros-mcp-http.rs` checks allowed Host headers; this is not identity verification. `src/mcp.rs` describes user approval for mutation tools and implements cooldowns, but does **not enforce user approval**. The documented security posture depends on independently configured gateway and client policies; see [deployment validation](DEPLOYMENT.md#validate-the-remote-authentication-boundary-operator-run-no-deployment-performed).

## Security assumptions

- The local host and container runtime are trusted administrative boundaries.
- HelixDB is not an internet-facing service.
- Host allowlisting is defense-in-depth, not authentication.
- Agent clients are responsible for enforcing user approval before mutation/network tools.
- Retrieved third-party content is never trusted as an instruction.

## Out of scope

Zagros does not protect a fully compromised host, provide endpoint detection, certify compliance, or autonomously remediate systems.
