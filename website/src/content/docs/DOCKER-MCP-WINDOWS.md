# Docker MCP Gateway on Windows (Without Docker Desktop)

This guide covers two paths for running the Docker MCP Gateway on Windows
without Docker Desktop:

- **Path A — WSL2 + Docker CE** (recommended): Run Docker Engine inside WSL2.
  The gateway runs natively in Linux; Windows clients (opencode, VS Code, Claude)
  connect to it over stdio or HTTP.
- **Path B — Docker Engine on Windows Server**: Run the Windows native Docker
  daemon. Suitable for server environments. Supports Windows containers only —
  Linux-based MCP server images require WSL2 or a remote daemon.

---

## Path A — WSL2 + Docker CE (Recommended for Workstations)

This is the closest equivalent to Docker Desktop without the licensing
requirements. Everything runs inside a WSL2 Ubuntu instance.

### Step 1 — Install WSL2

Open PowerShell as Administrator:

```powershell
wsl --install
```

This installs WSL2 and Ubuntu by default. Restart when prompted.

Verify WSL2 is active after reboot:

```powershell
wsl --list --verbose
```

You should see `VERSION 2` next to your distribution.

### Step 2 — Install Docker Engine Inside WSL2

Open your Ubuntu WSL2 terminal and run:

```bash
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER
# Exit and re-open the WSL2 terminal so group membership takes effect
```

Start the Docker daemon (WSL2 does not run systemd by default on older Ubuntu):

```bash
# If systemd is available (Ubuntu 22.04+ on WSL2):
sudo systemctl enable --now docker

# If systemd is NOT available (older Ubuntu or custom distros):
sudo service docker start
```

Verify Docker is working:

```bash
docker run --rm hello-world
```

### Step 3 — Install Go

The `docker-mcp` plugin must be built from source.

```bash
# Download and install Go 1.24+
wget https://go.dev/dl/go1.24.0.linux-amd64.tar.gz
sudo rm -rf /usr/local/go
sudo tar -C /usr/local -xzf go1.24.0.linux-amd64.tar.gz

# Add to PATH
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
source ~/.bashrc

go version  # should print go1.24.x
```

### Step 4 — Build and Install the docker-mcp Plugin

```bash
git clone https://github.com/docker/mcp-gateway.git
cd mcp-gateway

mkdir -p "$HOME/.docker/cli-plugins/"
make docker-mcp

docker mcp --help  # verify
```

### Step 5 — Set Required Environment Variables

On WSL2 without Docker Desktop, set the bypass flag to avoid
"Docker Desktop is not running" errors:

```bash
echo 'export DOCKER_MCP_IN_CONTAINER=1' >> ~/.bashrc
source ~/.bashrc
```

### Step 6 — Enable Profiles and Pull the Catalog

```bash
docker mcp feature enable profiles
docker mcp catalog pull mcp/docker-mcp-catalog
```

### Step 7 — Create a Profile and Run the Gateway

```bash
# Create a profile with the servers you want
docker mcp profile create --name my-profile \
  --server catalog://mcp/docker-mcp-catalog/github

# Start the gateway (stdio mode)
docker mcp gateway run --profile my-profile
```

---

## Path B — Docker Engine on Windows Server (Native Windows)

> This path runs the Windows-native Docker daemon. It supports **Windows
> containers only**. Most MCP servers in the Docker catalog are Linux-based
> and require WSL2 (Path A) or a remote Linux daemon.

### Step 1 — Download Docker Engine Binaries

From PowerShell (run as Administrator):

```powershell
# Download the latest stable Docker Engine zip
$version = "27.5.1"
Invoke-WebRequest `
  "https://download.docker.com/win/static/stable/x86_64/docker-$version.zip" `
  -OutFile "$env:TEMP\docker.zip"

# Extract to Program Files
Expand-Archive "$env:TEMP\docker.zip" -DestinationPath $env:ProgramFiles -Force
```

### Step 2 — Register and Start the Docker Service

```powershell
& "$env:ProgramFiles\Docker\dockerd.exe" --register-service
Start-Service docker
Set-Service -Name docker -StartupType Automatic

# Verify
& "$env:ProgramFiles\Docker\docker.exe" version
```

Add Docker to your PATH so `docker` works from any terminal:

```powershell
[Environment]::SetEnvironmentVariable(
  "Path",
  "$env:Path;$env:ProgramFiles\Docker",
  [EnvironmentVariableTarget]::Machine
)
# Restart your terminal after this
```

### Step 3 — Install Go and Build docker-mcp

Install Go for Windows from https://go.dev/dl/ (download the `.msi` installer).

Then from PowerShell:

```powershell
git clone https://github.com/docker/mcp-gateway.git
cd mcp-gateway

New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.docker\cli-plugins"

# Build for Windows
$env:GOOS = "windows"
$env:GOARCH = "amd64"
go build -o "$env:USERPROFILE\.docker\cli-plugins\docker-mcp.exe" ./cmd/docker-mcp

docker mcp --help
```

### Step 4 — Set Required Environment Variable

In PowerShell (current session):

```powershell
$env:DOCKER_MCP_IN_CONTAINER = "1"
```

To persist it permanently:

```powershell
[Environment]::SetEnvironmentVariable(
  "DOCKER_MCP_IN_CONTAINER", "1",
  [EnvironmentVariableTarget]::User
)
```

### Step 5 — Enable Profiles and Pull the Catalog

```powershell
docker mcp feature enable profiles
docker mcp catalog pull mcp/docker-mcp-catalog
```

### Step 6 — Create a Profile and Run the Gateway

```powershell
docker mcp profile create --name my-profile `
  --server catalog://mcp/docker-mcp-catalog/github

docker mcp gateway run --profile my-profile
```

---

## Connecting to opencode on Windows

Regardless of which path you used, configure opencode at
`%USERPROFILE%\.config\opencode\config.json`:

### If using WSL2 (Path A)

opencode runs on Windows but the gateway runs inside WSL2. Use the `wsl`
command to bridge them:

```json
{
  "mcp": {
    "docker": {
      "type": "local",
      "command": "wsl",
      "args": [
        "--",
        "bash", "-c",
        "DOCKER_MCP_IN_CONTAINER=1 docker mcp gateway run --profile my-profile"
      ]
    }
  }
}
```

### If using Native Windows (Path B)

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

---

## Connecting to Other Clients on Windows

```powershell
# VS Code
docker mcp client connect vscode --profile my-profile

# Claude Desktop
docker mcp client connect claude --profile my-profile

# Cursor
docker mcp client connect cursor --profile my-profile --global
```

> When using WSL2, run these commands inside the WSL2 terminal, not from
> PowerShell directly.

---

## Zagros MCP Server on Windows

The `zagros-mcp` server in this project uses Docker MCP. On Windows without
Docker Desktop, follow these steps instead of `register.ps1`.

### WSL2 Path

Inside your WSL2 terminal, from the project root:

```bash
# Build the image
docker build -t zagros-mcp:0.1.0 .

# Create a local server definition
cat > docker/server-wsl.yaml <<'EOF'
name: zagros
description: Zagros search and retrieval tools
image: zagros-mcp:0.1.0
environment:
  HELIX_URL: http://host.docker.internal:47474
allowHosts:
  - raw.githubusercontent.com:443
volumes:
  - zagros-data:/data
EOF

# Create a profile using the local server definition
docker mcp profile create --name zagros-profile \
  --server file://./docker/server-wsl.yaml

# Run the gateway
DOCKER_MCP_IN_CONTAINER=1 docker mcp gateway run --profile zagros-profile
```

### Native Windows Path

```powershell
# Build the image
docker build -t zagros-mcp:0.1.0 .

# Register via file reference
docker mcp profile create --name zagros-profile `
  --server file://./docker/server.yaml

docker mcp gateway run --profile zagros-profile
```

---

## Troubleshooting

### `Docker Desktop is not running`

```
Error: Docker Desktop is not running
```

**Fix:** Set `DOCKER_MCP_IN_CONTAINER=1`. In WSL2:

```bash
export DOCKER_MCP_IN_CONTAINER=1
```

In PowerShell:

```powershell
$env:DOCKER_MCP_IN_CONTAINER = "1"
```

---

### `docker mcp` command not found

The plugin binary must be in `~/.docker/cli-plugins/` (Linux/WSL2) or
`%USERPROFILE%\.docker\cli-plugins\` (Windows).

Verify:

```bash
# WSL2
ls ~/.docker/cli-plugins/docker-mcp

# PowerShell
ls "$env:USERPROFILE\.docker\cli-plugins\docker-mcp.exe"
```

If missing, rebuild following Step 4 of the relevant path above.

---

### WSL2 Docker daemon is not running

```
Cannot connect to the Docker daemon at unix:///var/run/docker.sock
```

Start the daemon:

```bash
# With systemd
sudo systemctl start docker

# Without systemd
sudo service docker start
```

To start Docker automatically when you open WSL2, add this to `~/.bashrc`:

```bash
if ! pgrep -x "dockerd" > /dev/null; then
  sudo service docker start
fi
```

---

### Container cannot reach HelixDB on the host

Inside Docker, `localhost` refers to the container itself. Use
`host.docker.internal` to reach the Windows/WSL2 host:

```yaml
environment:
  HELIX_URL: http://host.docker.internal:47474
```

On some WSL2 Docker CE setups, `host.docker.internal` is not resolved
automatically. Add `--add-host` when running containers manually:

```bash
docker run --add-host=host.docker.internal:host-gateway ...
```

---

### Linux MCP server images fail on Windows Server (Path B)

Windows-native Docker Engine runs Windows containers only. Linux-based MCP
server images from the catalog will not start.

**Fix:** Use WSL2 (Path A) for Linux container support on Windows workstations.

---

### opencode cannot find the WSL2 gateway

If opencode is running on Windows and cannot spawn the WSL2 gateway subprocess,
make sure:

1. WSL2 is installed and `wsl` is on the Windows `PATH`.
2. The WSL2 distro name matches what `wsl --list` returns.
3. Try specifying the distro explicitly:

```json
"args": ["-d", "Ubuntu", "--", "bash", "-c", "DOCKER_MCP_IN_CONTAINER=1 docker mcp gateway run --profile my-profile"]
```

---

## Comparison: WSL2 vs Native Windows

| | WSL2 + Docker CE | Native Windows Docker |
|---|---|---|
| Linux container support | Yes | No |
| Docker MCP Catalog servers | Full catalog | Very limited |
| Setup complexity | Medium | Medium |
| Requires Windows restart | Yes (WSL2 install) | Yes (service install) |
| Works on Windows 10/11 | Yes | Yes |
| Works on Windows Server | Limited | Yes |
| Recommended for MCP use | **Yes** | No (Linux images only) |

---

## Quick Reference

| Command | Context |
|---|---|
| `wsl --install` | PowerShell (Admin) — one-time WSL2 setup |
| `curl -fsSL https://get.docker.com \| sudo sh` | WSL2 bash — install Docker CE |
| `export DOCKER_MCP_IN_CONTAINER=1` | WSL2 bash — bypass Desktop check |
| `docker mcp feature enable profiles` | First-time setup |
| `docker mcp catalog pull mcp/docker-mcp-catalog` | Fetch default catalog |
| `docker mcp profile create --name X --server ...` | Create a profile |
| `docker mcp gateway run --profile X` | Start the gateway |
| `docker mcp tools ls` | List available tools |
| `docker mcp client connect vscode --profile X` | Connect VS Code |

---

## See Also

- [DOCKER-MCP-LINUX.md](DOCKER-MCP-LINUX.md) — Linux-native setup (no WSL2)
- [MCP.md](MCP.md) — Zagros MCP server tool reference
- [CONFIGURATION.md](CONFIGURATION.md) — HelixDB and environment setup
- [docker/mcp-gateway on GitHub](https://github.com/docker/mcp-gateway) — upstream source
