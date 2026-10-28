# Deploy Zagros on Railway (preview)

This is an **opt-in, unverified Railway deployment blueprint** for issue #97. It has not been live-deployed. The existing Docker Compose installation is unchanged.

## Services

- `zagros-helix`: HelixDB, private HTTP port 8080; Railway persistent volume at `/data`.
- `zagros-mcp`: Zagros MCP HTTP, port 8789; separate persistent volume at `/data` for refresh/status data. It calls `http://zagros-helix.railway.internal:8080`.
- Web UI is **not** included in the initial blueprint. A separate service can be added after the core stack is proven.

Both services use the same GitHub repository and build from the repository **root** with custom Dockerfiles: `docker/Dockerfile.helix.railway` and `Dockerfile.railway` respectively. Do not use the root default `Dockerfile` for Railway because it declares `VOLUME`. On each Railway service, verify the Dockerfile Path setting matches the respective filename and the build context is the repository root. `RAILWAY_DOCKERFILE_PATH` is provided in the blueprint; verify Railway resolves it before deploying.

## Review and provision (requires Railway account and explicit operator approval)

1. Install and authenticate the current Railway CLI and link the intended Railway workspace/project.
2. Review `.railway/railway.ts` including region (`us-west2`), costs, storage sizes, code source and environment variables. The blueprint references `main`, so test changes on a safe branch/environment until merged.
3. Run `railway config plan` and inspect **all** proposed changes before any `railway config apply`. Never auto-apply from CI. Applying the file may provision billable services and volumes.
4. Confirm both services use their custom Dockerfile paths, two separate `/data` volumes, and no public domain.
5. **Permissions blocker:** Railway volumes are mounted as root, while Zagros and Helix run under non-root users by default. Railway documents setting `RAILWAY_RUN_UID=0` to access root-owned volumes, which weakens the container privilege boundary. Do not enable root automatically. Review risk and validate a non-root-compatible ownership strategy before considering the deployment production-ready.
6. Confirm Helix starts and passes `/healthz`, then MCP passes `/health` and can complete `index_status` over the internal network. Railway does not translate Docker Compose `depends_on`; startup/retry behavior must be validated.
7. Before adding any public domain for MCP, put an authenticated and authorized gateway in front of `/mcp` (and the UI if later added). TLS and `MCP_ALLOWED_HOSTS` alone are **not authentication**. Confirm anonymous clients are denied and authorized clients can connect through the gateway. Do not publish HelixDB.
8. Verify persistence across restart/redeploy, including access permissions, refresh metadata and Helix indexes. Document actual deployment results and limits before closing #97.

A Railway volume is attached at runtime, not during a Dockerfile build. Removing Dockerfile `VOLUME` changes image metadata but does not remove existing Compose mounts.

## Verification

Run static checks:

```powershell
python -m unittest scripts/test_railway_deployment.py
```

Static checks do **not** prove a Railway build, storage permissions, Helix connectivity, or authentication.

References:
- https://docs.railway.com/infrastructure-as-code/reference
- https://docs.railway.com/guides/docker-compose
- https://docs.railway.com/volumes
- https://docs.railway.com/builds/dockerfiles
