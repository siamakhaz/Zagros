# Testing the Deployed Application

This document covers how to verify that a deployed Zagros instance is working
correctly at every layer: HelixDB, the CLI, the MCP gateway, and the AI client.

Run these checks in order — each layer depends on the one below it.

---

## Layer 1 — HelixDB

HelixDB must be healthy before anything else works.

```bash
curl -sf http://localhost:47474/healthz
```

Expected output:

```json
{"index_runtime":"ready","mode":"writer","ready":true}
```

If the container is not running:

```bash
docker ps --filter name=zagros-helix
# restart if needed:
cd /path/to/zagros
ZAGROS_DATA_DIR=/data/zagros/data \
  docker compose -f docker/compose.yml --project-name zagros up -d helix
```

---

## Layer 2 — CLI

The CLI binaries talk to HelixDB directly. No Docker gateway involved.

### Check index status

```bash
./target/release/zagros status
```

Expected output (after seeding):

```
HelixDB URL  : http://localhost:47474
CVE nodes    : 1000
  asvs        : 345
  attack      : 697
  capec        : 556
  cwe         : 969
  total       : 2567
```

If CVE nodes is 0, run the seed commands:

```bash
./target/release/zagros backfill --limit 1000
./target/release/zagros source all
```

### Search CVEs

```bash
./target/release/zagros search "remote code execution" --top-k 3
```

Expected: 3 ranked CVE records with IDs, scores, and descriptions.

```bash
./target/release/zagros search "CVE-2026-62819"
```

Expected: exact match for that CVE ID ranked first.

### Search knowledge base

```bash
./target/release/zagros know "SQL injection"
```

Expected: ranked results from CWE / ASVS / CAPEC / ATT&CK.

---

## Layer 3 — MCP gateway

Tests the docker-mcp gateway and the `zagros-mcp` container in isolation,
without an AI client.

### Check gateway starts and lists tools

```bash
export DOCKER_MCP_IN_CONTAINER=1
docker mcp gateway run --profile profile --verbose --dry-run 2>&1
```

Expected output includes:

```
- Those servers are enabled: zagros
  - Running zagros-mcp:0.1.0 with [...  -e HELIX_URL ... --add-host host.docker.internal:host-gateway ...]
  > zagros: (7 tools)
```

Seven tools must be listed: `search_cves`, `get_cve`, `index_status`, `sync_cves`, `search_knowledge`, `sync_knowledge_source`, `backfill_cves`.

If `-e HELIX_URL` or `--add-host` is missing the profile snapshot is stale —
re-run `setup.sh` or recreate the profile manually.

### Verify the container can reach HelixDB

```bash
docker run --rm \
  --add-host host.docker.internal:host-gateway \
  -e HELIX_URL=http://host.docker.internal:47474 \
  alpine wget -qO- http://host.docker.internal:47474/healthz
```

Expected:

```json
{"index_runtime":"ready","mode":"writer","ready":true}
```

If this times out, the iptables rule is missing:

```bash
sudo iptables -I INPUT -p tcp --dport 47474 -j ACCEPT
sudo netfilter-persistent save
```

---

## Layer 4 — opencode MCP

### List connected MCP servers

```bash
opencode mcp list
```

Expected:

```
┌  MCP Servers
│
●  ✓ MCP_DOCKER  connected
│      docker mcp gateway run --profile profile
│
└  1 server(s)
```

If it shows `✗ MCP_DOCKER failed` or `No MCP servers configured`:

```bash
# Check the config has the MCP entry
cat ~/.config/opencode/opencode.json | python3 -c \
  "import json,sys; d=json.load(sys.stdin); print(d.get('mcp',{}))"

# Restore it if missing
python3 << 'EOF'
import json
path = "/home/$USER/.config/opencode/opencode.json"
with open(path) as f:
    d = json.load(f)
d["mcp"] = {
    "MCP_DOCKER": {
        "type": "local",
        "command": ["docker", "mcp", "gateway", "run", "--profile", "profile"],
        "enabled": True,
        "environment": {"DOCKER_MCP_IN_CONTAINER": "1"}
    }
}
with open(path, "w") as f:
    json.dump(d, f, indent=2)
print("restored")
EOF
```

---

## Layer 5 — End-to-end via opencode headless

Tests the full stack: opencode → gateway → zagros-mcp container → HelixDB.

```bash
export DOCKER_MCP_IN_CONTAINER=1

# Start opencode server
opencode serve --port 43210 > /tmp/oc-test.log 2>&1 &
sleep 12   # wait for MCP gateway to initialise

# Create a session
SID=$(curl -sf -X POST http://127.0.0.1:43210/session \
  -H "Content-Type: application/json" \
  -d '{"modelID":"github-copilot/claude-sonnet-4.6"}' \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")

# Send a test message
curl -sf -X POST "http://127.0.0.1:43210/session/$SID/message" \
  -H "Content-Type: application/json" \
  -d '{"parts":[{"type":"text","text":"Call index_status and tell me exactly how many CVEs are indexed. Then call search_cves with query remote code execution top_k 3. List the CVE IDs and scores."}],
      "modelID":"github-copilot/claude-sonnet-4.6"}' > /dev/null

sleep 40

# Read response
curl -sf "http://127.0.0.1:43210/session/$SID/message" \
  | python3 -c "
import sys, json
msgs = json.load(sys.stdin)
for m in msgs:
    for p in m.get('parts', []):
        if p.get('type') == 'tool-invocation':
            ti = p['toolInvocation']
            print(f'TOOL: {ti[\"toolName\"]} state={ti[\"state\"]}')
            if ti.get('state') == 'result':
                print(f'  {str(ti[\"result\"])[:300]}')
        elif p.get('type') == 'text' and p.get('text'):
            print(f'RESPONSE: {p[\"text\"][:400]}')
"

# Clean up
kill %1 2>/dev/null
```

**Pass criteria:**

- `TOOL: MCP_DOCKER_index_status state=result` — no `-32603` error
- `TOOL: MCP_DOCKER_search_cves state=result` — returns actual CVE IDs
- `RESPONSE:` contains a CVE count > 0 and at least one CVE ID like `CVE-YYYY-NNNNN`

---

## Quick reference — what each error means

| Symptom | Cause | Fix |
|---|---|---|
| `curl healthz` → connection refused | HelixDB container not running | `docker compose up -d helix` |
| `zagros status` → CVE nodes: 0 | DB empty | `backfill --limit 1000 && source all` |
| `zagros status` → `failed to load CVE nodes` | Wrong `HELIX_URL` | Unset `HELIX_URL` to use default `localhost:47474` |
| `opencode mcp list` → No MCP servers | Config wiped | Restore `MCP_DOCKER` entry in `opencode.json` |
| `opencode mcp list` → `MCP_DOCKER failed` | Gateway fails to start | Check `DOCKER_MCP_IN_CONTAINER=1` is set |
| MCP tool call → `-32603` | Container can't reach HelixDB | Check iptables rule; check `docker port zagros-helix` shows `0.0.0.0:47474` |
| MCP tool call → `-32603` | Stale docker image | `docker build -t zagros-mcp:0.1.0 .` |
| `backfill` returns fewer CVEs than `--limit` | deltaLog parse bug (fixed in current code) | Ensure `src/lib.rs` uses `Vec<DeltaLogEntry>` not `StreamDeserializer` |
