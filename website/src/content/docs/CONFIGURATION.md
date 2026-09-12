# Configuration

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `HELIX_URL` | `http://localhost:47474` | HelixDB instance URL used by all CLI commands and the MCP server |
| `ZAGROS_DATA_DIR` | `data/` relative to CWD | Directory for the legacy flat-file cache (`cves.json`) |
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

Used only by `save_local_index` and `load_local_index` (legacy flat-file path).
HelixDB is the canonical store; this variable is rarely needed.

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

Build the image and start the container (HelixDB must already be running):

```powershell
docker compose -f docker/compose.ui.yml up -d --build
```

The UI is available at `http://localhost:8788`.

Stop:

```powershell
docker compose -f docker/compose.ui.yml down
```

| File | Purpose |
|---|---|
| `Dockerfile.ui` | Two-stage build that produces the `zagros-ui` binary image |
| `docker/compose.ui.yml` | Compose service for the UI; exposes loopback port `8788` |

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

Data persists in the named volume `zagros_helix-data` across container restarts
and `down` commands. Use `down -v` to also delete the volume.

### Docker Compose configuration (`docker/compose.yml`)

| Setting | Value | Notes |
|---|---|---|
| Image | `ghcr.io/helixdb/helixdb` (pinned SHA256) | Exact digest â€” upgrade by updating the digest |
| Port | `127.0.0.1:47474:8080` | Loopback-only; not exposed to the network |
| Memory limit | `256m` | Sufficient for the current corpus size |
| CPU limit | `1.0` | One vCPU |
| Volume | `zagros_helix-data` | Named, survives container restarts |

### Connecting from inside a Docker container

When the MCP server container needs to reach HelixDB on the host:

```yaml
environment:
  HELIX_URL: http://host.docker.internal:47474
```

`host.docker.internal` resolves to the Docker host on Windows and macOS.

---

## Docker MCP Toolkit

### Manual registration

```powershell
# Create a profile
docker mcp profile create --name profile

# Add the server (uses docker/zagros-server.yaml)
docker mcp profile server add zagros `
  --server file://C:/Projects/RAG/zagros/docker/zagros-server.yaml

# Verify tools are visible
docker mcp tools ls --gateway-arg=--profile --gateway-arg=profile
```

### Scripted registration (`docker/register.ps1`)

The script is idempotent â€” it removes a stale registration before re-adding:

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
| `docker/server.yaml` | MCP server runs inside Docker; HelixDB also in Docker (internal network) |
| `docker/zagros-server.yaml` | MCP server inside Docker; HelixDB on host (`host.docker.internal:47474`) |

### Allowed hosts

The MCP server is permitted to reach only:

- `raw.githubusercontent.com:443` â€” CVE delta feed
- `host.docker.internal:47474` â€” HelixDB on the Docker host (zagros-server.yaml only)

---

## Automated setup (`scripts/setup.ps1`)

The setup script runs all configuration steps in order and is safe to re-run.

```powershell
.\scripts\setup.ps1 [OPTIONS]
```

**Parameters:**

| Parameter | Type | Default | Description |
|---|---|---|---|
| `-Profile` | string | `profile` | Docker MCP profile name |
| `-Seed` | switch | false | Run `backfill` after setup to pre-populate the CVE index |
| `-SeedLimit` | integer | `500` | Number of CVEs to load during seed |
| `-SkipBuild` | switch | false | Skip `cargo build --release` |
| `-SkipImage` | switch | false | Skip `docker build` |

**Steps performed:**

1. **Preflight** â€” verifies `docker`, `cargo`, and `pwsh` â‰¥ 7 are available.
2. **HelixDB** â€” starts HelixDB with Docker Compose; polls until responsive (up to 30 s).
3. **Build** â€” `cargo build --release --bin zagros-mcp`.
4. **Image** â€” `docker build -t zagros-mcp:0.1.0 .`
5. **Register** â€” runs `docker/register.ps1` to create catalog entry and add to profile.
6. **Seed** (if `-Seed`) â€” runs `zagros-mcp backfill --limit $SeedLimit`.

**Example:**

```powershell
# Full first-time setup with 500 CVEs pre-loaded
.\scripts\setup.ps1 -Seed -SeedLimit 500

# Rebuild image only (HelixDB already running, already registered)
.\scripts\setup.ps1 -SkipBuild:$false -SkipImage:$false
```

---

## Dockerfile

The `Dockerfile` produces the `zagros-mcp` image only (the CLI binary is not included).

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

- `/data/refresh-status.json` â€” latest attempt/success status.
- `/data/refresh-history.jsonl` â€” append-only refresh event history.

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
