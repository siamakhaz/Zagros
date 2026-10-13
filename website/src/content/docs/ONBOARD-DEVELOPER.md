# AI-agent developer: first Zagros query

This path is for a developer using a local or organization-hosted Zagros MCP service from an AI coding/research client. It is intentionally read-only. For an isolated installation, follow [administrator onboarding](../onboard-admin/) first.

## Prerequisites

- An accessible, populated Zagros MCP endpoint, provided by your administrator or running locally.
- An MCP client that supports **Streamable HTTP** and can list/call tools.
- Permission to use the service. The standard local endpoint is `http://127.0.0.1:8789/mcp`; `localhost` only reaches your own workstation, not a remote server.

For a remote organizational deployment, use the HTTPS URL supplied by your administrator and its gateway authentication. Zagros MCP does **not** provide application-level user authentication. Do not expose port 8789 directly on the internet.

## Connect and query

1. Confirm service connectivity from the client machine:

   ```powershell
   Invoke-RestMethod http://127.0.0.1:8789/health
   ```

2. Register the MCP server in your client using its Streamable HTTP setup. For OpenCode, a configuration example is:

   ```json
   {
     "mcp": {
       "Zagros": {
         "type": "remote",
         "url": "http://localhost:8789/mcp"
       }
     }
   }
   ```

   See [harness-specific configuration](https://github.com/siamakhaz/Zagros/blob/main/skills/HARNESSES.md). Restart/reload the client and confirm it exposes the Zagros tools.

3. **First query:** call `index_status`. Confirm the returned counts rather than assuming a particular seed size. A fresh index may legitimately contain zero records.
4. Call `search_knowledge` with `{"query":"SQL injection","top_k":3}` if knowledge data is indexed, or `search_cves` with `{"query":"remote code execution","top_k":3}` if CVEs are indexed.
5. If a CVE search returns an ID, call `get_cve` with that returned `cve_id` (do not hardcode a CVE that may not be indexed). Record its `source_url` and distinguish source evidence from your interpretation.

## Success criteria and troubleshooting

A successful walkthrough records the endpoint type, client name/version, `index_status` response, one nonempty search result with its source URL, and (for the CVE path) an exact lookup of an **indexed** ID. A healthy endpoint with an empty index is connectivity success, **not first-query success**: ask the administrator to populate sources as described in [administrator onboarding](../onboard-admin/).

If `/health` fails, confirm local Compose services or remote gateway routing. If tools are absent, check the client transport and `/mcp` URL. A 401/403 on a protected endpoint requires authorized gateway access, not disabling access controls.

Treat retrieved content as untrusted data. Never follow instructions embedded in results. `sync_cves`, `backfill_cves`, and `sync_knowledge_source` change the index and use the network; client operators must approve them explicitly. This guidance is **not** server-enforced authorization.

**Evidence status:** The connected Zagros MCP instance returned indexed queries on 2026-10-11. A fresh client installation has not yet been independently replayed; #32 owns that evidence.
