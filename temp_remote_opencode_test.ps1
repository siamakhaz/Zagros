$ErrorActionPreference = 'Stop'

$remote = @'
set -euo pipefail

cat >/tmp/opencode_restore_mcp.py <<'PY'
import json
path = "/home/sizan/.config/opencode/opencode.json"
with open(path) as f:
    d = json.load(f)
d["mcp"] = {
    "MCP_DOCKER": {
        "type": "local",
        "command": ["docker", "mcp", "gateway", "run", "--profile", "profile"],
        "enabled": True,
        "environment": {"DOCKER_MCP_IN_CONTAINER": "1"},
    }
}
with open(path, "w") as f:
    json.dump(d, f, indent=2)
print("config restored")
PY
python3 /tmp/opencode_restore_mcp.py

export DOCKER_MCP_IN_CONTAINER=1
pkill -f 'opencode serve' 2>/dev/null || true
sleep 2
opencode serve --port 43225 > /tmp/oc-final-test.log 2>&1 &
SERVE_PID=$!
cleanup() {
  kill "$SERVE_PID" 2>/dev/null || true
  pkill -f 'opencode serve --port 43225' 2>/dev/null || true
}
trap cleanup EXIT

sleep 15

SID=$(curl -sf -X POST http://127.0.0.1:43225/session \
  -H 'Content-Type: application/json' \
  -d '{"modelID":"github-copilot/claude-sonnet-4.6"}' \
  | python3 -c 'import sys,json; print(json.load(sys.stdin)["id"])')
printf 'SID=%s\n' "$SID"

curl -sf -X POST "http://127.0.0.1:43225/session/$SID/message" \
  -H 'Content-Type: application/json' \
  -d '{"parts":[{"type":"text","text":"Call index_status and tell me exact records count. Then call search_cves with query remote code execution top_k 3. Show raw CVE IDs and scores."}],"modelID":"github-copilot/claude-sonnet-4.6"}' \
  > /tmp/opencode-message-post.json

sleep 40

curl -sf "http://127.0.0.1:43225/session/$SID/message" > /tmp/opencode-message-get.json

python3 - <<'PY'
import json

def dump(label, path):
    print(label)
    with open(path) as f:
        print(json.dumps(json.load(f), indent=2))

dump('MESSAGE_POST_RESPONSE:', '/tmp/opencode-message-post.json')
dump('MESSAGE_GET_RESPONSE:', '/tmp/opencode-message-get.json')
PY

if grep -q -- '-32603' /tmp/oc-final-test.log /tmp/opencode-message-post.json /tmp/opencode-message-get.json 2>/dev/null; then
  echo 'ERROR_32603=YES'
else
  echo 'ERROR_32603=NO'
fi
'@

$remote | ssh -o BatchMode=yes sizan@10.100.50.116 bash -s
