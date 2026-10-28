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
5. **Volume initialization:** Railway mounts volumes root-owned. The Railway-specific images start a minimal root-owned `railway-volume-entrypoint` that validates `/data`, adjusts its ownership, and immediately runs the MCP/Helix workload via `runuser` under the unprivileged `zagros`/`helix` account. This introduces a privileged startup phase only on Railway; review its effects on existing volume data and verify process UIDs and persistence before production. Do not set `RAILWAY_RUN_UID=0` on the regular Docker images.
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

## October 28 Railway build follow-up

Railway's Helix build failed because the exact apt package version `ca-certificates=20250419~deb12u1` was unavailable from current Bookworm mirrors. The Railway-only Dockerfile installs distribution packages without frozen point-release package versions, while retaining the digest-pinned Debian base image. The resulting package versions need to be recorded from an actual successful build. The first MCP volume deployment reached `SUCCESS` but its process repeatedly failed to create `/data/zagros-crontab` with `Permission denied`; the Railway-only volume-entrypoint addresses this with privileged ownership initialization followed by unprivileged execution. Recheck actual logs; neither fix is verified merely by a passing CI run.

A separate Railway UI service uses `Dockerfile.ui` and `HELIX_URL` pointing to the private Helix service. The current Free plan supported 500 MB volumes; it required sleeping and rejected a standalone cron service. The MCP process includes supercronic with `ZAGROS_REFRESH_CRON=0 3 * * *`, but an in-process cron may not execute reliably while Railway sleeps the service. Do not claim scheduled backfill reliability without observed execution. No public domain should be added without authenticated ingress.
