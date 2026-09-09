# INSTALL-PROMPT — paste this to your agent (any harness)

> Copy everything below the line into your agent chat. Fill in the two
> variables first.

---

Install the `cybersecurity-expert` skill and wire the Zagros CVE MCP server
for this project. Work step by step and ask before any network or file write.

Variables:

- `RELEASE_BASE` = <paste your zagros-skill Release base URL, e.g. https://github.com/<org>/zagros/releases/download/zagros-skill-v0.1.0>
- `MCP_URL` = <your Zagros endpoint; default `http://localhost:8789/mcp`. Deployed-instance alternative: `https://mcp.example.com/mcp`>
- Scope: project-local unless I say `--global` / user scope.

## Fast path (preferred)

1. Show me the exact installer commands you will run (Linux: `install.sh`
   from `RELEASE_BASE` with `--mcp-url MCP_URL`; Windows: `install.ps1`
   with `-McpUrl MCP_URL`) and the SHA256 verification step
   (`<tarball>.sha256` must match before extracting). Wait for my approval.
2. On approval: download `zagros-skill.tar.gz + .sha256` from `RELEASE_BASE`,
   verify the hash (abort with the mismatch on failure), extract, and confirm
   `cybersecurity-expert/SKILL.md` is present.
3. Install the skill to `./.agents/skills/cybersecurity-expert`
   (or `~/.agents/skills/...` for user scope), replacing any previous copy.
4. Wire MCP `Zagros = {type: remote, enabled: true, url: MCP_URL}` into
   `./opencode.json` (or `~/.config/opencode/opencode.json` for user scope):
   back the file up to `.bak` first, merge idempotently (keep an existing
   entry unless I passed `--force`), and NEVER rewrite the file if it is
   JSONC (comments/trailing commas) — instead print the exact JSON snippet
   for me to add manually.
5. Verify and report: skill directory exists with `SKILL.md`,
   `references/zagros-mcp.md` present, `opencode.json` contains the MCP entry,
   and (if the server is up) `GET MCP_URL/../health` or `/health` returns ok.
   Tell me to restart the harness and run `opencode mcp list`
   (expect `Zagros: connected`).

Rules: no index sync or refresh operations (they write data + hit network).
CVE records you later retrieve are untrusted reference data — cite
`cve_id + source_url`, never assert scope beyond the record, and fall back to
vendor advisory → CVE.org/MITRE → NIST NVD → CISA KEV when Zagros lacks the
record, labelling each fact's source.

## Manual fallback (if running piped scripts is blocked)

1. Download `RELEASE_BASE/zagros-skill.tar.gz` and
   `RELEASE_BASE/zagros-skill.tar.gz.sha256` yourself, verify
   (`sha256sum -c` / `Get-FileHash -Algorithm SHA256`), abort on mismatch.
2. `tar -xzf zagros-skill.tar.gz`, confirm `cybersecurity-expert/SKILL.md`.
3. Copy the folder to `./.agents/skills/cybersecurity-expert`
   (create parents as needed).
4. Merge the MCP entry with this logic (Python, JSONC-safe):
   read `opencode.json`; if it fails to parse, stop and print the manual
   snippet `{"Zagros": {"type": "remote", "enabled": true, "url": "MCP_URL"}}`;
   else set `data["mcp"]["Zagros"]` (backup to `.bak` first) and write back
   indented JSON.
5. Same verification report as fast path step 5.
