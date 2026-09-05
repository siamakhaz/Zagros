# Configuration

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `HELIX_URL` | `http://localhost:47474` | HelixDB instance URL used by all CLI commands and the MCP server |
| `CVE_RAG_DATA_DIR` | `data/` relative to CWD | Directory for the legacy flat-file cache (`cves.json`) |
| `UI_PORT` | `8788` | Port for the optional CVE UI binary (`cve-ui`) |

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

### Setting `CVE_RAG_DATA_DIR`

Used only by `save_local_index` and `load_local_index` (legacy flat-file path).
HelixDB is the canonical store; this variable is rarely needed.

```powershell
$env:CVE_RAG_DATA_DIR = "C:\data\cve-rag"
```

### Running the UI

The optional web UI binds to `http://localhost:8788` by default.

```powershell
cargo run --bin cve-ui
```

To use a different port:

```powershell
$env:UI_PORT = "8790"
cargo run --bin cve-ui
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
| `Dockerfile.ui` | Two-stage build that produces the `cve-ui` binary image |
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
```

There is no HTTP healthcheck endpoint (distroless image — no shell or curl).
Wait approximately 3–5 seconds after container start before running ingestion commands.

### Stopping HelixDB

```powershell
docker compose -f docker/compose.yml down
```

Data persists in the named volume `cve-rag_helix-data` across container restarts
and `down` commands. Use `down -v` to also delete the volume.

### Docker Compose configuration (`docker/compose.yml`)

| Setting | Value | Notes |
|---|---|---|
| Image | `ghcr.io/helixdb/helixdb` (pinned SHA256) | Exact digest — upgrade by updating the digest |
| Port | `127.0.0.1:47474:8080` | Loopback-only; not exposed to the network |
| Memory limit | `256m` | Sufficient for the current corpus size |
| CPU limit | `1.0` | One vCPU |
| Volume | `cve-rag_helix-data` | Named, survives container restarts |

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

# Add the server (uses docker/cve-rag-server.yaml)
docker mcp profile server add cve_rag `
  --server file://C:/Projects/Zagros/docker/cve-rag-server.yaml

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
| `docker/server.yaml` | MCP server runs inside Docker; HelixDB also in Docker (internal network) |
| `docker/cve-rag-server.yaml` | MCP server inside Docker; HelixDB on host (`host.docker.internal:47474`) |

### Allowed hosts

The MCP server is permitted to reach only:

- `raw.githubusercontent.com:443` — CVE delta feed
- `host.docker.internal:47474` — HelixDB on the Docker host (cve-rag-server.yaml only)

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

1. **Preflight** — verifies `docker`, `cargo`, and `pwsh` ≥ 7 are available.
2. **HelixDB** — starts HelixDB with Docker Compose; polls until responsive (up to 30 s).
3. **Build** — `cargo build --release --bin cve-rag-mcp`.
4. **Image** — `docker build -t cve-rag-mcp:0.1.0 .`
5. **Register** — runs `docker/register.ps1` to create catalog entry and add to profile.
6. **Seed** (if `-Seed`) — runs `cve-rag-mcp backfill --limit $SeedLimit`.

**Example:**

```powershell
# Full first-time setup with 500 CVEs pre-loaded
.\scripts\setup.ps1 -Seed -SeedLimit 500

# Rebuild image only (HelixDB already running, already registered)
.\scripts\setup.ps1 -SkipBuild:$false -SkipImage:$false
```

---

## Dockerfile

The `Dockerfile` produces the `cve-rag-mcp` image only (the CLI binary is not included).

**Build:**

```powershell
docker build -t cve-rag-mcp:0.1.0 .
```

**Runtime environment variables available inside the container:**

| Variable | Default set by image | Description |
|---|---|---|
| `CVE_RAG_DATA_DIR` | `/data` | Mounted volume path |
| `HELIX_URL` | not set (use `http://host.docker.internal:47474`) | HelixDB URL |

**Volume:**

The image declares `/data` as a Docker volume. Mount a named volume for persistence:

```yaml
volumes:
  - cve-rag-data:/data
```
