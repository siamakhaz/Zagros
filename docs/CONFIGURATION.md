# Configuration

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `HELIX_URL` | `http://localhost:47474` for native processes | HelixDB URL; Compose overrides it to `http://helix:8080` for MCP and CLI containers |
| `ZAGROS_DATA_DIR` | `./data` in `.env.example` | Host directory bind-mounted at `/data` for HelixDB by Compose; native CLI also uses it for legacy local files |
| `UI_PORT` | `8788` | Port for the optional web UI binary (`zagros-ui`) |

### Setting `HELIX_URL`

**PowerShell (session):**

```powershell
$env:HELIX_URL = "http://my-helix-host:47474"
```

**PowerShell (permanent for the current user):**

```powershell
[Environment]::SetEnvironmentVariable("HELIX_URL", "http://my-helix-host:47474", "User")
```

**Docker Compose service:**

```yaml
environment:
  HELIX_URL: http://host.docker.internal:47474
```

### Setting `ZAGROS_DATA_DIR`

In the supported Compose stack, this sets the **host directory** bind-mounted into HelixDB at `/data`. Set it in `.env` before starting Compose; ensure it is writable by `ZAGROS_HOST_UID`/`ZAGROS_HOST_GID` (default 1000:1000). In a native CLI process, the same variable also controls the legacy `cves.json` file path. HelixDB remains the canonical store.

```powershell
$env:ZAGROS_DATA_DIR = "C:\data\zagros"
```

### Running the UI

The optional web UI binds to `http://localhost:8788` by default.

```powershell
cargo run --bin zagros-ui
```

To use a different port:

```powershell
$env:UI_PORT = "8790"
cargo run --bin zagros-ui
```

### Running the UI with Docker Compose

The canonical Compose file includes HelixDB, MCP HTTP, and the UI. Start the stack from the repository root:

```powershell
docker compose -f docker/compose.yml up -d --build
docker compose -f docker/compose.yml ps
```

The UI is available at `http://127.0.0.1:8788` by default. To stop the stack without deleting the host index directory:

```powershell
docker compose -f docker/compose.yml down
```

`Dockerfile.ui` builds the `zagros-ui` image.

---

## HelixDB

### Starting HelixDB

```powershell
docker compose -f docker/compose.yml up -d helix
```

HelixDB starts on `http://127.0.0.1:47474` (loopback-only binding).

### Checking health

```powershell
docker compose -f docker/compose.yml ps
curl -sf http://localhost:47474/healthz
```

Compose starts the MCP service after the HelixDB container starts. HelixDB's
`/healthz` endpoint can be checked from the host with the command above. The MCP service exposes `/health` on port 8789;
the UI Compose stack checks `/api/status` on port 8788, which also verifies its
connection to HelixDB.

### Stopping HelixDB

```powershell
docker compose -f docker/compose.yml down
```

HelixDB data persists in the **host bind-mounted directory** selected by `ZAGROS_DATA_DIR` (default `./data` in `.env.example`). `docker compose down` does not delete this directory. Do not use `down -v` as a data cleanup command: it removes Compose-managed named volumes such as `zagros-data`, but does not delete the HelixDB host directory.

### Docker Compose configuration (`docker/compose.yml`)

| Setting | Value | Notes |
|---|---|---|
| Image | `zagros-helix:3.1.1-pinned` | Built from `docker/Dockerfile.helix` at the repository-pinned HelixDB source commit |
| Port | `127.0.0.1:47474:8080` | Loopback-only; not exposed to the network |
| Memory limit | `512m` | Compose resource ceiling, not a capacity guarantee |
| CPU limit | `1.0` | One vCPU |
| Data | `ZAGROS_DATA_DIR` bind-mounted at `/data` | Host-owned directory; writable by the configured UID/GID |

### Connecting from inside a Docker container

In the supported `docker/compose.yml` stack, MCP and CLI containers reach HelixDB through the private Compose network:

```yaml
environment:
  HELIX_URL: http://helix:8080
```

The separate Docker Desktop MCP Toolkit registration uses `http://host.docker.internal:47474` for HelixDB; `docker/server.yaml` also declares a host-gateway mapping. This is **not** the canonical Compose service-to-service address.

---

## Docker MCP Toolkit

### Manual registration

```powershell
# Create a profile
docker mcp profile create --name profile

# Add the server (uses docker/zagros-server.yaml)
docker mcp profile server add zagros `
  --server file://C:/Projects/Zagros/docker/zagros-server.yaml

# Verify tools are visible
docker mcp tools ls --gateway-arg=--profile --gateway-arg=profile
```

### Scripted registration (`docker/register.ps1`)

The script is idempotent — it removes a stale registration before re-adding:

```powershell
.\docker\register.ps1 -Profile profile
```

Parameters:

| Parameter | Default | Description |
|---|---|---|
| `-Profile` | `profile` | Docker MCP profile name |

### Server registration files

Two YAML files are provided:

| File | Use case |
|---|---|
| `docker/server.yaml` | Full Toolkit registration with tool metadata, host-gateway mapping, and host-accessible HelixDB |
| `docker/zagros-server.yaml` | Minimal Toolkit registration; also uses host-accessible HelixDB (`host.docker.internal:47474`) |

### Allowed hosts (Docker MCP Toolkit registration)

The current `allowHosts` entries in `docker/server.yaml` and `docker/zagros-server.yaml` are:

- `raw.githubusercontent.com:443` — CVE delta feed
- `cwe.mitre.org:443` — CWE source
- `capec.mitre.org:443` — CAPEC source
- `host.docker.internal:47474` — host-accessible HelixDB

These are Toolkit registration settings, **not** proof of a complete network egress policy and **not** authentication. Verify actual network controls independently. The Compose HTTP MCP service uses a different network configuration; see [DEPLOYMENT.md](DEPLOYMENT.md).

---

## Automated setup (`scripts/setup.ps1`)

The setup script is an **optional Docker Desktop MCP Toolkit** workflow, not the supported Compose first-query path. For the supported deployment, follow [administrator onboarding](ONBOARD-ADMIN.md). The script builds both the native stdio MCP server and the separate CLI used for seeding.

```powershell
.\scripts\setup.ps1 [OPTIONS]
```

**Parameters:**

| Parameter | Type | Default | Description |
|---|---|---|---|
| `-Profile` | string | `profile` | Docker MCP profile name |
| `-Seed` | switch | false | Run `backfill` after setup to pre-populate the CVE index |
| `-SeedLimit` | integer | `500` | Number of CVEs to load during seed |
| `-SkipBuild` | switch | false | Skip building both `zagros.exe` (CLI) and `zagros-mcp.exe` (stdio MCP); both must already exist |
| `-SkipImage` | switch | false | Skip `docker build` |

**Steps performed:**

1. **Preflight** — verifies `docker`, `cargo`, and `pwsh` ≥ 7 are available.
2. **HelixDB** — starts HelixDB with Docker Compose; polls until responsive (up to 30 s).
3. **Build** — `cargo build --release --locked --bin zagros --bin zagros-mcp` (CLI and stdio MCP binaries).
4. **Image** — `docker build -t zagros-mcp:0.1.0 .`
5. **Register** — runs `docker/register.ps1` to create catalog entry and add to profile.
6. **Seed** (if `-Seed`) — runs `zagros.exe backfill --limit $SeedLimit` against local HelixDB. The stdio MCP binary does not accept CLI subcommands; a failed backfill stops setup.

**Example:**

```powershell
# Full first-time setup with 500 CVEs pre-loaded
.\scripts\setup.ps1 -Seed -SeedLimit 500

# Rebuild image only (HelixDB already running, already registered)
.\scripts\setup.ps1 -SkipBuild:$false -SkipImage:$false
```

---

## Dockerfile

The `Dockerfile` produces the `zagros-mcp:0.1.0` image containing the `zagros` CLI, `zagros-mcp` stdio server, and `zagros-mcp-http` server. Its default entrypoint starts the HTTP server; it does not run CLI subcommands unless explicitly invoked.

**Build:**

```powershell
docker build -t zagros-mcp:0.1.0 .
```

**Runtime environment variables available inside the container:**

| Variable | Default set by image | Description |
|---|---|---|
| `ZAGROS_DATA_DIR` | `/data` | Mounted volume path |
| `HELIX_URL` | not set (use `http://host.docker.internal:47474`) | HelixDB URL |

**Volume:**

The image declares `/data` as a Docker volume. Mount a named volume for persistence:

```yaml
volumes:
  - zagros-data:/data
```

---

## Daily freshness and automatic refresh

The persistent MCP container refreshes Zagros automatically once per day. The default schedule is **03:00 UTC** and runs:

```text
zagros ingest --limit 1000
zagros source all
```

This updates the latest CVE delta and fully refreshes CWE, ASVS, CAPEC, and ATT&CK from their configured authoritative upstream sources.

| Variable | Default | Purpose |
|---|---|---|
| `ZAGROS_AUTO_REFRESH` | `true` | Enable/disable scheduled refresh |
| `ZAGROS_REFRESH_CRON` | `0 3 * * *` | Cron expression, interpreted in UTC |
| `ZAGROS_DAILY_CVE_LIMIT` | `1000` | Maximum changed CVEs processed per daily refresh |
| `ZAGROS_PRESERVE_RAW_SOURCES` | `false` | Optionally retain raw source snapshots under `/data/snapshots`; disabled by default because Zagros is a rebuildable index |

The scheduler uses Supercronic v0.2.49 and runs as the non-root `zagros` user. The binary is version-pinned and SHA-256 verified during image build.

Refresh state is persisted under the MCP data volume:

- `/data/refresh-status.json` — latest attempt/success status.
- `/data/refresh-history.jsonl` — append-only refresh event history.

A failed refresh does not stop the MCP server. The previous indexed corpus remains available and the failure is recorded for investigation.

For deployments that require a different schedule, set `ZAGROS_REFRESH_CRON` in `.env` or the Compose environment.

## Data durability and backup policy

Zagros is designed as a **rebuildable derived index**, not a system of record. Its authoritative data lives upstream at CVE, MITRE, and OWASP sources.

A backup of the Zagros/HelixDB index is therefore **not required for correctness**. If the index is lost, recreate it by starting a fresh HelixDB instance and running the normal seed/synchronization commands.

The only locally generated operational data worth retaining when desired is the refresh history under `/data`. Users may copy that log for audit purposes, but Zagros does not require a database backup workflow.

## Resource limits

The default Compose stack applies explicit resource ceilings so ingestion cannot consume the host without bounds.

| Service | Memory | CPU | PID limit |
|---|---:|---:|---:|
| HelixDB | 512 MiB | 1.0 CPU | 256 |
| MCP + daily ingestion | 512 MiB | 1.0 CPU | 256 |
| UI | 128 MiB | 0.5 CPU | 128 |
| CLI profile | 512 MiB | 1.0 CPU | 256 |

The MCP limit is intentionally higher than the old 128 MiB setting because ATT&CK ingestion can temporarily hold a large STIX payload plus parsed structures in memory.

These are safe defaults for the current corpus, not capacity guarantees. Increase them deliberately if future sources or corpus sizes grow.

## Supported deployment environments

The canonical deployment is Docker Compose and is OS-independent wherever a current Docker Engine/Compose implementation is available.

- **Linux:** supported; preferred for servers/cloud deployments.
- **Windows:** supported with Docker Desktop or Docker Engine workflows documented in this repository.
- **macOS:** supported through Docker Desktop using the same Compose workflow.
- **WSL2:** supported when Docker Desktop integration or a working Docker Engine is available inside WSL.

Native Rust binaries may also run directly, but Docker Compose is the supported public-release path because it keeps HelixDB, resource limits, scheduling, and networking consistent.

## Local vs reverse-proxy deployment

The default stack is local-safe and requires no Traefik network:

```bash
docker compose -f docker/compose.yml up -d --build
```

MCP and UI bind to `127.0.0.1` by default.

For an existing Traefik installation, add the optional override:

```bash
docker compose -f docker/compose.yml -f docker/compose.traefik.yml up -d --build
```

Set the `TRAEFIK_*` values from `.env.example` for your network, domains, entrypoint, and certificate resolver. The base Compose file remains independent of any particular reverse proxy.
