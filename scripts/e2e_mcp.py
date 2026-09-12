#!/usr/bin/env python3
import json
import os
import subprocess
import sys

proc = subprocess.Popen(
    ["target/debug/zagros-mcp"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=sys.stderr,
    text=True,
    env=os.environ.copy(),
)

def send(message):
    proc.stdin.write(json.dumps(message) + "\n")
    proc.stdin.flush()

def read_id(expected):
    while True:
        line = proc.stdout.readline()
        if not line:
            raise RuntimeError("MCP process exited before response")
        payload = json.loads(line)
        if payload.get("id") == expected:
            return payload

send({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2025-06-18",
        "capabilities": {},
        "clientInfo": {"name": "zagros-e2e", "version": "1.0"},
    },
})
init = read_id(1)
if "error" in init:
    raise RuntimeError(init)

send({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
send({
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
        "name": "search_cves",
        "arguments": {"query": "CVE-2099-0001", "top_k": 5},
    },
})
response = read_id(2)
serialized = json.dumps(response)
if "CVE-2099-0001" not in serialized:
    raise RuntimeError(f"fixture CVE missing from MCP response: {serialized}")

print("MCP_E2E_OK")
proc.terminate()
proc.wait(timeout=5)
