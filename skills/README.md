# zagros-skill â€” cybersecurity skill + Zagros MCP wiring

Pairs the `cybersecurity-expert` skill with your own [Zagros](../README.md)
security knowledge MCP server. The skill content is independently authored from
the maintainer's personal cybersecurity study notes prepared while studying for
the ISC2 Certified in Cybersecurity (CC) exam; it is not ISC2 courseware,
official exam content, or an ISC2-endorsed derivative work.

- **No PII, no phone-home defaults.** The installer targets your self-hosted
  Zagros at `http://localhost:8789/mcp` unless you pass `--mcp-url` / `-McpUrl`
  (or `ZAGROS_MCP_URL`).
- **Two install paths:** paste `INSTALL-PROMPT.md` to any agent harness, or run
  the one-liner URL installers below (Linux/macOS/WSL + Windows).
- **Releases carry the binaries.** `dist/` is git-ignored; `build-dist.py`
  builds `zagros-skill.tar.gz + .sha256` for a `zagros-skill-v<version>`
  GitHub Release. Installers fetch from your Release base URL.

## Layout

| Path | What it is |
|---|---|
| `.apm/skills/cybersecurity-expert/` | Skill source: `SKILL.md`, `references/` (domains 1â€“5 + `zagros-mcp.md`), `evals/` |
| `apm.yml` / `skill-meta.yml` | APM package metadata (no personal data; MCP default = localhost) |
| `install.sh` / `install.ps1` | Standalone installers: fetch tarball, verify SHA256, install skill, wire MCP into `opencode.json` |
| `INSTALL-PROMPT.md` | Copy-paste prompt for agent-harness installs (script path + manual fallback) |
| `build-dist.py` | Builds the versioned distributable from `apm.yml` |

## Path 1 â€” URL install (modern one-liners)

Replace `<release>` with your published Release base, e.g.
`https://github.com/<org>/zagros/releases/download/zagros-skill-v0.1.0`.

**Linux / macOS / WSL / git-bash** (defaults to localhost MCP):

```bash
curl -fsSL <release>/install.sh | bash
# user scope:
curl -fsSL <release>/install.sh | bash -s -- --global
# your deployed instance instead of localhost:
curl -fsSL <release>/install.sh | bash -s -- --mcp-url https://mcp.example.com/mcp
```

**Windows PowerShell** (run in the project directory):

```powershell
irm <release>/install.ps1 | iex
# user scope:
irm <release>/install.ps1 | iex  # then re-run with -UserScope after download-verify (see file header)
# deployed instance:
$env:ZAGROS_MCP_URL = "https://mcp.example.com/mcp"; irm <release>/install.ps1 | iex
```

Safer variant (download, verify hash, then run) is documented in each script's
header. Every run backs up `opencode.json â†’ opencode.json.bak`, refuses to
clobber JSONC configs, and is idempotent (re-run safe; `--force` / `-Force` to
overwrite the MCP entry).

Then: restart your harness and verify with `opencode mcp list`
(expect `Zagros: connected`), plus `GET http://localhost:8789/health â†’ ok`.

## Path 2 â€” agent prompt install

Copy the whole of `INSTALL-PROMPT.md` into any agent harness (opencode,
copilot, generic). It instructs the agent to do the fast script path with hash
verification and approval gates, with a manual download â†’ verify â†’ extract â†’
JSON-merge fallback when piping scripts is not allowed.

## Self-host Zagros (what the default MCP URL expects)

```powershell
docker compose -f docker/compose.yml up -d helix
.\target\debug\zagros.exe backfill --limit 500
.\target\debug\zagros.exe source all
cargo run --bin zagros-mcp-http   # serves POST /mcp on 0.0.0.0:8789
curl http://localhost:8789/health
```

LAN access: add your host IP to `MCP_ALLOWED_HOSTS`. Full reference:
`docs/MCP.md`.

## Contract, compatibility, and harnesses

- [Skill Contract v1](../docs/SKILL-CONTRACT.md) defines the portable package and behavior contract.
- [Compatibility](COMPATIBILITY.md) defines independent skill SemVer, Core compatibility, upgrade, uninstall, and APM expectations.
- [Harness examples](HARNESSES.md) cover OpenCode, Copilot-compatible environments, Claude-compatible clients, and generic MCP clients.
- [Skill trust policy](../docs/SKILL-TRUST-POLICY.md) defines review requirements for community-contributed skills.

Validate the package before release:

```bash
python skills/validate.py
```

The validator checks required files and metadata, Contract v1, SemVer, eval structure/unique IDs, and that declared required MCP tools are documented by Zagros Core.

## Publishing a release

```bash
cd skills
python build-dist.py            # reads version from apm.yml â†’ dist/
# attach dist/zagros-skill.tar.gz + .sha256 with install.sh / install.ps1
# to GitHub Release zagros-skill-v<version>, then update the BASE_URL
# defaults in both installers to that Release base.
```

`ZAGROS_MCP_URL` at build time is recorded in `META.json` for provenance only;
the install-time `--mcp-url` always wins.

## Provenance

Cybersecurity guidance in this package is independently authored from the
maintainer's personal study notes and practical experience. Studying for the
ISC2 Certified in Cybersecurity (CC) exam influenced the topic coverage, but no
ISC2 courseware or official exam content is included. Zagros and this skill are
not affiliated with or endorsed by ISC2. Zagros-authored skill content is
licensed under Apache-2.0; upstream standards and trademarks remain subject to
their respective owners' terms. The generic rules in `SKILL.md` stay
harness-neutral; `references/zagros-mcp.md` binds them to Zagros.
