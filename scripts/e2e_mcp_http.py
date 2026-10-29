#!/usr/bin/env python3
"""End-to-end MCP HTTP transport compatibility and Host-header regression check.

Run after HelixDB is seeded with CVE-2099-0001 (see e2e.yml).
No public listener or external dependencies are required.
"""
import http.client
import json
import os
import socket
import subprocess
import time


def request(port, payload, *, host="localhost", session=None):
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=15)
    headers = {
        "Host": host,
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
    }
    if session:
        headers["Mcp-Session-Id"] = session
    conn.request("POST", "/mcp", body=json.dumps(payload), headers=headers)
    response = conn.getresponse()
    status = response.status
    session_id = response.getheader("Mcp-Session-Id")
    content_type = response.getheader("Content-Type", "")
    body = response.read(1024 * 1024).decode("utf-8", errors="replace")
    conn.close()
    # Rejected Host headers may return a plain-text HTTP error, not JSON.
    # Keep the HTTP status authoritative; do not hide malformed 2xx MCP replies.
    if status >= 400 and "text/event-stream" not in content_type and "json" not in content_type:
        result = {"http_error": body[:512]}
    elif "text/event-stream" in content_type:
        messages = [
            json.loads(line[5:].strip())
            for line in body.splitlines()
            if line.startswith("data:") and line[5:].strip().startswith("{")
        ]
        if status == 200 and not messages:
            raise AssertionError(f"Empty MCP event stream: HTTP {status}, {body[:256]!r}")
        result = messages[-1] if messages else {}
    elif body.strip():
        try:
            result = json.loads(body)
        except json.JSONDecodeError as exc:
            raise AssertionError(
                f"Malformed MCP JSON: HTTP {status}, Content-Type={content_type}, {body[:256]!r}"
            ) from exc
    else:
        result = {}
    return status, session_id, result


def rpc(rpc_id, method, params=None):
    message = {"jsonrpc": "2.0", "id": rpc_id, "method": method}
    if params is not None:
        message["params"] = params
    return message


def main():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        port = sock.getsockname()[1]
    env = os.environ.copy()
    env["MCP_BIND_ADDR"] = f"127.0.0.1:{port}"
    env["MCP_ALLOWED_HOSTS"] = "localhost,127.0.0.1"
    proc = subprocess.Popen(
        ["target/debug/zagros-mcp-http"],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        for _ in range(100):
            if proc.poll() is not None:
                raise RuntimeError(f"MCP HTTP exited early: {proc.returncode}")
            try:
                with socket.create_connection(("127.0.0.1", port), timeout=0.2):
                    break
            except OSError:
                time.sleep(0.1)
        else:
            raise TimeoutError("MCP HTTP did not start")

        init = rpc(1, "initialize", {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "zagros-http-e2e", "version": "1.0"},
        })
        blocked_status, _, _ = request(port, init, host="untrusted.example")
        assert blocked_status in (400, 403, 421), (
            f"Unexpected Host header accepted or unexpected error: {blocked_status}"
        )

        status, session, initialized = request(port, init)
        assert status == 200 and "result" in initialized, (
            f"MCP HTTP initialize failed: HTTP {status}, {initialized}"
        )
        assert "protocolVersion" in initialized["result"], initialized

        # A session must be preserved for subsequent stateful MCP calls.
        assert session, "MCP HTTP did not return a session ID"
        status, _, _ = request(
            port,
            {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}},
            session=session,
        )
        assert status in (200, 202, 204), f"Initialized notification failed: {status}"
        status, _, tools = request(port, rpc(2, "tools/list"), session=session)
        assert status == 200 and "result" in tools, (
            f"MCP HTTP tools/list failed: HTTP {status}, {tools}"
        )
        names = {item["name"] for item in tools["result"]["tools"]}
        assert {"search_cves", "get_cve"}.issubset(names), names

        status, _, search = request(
            port,
            rpc(3, "tools/call", {
                "name": "search_cves",
                "arguments": {"query": "CVE-2099-0001", "top_k": 5},
            }),
            session=session,
        )
        assert status == 200 and "result" in search, (
            f"MCP HTTP tools/call failed: HTTP {status}, {search}"
        )
        assert "CVE-2099-0001" in json.dumps(search), search
        print("MCP_HTTP_E2E_OK")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)


if __name__ == "__main__":
    main()
