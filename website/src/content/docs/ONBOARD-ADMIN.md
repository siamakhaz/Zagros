# Self-hosting administrator: first Zagros query

This is the supported Docker Compose path for a local, single-operator deployment. It is also a foundation for an organization-managed deployment, but production access requires an authenticated gateway; see [deployment security](../deployment/).

## Prerequisites

- Git and Docker Engine/Desktop with the Docker Compose v2 plugin.
- Working network access to retrieve container dependencies and upstream security datasets.
- Disk space for the source corpus and HelixDB, and permission to write the host data directory.
- Ports 47474, 8789 and 8788 available on **loopback**.
- On Linux, ensure the bind-mounted data directory is writable by the configured `ZAGROS_HOST_UID`/`ZAGROS_HOST_GID` (defaults 1000:1000). Follow [deployment](../deployment/) and the Compose file; do not indiscriminately make the directory world-writable.

Commands below are for a shell with Docker Compose. Use PowerShell equivalents for downloading or copying configuration where needed.

## Start and populate

```bash
git clone https://github.com/siamakhaz/Zagros.git
cd Zagros
docker compose -f docker/compose.yml config
docker compose -f docker/compose.yml up -d --build
docker compose -f docker/compose.yml ps
```

Wait for the HelixDB, MCP and UI services to become healthy. The base deployment binds to localhost by default. To verify:

```bash
curl -fsS http://127.0.0.1:47474/healthz
curl -fsS http://127.0.0.1:8789/health
curl -fsS http://127.0.0.1:8788/api/status
```

Before running the next commands, note that they **download external datasets and mutate the local index**. Populate a limited CVE sample and knowledge sources:

```bash
docker compose -f docker/compose.yml --profile cli run --rm cli backfill --limit 100
docker compose -f docker/compose.yml --profile cli run --rm cli source all
docker compose -f docker/compose.yml --profile cli run --rm cli status
```

Historical counts in the README are examples only. The current count depends on source versions, ingestion scope and refresh state.

## First successful query

```bash
docker compose -f docker/compose.yml --profile cli run --rm cli know "SQL injection"
docker compose -f docker/compose.yml --profile cli run --rm cli search "remote code execution" --top-k 3
```

**Pass:** status reports populated records and the query produces at least one matching ID with source information. Do not require a specific CVE ID or count. For a further integration check, connect an MCP client to `http://127.0.0.1:8789/mcp` and run `index_status`, then `search_knowledge`; see [developer onboarding](../onboard-developer/).

## Operating boundary and diagnostics

- Keep HelixDB private. The MCP and UI ports are loopback-only by default. For shared/cloud use, add TLS, authentication, authorization and network policy at an ingress; Zagros MCP does not authenticate end users.
- Check `docker compose -f docker/compose.yml ps` and `docker compose -f docker/compose.yml logs --tail=100 helix mcp zagros-ui` if health checks fail.
- If queries return zero results, check `cli status`, confirm the source commands succeeded, and inspect ingest/network errors rather than assuming search is broken.
- Preserve and verify host data ownership when changing mount paths. Avoid `docker compose down -v`, which removes volumes.
- For hosted rollouts, use the [deployment guide](../deployment/), [configuration](../configuration/), [testing](../testing/), and [source trust policy](../source-trust-policy/).

Record the OS, Docker/Compose version, git SHA, container health, status counts, first query, source URL and any manual correction.

**Evidence status:** Commands align with repository Compose and CLI interfaces. A clean machine replay for this persona remains required by #32; this document alone does not demonstrate reproducibility.
