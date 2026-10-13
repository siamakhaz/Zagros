# Deployment

Zagros is designed to run on infrastructure controlled by the organization or team using it. The supported public-release path is Docker Compose. A single-user local deployment is supported, but the primary shared-service model is one controlled Zagros instance serving the same security reference to multiple developers, AI agents, IDE integrations, and internal tools.

## Local deployment

Copy the example environment file if customization is needed:

```bash
cp .env.example .env
docker compose -f docker/compose.yml up -d --build
```

Default bindings are loopback-only:

- HelixDB: `127.0.0.1:47474`
- MCP HTTP: `127.0.0.1:8789`
- UI: `127.0.0.1:8788`

Verify readiness:

```bash
curl -fsS http://127.0.0.1:8789/health
curl -fsS http://127.0.0.1:8788/api/status
```

The MCP health endpoint verifies that HelixDB is reachable, so it acts as the primary stack readiness check.
## Shared team / cloud deployment

Do **not** expose HelixDB directly. Keep port 47474 loopback-only/internal.

For team, remote, or cloud MCP/UI access, place Zagros behind a TLS reverse proxy or authenticated internal gateway. Give clients one stable internal endpoint (for example, `https://zagros.example.com/mcp`) so multiple agents query the same corpus. An optional Traefik override is provided:

```bash
docker compose -f docker/compose.yml -f docker/compose.traefik.yml up -d --build
```

Configure the `TRAEFIK_*` variables from `.env.example`.

The base Compose stack deliberately has no dependency on Traefik, so operators may use Caddy, Nginx, HAProxy, a cloud load balancer, or another ingress instead.

## Authentication boundary

Zagros MCP currently does not implement application-level user authentication. `MCP_ALLOWED_HOSTS` is a Host-header defense, **not authentication**.

For any MCP endpoint reachable outside a trusted local network, terminate access at an authenticated reverse proxy or access gateway. Recommended patterns are:

- OIDC/OAuth2 access proxy tied to your identity provider;
- mutual TLS for machine-to-machine agent access;
- private VPN/Zero-Trust network with identity-aware policy.

Never publish port 8789 directly to the public internet without an authentication layer. Keep HelixDB private in all deployment modes.
## Freshness

The persistent MCP container runs a non-root cron-compatible scheduler.

Default schedule: `03:00 UTC` daily.

Each run updates the current CVE delta and refreshes all configured knowledge sources. Operational history is written to:

- `/data/refresh-status.json`
- `/data/refresh-history.jsonl`

Set `ZAGROS_AUTO_REFRESH=false` to disable scheduled refresh or change `ZAGROS_REFRESH_CRON` to use another UTC schedule.

## Recovery

Zagros is a derived index of authoritative upstream sources. Database backup is not required.

If HelixDB data is lost:

```bash
docker compose -f docker/compose.yml down
# remove/recreate the data directory if necessary
docker compose -f docker/compose.yml up -d --build
docker compose -f docker/compose.yml --profile cli run --rm cli backfill --limit 1000
docker compose -f docker/compose.yml --profile cli run --rm cli source all
```

The local index is then reconstructed from authoritative sources.
## Resource limits

Default ceilings:

| Service | RAM | CPU | PIDs |
|---|---:|---:|---:|
| HelixDB | 512 MiB | 1.0 | 256 |
| MCP + scheduler/ingestion | 512 MiB | 1.0 | 256 |
| UI | 128 MiB | 0.5 | 128 |
| CLI | 512 MiB | 1.0 | 256 |

If ingestion is killed by the runtime because a future corpus grows, increase the relevant limit deliberately rather than removing limits entirely.

## Platform support

The same Compose workflow is supported on Linux, Windows with Docker Desktop/Engine, macOS with Docker Desktop, and WSL2 with a working Docker integration.

For production/cloud deployments, Linux is recommended because container networking, service management, logging, and reverse-proxy integration are the most predictable there.

## Upgrade

For a source checkout deployment:

```bash
git pull
docker compose -f docker/compose.yml build --pull
docker compose -f docker/compose.yml up -d
```

Schema/data migrations should be documented per release if Zagros later introduces them. The current index is rebuildable from upstream sources, so a failed upgrade can recover by recreating the index and reseeding.

## Uninstall

Stop and remove Zagros containers:

```bash
docker compose -f docker/compose.yml down
```

To also remove the local derived index, remove the configured `ZAGROS_DATA_DIR` directory and the `zagros-data` volume if present. No authoritative source data is lost because the index can be rebuilt from upstream.

If the optional Traefik override was used, remove Zagros from that deployment with the same pair of Compose files.
