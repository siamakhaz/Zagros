# Zagros AI Host Bootstrap Prompt

You are installing Zagros on this host for use by AI agents. Complete the setup end-to-end, make it idempotent, verify every stage, and report exactly what changed.

Authoritative repository: https://github.com/siamakhaz/Zagros
Default install location:
- Windows: `$HOME\zagros`
- Linux/macOS: `$HOME/zagros`

## Operating rules

1. Detect the OS, shell, CPU architecture, current user privileges, and whether this is an existing Zagros installation.
2. Do not delete user data, Docker volumes, existing AI-client configuration, or unrelated files.
3. Reuse an existing Zagros clone when valid; otherwise clone the repository from GitHub.
4. Prefer stable package-manager installs for missing prerequisites. Do not install unrelated developer tooling.
5. Back up any AI-client configuration before modifying it.
6. Never expose secrets in output. Zagros itself requires no API key for its default public security sources.
7. If a prerequisite requires an interactive elevation, reboot, logout/login, or Docker Desktop startup, perform everything possible first and then give the user the single blocking action required.
8. Continue automatically through non-destructive installation steps; do not stop merely to ask permission for ordinary file creation, Git clone/pull, Docker builds, or Zagros configuration.

## Target state

When finished, this host must have:

- Git available.
- Docker Engine/Desktop running with Docker Compose v2 available as `docker compose`.
- A Zagros checkout at the default location above, unless the user already chose another location.
- The Zagros Compose stack running:
  - HelixDB on host loopback port 47474.
  - Zagros MCP HTTP server on `http://localhost:8789/mcp`.
  - Zagros UI on `http://localhost:8788`.
- A populated index containing CVE and Tier-1 knowledge data.
- The `cybersecurity-expert` skill installed at user scope:
  - `~/.agents/skills/cybersecurity-expert`
- The current AI harness connected to the Zagros MCP endpoint when the harness supports MCP.
- A successful end-to-end verification.

## Phase 1 - Preflight and prerequisites

Check, in this order:

```
git --version
docker --version
docker compose version
docker info
```

If Git is missing, install it with the platform's normal package manager.

If Docker or Compose v2 is missing:
- Windows 10/11: prefer Docker Desktop through winget when winget is available.
- Linux: prefer the distribution-supported Docker packages or Docker's official repository instructions.
- macOS: prefer Docker Desktop.
- Windows Server: do not assume Linux containers are available natively; use a supported Linux-container environment such as WSL2 when required.

After installing Docker, verify `docker info` succeeds before continuing. If Docker needs a user-visible first launch, reboot, or group-membership refresh, stop only at that blocking point and state the exact action needed.

Rust, Cargo, Go, Node.js, Python, and Docker MCP Gateway are NOT prerequisites for the normal AI installation path.

## Phase 2 - Obtain Zagros

If the target directory is already a Git checkout of `siamakhaz/Zagros`:
- Preserve local changes.
- Run `git status --short`.
- If clean, update with a fast-forward-only pull.
- If dirty, do not overwrite changes; use the existing checkout and report that it was not updated.

Otherwise clone:

```
git clone https://github.com/siamakhaz/Zagros.git <target>
```

Enter the repository root before continuing.

## Phase 3 - Start Zagros

Start the supported local stack:

```
docker compose -f docker/compose.yml up -d --build
```

Wait for the MCP health endpoint to become ready, retrying for a reasonable period:

```
http://localhost:8789/health
```

Also verify the UI endpoint responds:

```
http://localhost:8788/api/status
```

If startup fails, inspect:

```
docker compose -f docker/compose.yml ps
docker compose -f docker/compose.yml logs --tail 100 helix
docker compose -f docker/compose.yml logs --tail 100 mcp
```

Fix only Zagros-related configuration or startup issues; do not weaken host firewall/security settings globally.

## Phase 4 - Seed security knowledge

Populate the local database with the documented baseline:

```
docker compose -f docker/compose.yml --profile cli run --rm cli backfill --limit 500
docker compose -f docker/compose.yml --profile cli run --rm cli source all
docker compose -f docker/compose.yml --profile cli run --rm cli status
```

The commands are idempotent and may be re-run.

Do not claim the database is ready until `status` reports populated CVE and knowledge-source counts.

The MCP service has automatic refresh enabled by default through the Compose configuration. Do not create an additional host cron/scheduled task unless the repository documentation explicitly requires one.

## Phase 5 - Install the Zagros skill

Install the repository skill source from:

```
skills/.apm/skills/cybersecurity-expert
```

to the user-scoped skill directory:

```
~/.agents/skills/cybersecurity-expert
```

On Windows, resolve `~` using the current user's home directory.

If the destination already exists, replace only that skill directory with the repository version. Preserve unrelated skills.

Verify at minimum:

- `SKILL.md` exists.
- `references/zagros-mcp.md` exists.
- The installed skill content matches the current checkout.

## Phase 6 - Connect the current AI harness

Detect the harness you are running inside and configure Zagros using that harness's supported MCP configuration mechanism.

Use this MCP identity:

```
Name: Zagros
Transport: Streamable HTTP / remote HTTP MCP
URL: http://localhost:8789/mcp
```

Rules:
- Prefer user-scope configuration so Zagros is available across projects.
- Preserve all existing MCP servers and settings.
- Back up a config file before editing it.
- Merge idempotently; do not duplicate an existing Zagros entry.
- If a config file is JSONC/TOML/YAML, preserve its format and comments.
- If you cannot safely determine the harness configuration format, do not guess. Verify Zagros itself and print the exact MCP name/URL the user must add manually.

For OpenCode, use its current supported remote-MCP configuration and verify with `opencode mcp list` when the command exists.

## Phase 7 - End-to-end verification

Do all applicable checks:

1. `docker compose -f docker/compose.yml ps` shows the required services running.
2. `GET http://localhost:8789/health` succeeds.
3. `GET http://localhost:8788/api/status` succeeds.
4. CLI status reports populated data:
   `docker compose -f docker/compose.yml --profile cli run --rm cli status`
5. The installed skill contains `SKILL.md` and `references/zagros-mcp.md`.
6. The active AI harness lists or connects to the `Zagros` MCP server, when harness tooling permits verification.
7. If MCP tool invocation is available, call a read-only Zagros tool such as `index_status` or search for a known CVE and confirm a structured response with source URLs.

Do not run destructive Docker cleanup commands such as `docker system prune`, `docker compose down -v`, or volume deletion.

## Final report

Return a compact report with:
- OS and architecture.
- Zagros checkout path and commit SHA.
- Docker/Compose versions.
- Service health.
- Indexed data status/counts.
- Skill installation path.
- AI harness detected and MCP configuration location.
- MCP verification result.
- Any single remaining user action, only if truly blocked by an interactive system requirement.

Success means the agent can immediately use the installed `cybersecurity-expert` skill together with the local Zagros MCP server.
