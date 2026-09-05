# Docker MCP Gateway on Linux

This guide covers installing and running the Docker MCP Gateway on Linux without
Docker Desktop. The gateway is the same technology that powers the MCP Toolkit in
Docker Desktop, but it runs entirely from the open-source
[`docker/mcp-gateway`](https://github.com/docker/mcp-gateway) CLI plugin.

---

## Prerequisites

| Requirement | Notes |
|---|---|
| Docker Engine (CE) | Installed and running. Not Docker Desktop. |
| Go 1.24+ | Only required if building from source. |
| Git | To clone the repository. |
| `make` | Standard build tooling. |

Install Docker Engine on Ubuntu/Debian:

```bash
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER
# Log out and back in so group membership takes effect
```

---

## Installation

### 1. Clone and Build the CLI Plugin

```bash
git clone https://github.com/docker/mcp-gateway.git
cd mcp-gateway

# Create the Docker CLI plugins directory
mkdir -p "$HOME/.docker/cli-plugins/"

# Build and install
make docker-mcp
```

Verify the installation:

```bash
docker mcp --help
```

### 2. Set the Required Environment Variable

On Linux with Docker CE, the gateway must be told it is not running inside
Docker Desktop. Without this, commands fail with
`"Docker Desktop is not running"` even when the daemon is active.

```bash
export DOCKER_MCP_IN_CONTAINER=1
```

Persist it across sessions:

```bash
echo 'export DOCKER_MCP_IN_CONTAINER=1' >> ~/.bashrc
source ~/.bashrc
```

### 3. Enable Profiles

Profiles are enabled automatically in Docker Desktop. On Linux CE you must
enable the feature manually once:

```bash
docker mcp feature enable profiles
```

---

## Catalog Setup

The Docker MCP Catalog is a curated OCI image of pre-packaged MCP server
definitions hosted on Docker Hub.

### Pull the Default Catalog

```bash
docker mcp catalog pull mcp/docker-mcp-catalog
```

### Browse Available Servers

```bash
docker mcp catalog server ls mcp/docker-mcp-catalog
```

---

## Profile Setup

A **profile** groups one or more MCP servers together. The gateway runs a single
profile at a time and exposes all the tools from its servers to the connected
AI client.

### Create a Profile

```bash
docker mcp profile create --name my-profile \
  --server catalog://mcp/docker-mcp-catalog/github
```

To include multiple servers in one profile:

```bash
docker mcp profile create --name my-profile \
  --server catalog://mcp/docker-mcp-catalog/github \
  --server catalog://mcp/docker-mcp-catalog/grafana \
  --server catalog://mcp/docker-mcp-catalog/atlassian
```

### List Profiles

```bash
docker mcp profile list
```

### Configure a Server in a Profile

Some MCP servers require API keys or tokens. Set them as profile config values:

```bash
docker mcp profile config my-profile --set github.token=ghp_...
docker mcp profile config my-profile --set grafana.url=http://localhost:3000
docker mcp profile config my-profile --set grafana.token=glsa_...
```

### Manage Secrets (Recommended)

For credentials, use the secrets store instead of plain config so values are
not stored in plaintext:

```bash
docker mcp secret set github.token
# Prompts for value interactively
```

---

## Running the Gateway

### stdio Mode (for use with opencode, Claude, VS Code, etc.)

The AI client spawns the gateway as a subprocess. This is the standard mode
for opencode and most MCP clients.

```bash
docker mcp gateway run --profile my-profile
```

### HTTP Streaming Mode (for multi-client or remote use)

```bash
docker mcp gateway run --profile my-profile --port 8080 --transport streaming
```

---

## Connecting to opencode

Add the gateway as an MCP server in your opencode configuration at
`~/.config/opencode/config.json`:

```json
{
  "mcp": {
    "docker": {
      "type": "local",
      "command": "docker",
      "args": ["mcp", "gateway", "run", "--profile", "my-profile"],
      "env": {
        "DOCKER_MCP_IN_CONTAINER": "1"
      }
    }
  }
}
```

> The `DOCKER_MCP_IN_CONTAINER` env var is set inline here as a safety net,
> even if it is already in your shell profile.

### Verify Tools Are Visible

After opening opencode, confirm the tools loaded correctly:

```bash
docker mcp tools ls --profile my-profile
docker mcp tools count
```

---

## Connecting to Other Clients

The gateway also supports Claude Desktop, VS Code, and Cursor.

```bash
# Connect to Claude Desktop
docker mcp client connect claude --profile my-profile

# Connect to VS Code
docker mcp client connect vscode --profile my-profile

# Connect to Cursor
docker mcp client connect cursor --profile my-profile --global
```

---

## Tool Management

### List All Available Tools

```bash
docker mcp tools ls
```

### Inspect a Specific Tool

```bash
docker mcp tools inspect search_cves
```

### Call a Tool Directly (for testing)

```bash
docker mcp tools call search_cves '{"query": "remote code execution", "top_k": 5}'
```

### Restrict Which Tools Are Exposed

```bash
# Enable only specific tools from a server
docker mcp profile tools my-profile --enable github.create_issue --enable github.list_repos

# Disable a tool
docker mcp profile tools my-profile --disable github.search_code

# Disable all tools for a server, then selectively enable
docker mcp profile tools my-profile --disable-all grafana
docker mcp profile tools my-profile --enable grafana.search_dashboards
```

---

## cve-rag MCP Server on Linux

The `cve-rag-mcp` server in this project uses the same Docker MCP mechanism.
To register it manually on Linux instead of using `register.ps1`:

### 1. Build the Image

```bash
docker build -t cve-rag-mcp:0.1.0 .
```

### 2. Create a Local Server Definition

Create `docker/server-linux.yaml`:

```yaml
name: cve-rag
description: CVE RAG search and retrieval tools
image: cve-rag-mcp:0.1.0
environment:
  HELIX_URL: http://host.docker.internal:47474
allowHosts:
  - raw.githubusercontent.com:443
volumes:
  - cve-rag-data:/data
```

### 3. Add to a Profile

```bash
docker mcp profile create --name cve-profile \
  --server file://./docker/server-linux.yaml
```

### 4. Run the Gateway

```bash
DOCKER_MCP_IN_CONTAINER=1 docker mcp gateway run --profile cve-profile
```

---

## Troubleshooting

### `Docker Desktop is not running`

```
Error: Docker Desktop is not running
```

**Fix:** Set `DOCKER_MCP_IN_CONTAINER=1` in your environment.

---

### `permission denied` on `/var/run/docker.sock`

```
permission denied while trying to connect to the Docker daemon socket
```

**Fix:** Add your user to the `docker` group and re-login:

```bash
sudo usermod -aG docker $USER
```

---

### `docker mcp` command not found

The CLI plugin binary must be in `~/.docker/cli-plugins/`. Verify:

```bash
ls -la ~/.docker/cli-plugins/docker-mcp
```

If missing, rebuild:

```bash
cd mcp-gateway && make docker-mcp
```

---

### Container cannot reach HelixDB

Inside a Docker container, `localhost` refers to the container itself, not the
host. Use the Docker-provided host alias instead:

```yaml
environment:
  HELIX_URL: http://host.docker.internal:47474
```

On some Linux Docker Engine versions, `host.docker.internal` is not
automatically available. Add it explicitly when running the container:

```bash
docker run --add-host=host.docker.internal:host-gateway ...
```

---

### Gateway exits immediately with no output

Run with verbose logging to diagnose:

```bash
DOCKER_MCP_IN_CONTAINER=1 docker mcp gateway run --profile my-profile --log-level debug
```

---

## Reference

| Command | Purpose |
|---|---|
| `docker mcp feature enable profiles` | One-time setup on Linux CE |
| `docker mcp catalog pull mcp/docker-mcp-catalog` | Fetch the default catalog |
| `docker mcp catalog server ls mcp/docker-mcp-catalog` | Browse available servers |
| `docker mcp profile create --name X --server ...` | Create a profile |
| `docker mcp profile list` | List all profiles |
| `docker mcp profile config X --set key=value` | Configure a server in a profile |
| `docker mcp secret set key` | Store a credential securely |
| `docker mcp gateway run --profile X` | Start the gateway (stdio) |
| `docker mcp gateway run --profile X --port 8080 --transport streaming` | Start the gateway (HTTP) |
| `docker mcp client connect <client> --profile X` | Connect a client |
| `docker mcp tools ls` | List exposed tools |
| `docker mcp tools call <tool> '<json>'` | Test a tool directly |

---

## See Also

- [MCP.md](MCP.md) — cve-rag MCP server tool reference
- [CONFIGURATION.md](CONFIGURATION.md) — HelixDB and environment setup
- [docker/mcp-gateway on GitHub](https://github.com/docker/mcp-gateway) — upstream source
