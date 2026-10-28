# Railway deployment — Zagros (preview)

This guide documents the three-service deployment tested on 2026-10-28 in the Railway project `wonderful-patience`. The original Docker Builder v4 `VOLUME` error is fixed in the Railway-specific Dockerfiles. **The deployment is unverified for security and scheduled ingestion.** Do not expose the services publicly until authenticated ingress is implemented and tested. Normal Docker Compose installation is unchanged.

## Architecture and configuration

| Service | Dockerfile | Private endpoint | Probe | Storage |
| --- | --- | --- | --- | --- |
| `zagros-helix` | `docker/Dockerfile.helix.railway` | `zagros-helix.railway.internal:8080` | `/healthz` | 500 MB at `/data` |
| `Zagros` (MCP HTTP) | `Dockerfile.railway` | `zagros.railway.internal:8789` | `/health` | 500 MB at `/data` |
| `zagros-ui` | `Dockerfile.ui` | `zagros-ui.railway.internal:8788` | `/healthz` | none |

All three services build from repository root. The Railway Free plan tested here limits volume size to 500 MB, requires sleeping, and does not allow a fourth cron worker; limits may change by plan. If deploying from a feature branch, use the same commit across services. After PR approval, use a reviewed release commit rather than an unreviewed mutable development branch.

Required service variables:

```text
# zagros-helix
HELIX_HTTP_ADDR=0.0.0.0:8080
HELIX_GRPC_ADDR=0.0.0.0:8081
HELIX_DATA_DIR=/data

# Zagros MCP
HELIX_URL=http://zagros-helix.railway.internal:8080
MCP_BIND_ADDR=0.0.0.0:8789
MCP_ALLOWED_HOSTS=localhost,127.0.0.1,zagros.railway.internal
ZAGROS_DATA_DIR=/data
ZAGROS_AUTO_REFRESH=true
ZAGROS_REFRESH_CRON=0 3 * * *
ZAGROS_DAILY_CVE_LIMIT=1000

# zagros-ui
HELIX_URL=http://zagros-helix.railway.internal:8080
UI_PORT=8788
PORT=8788
```

`PORT=8788` is essential: Railway's healthcheck otherwise may probe an unrelated injected port and fail even while the UI logs that it is listening on 8788. `/healthz` verifies the UI process only; `/api/status` remains dependency-aware and must be checked separately to validate HelixDB access.

## Provisioning

1. Create a Railway project and three GitHub-backed services from `siamakhaz/Zagros` in a private environment. Specify the Dockerfile path from the table for each service. The default root `Dockerfile` is **not** Railway-compatible due to its Dockerfile-level `VOLUME`.
2. Set the variables above, enable private networking, configure each healthcheck, and attach **two different** volumes to `/data` (never share one volume across services). Do not create public domains.
3. Review the opt-in `.railway/railway.ts` blueprint if using Railway IaC. Treat it as a reference until its syntax and `railway config plan` output are independently validated against the installed Railway CLI. Do not automatically apply it; inspect resource costs and all planned changes first.
4. Deploy the HelixDB image, check its logs and `/healthz`, then deploy MCP and UI and check their logs and health checks.
5. Confirm the MCP can query Helix and the UI `/api/status` can fetch real data. Liveness-only responses do not prove data-plane connectivity.
6. Verify writes to each volume, restart/redeploy, then compare persisted indexes and refresh metadata. Check data size and the volume capacity before backfill.
7. Test anonymous denial and authorized access through an approved authentication/authorization gateway *before* adding any public MCP/UI domain. TLS and the MCP Host header allowlist do not authenticate callers; Helix must remain private.

## Startup permissions and security gate

Railway's root-owned volume mounts caused `cannot create /data/zagros-crontab: Permission denied` under the original non-root container. This experimental branch removes the root startup helper and declares `USER zagros` and `USER helix` in the respective Railway Dockerfiles. This eliminates privileged application startup and the Dockerfile-level DS-0002 finding; it **does not itself fix root-owned Railway volume mounts**. The existing running Railway volumes were previously chowned by the old startup helper, but we have not verified that ownership is retained after a fresh deploy or restart. Do not deploy these images to a newly attached volume until a one-time, explicitly reviewed ownership bootstrap has set the mount ownership for the correct service UID. Bootstrap must be isolated from the long-running application, use a dedicated operator-controlled administrative job with the volume attached, and be removed after validation. Do not use a long-running root application or disable Trivy. Do not perform recursive ownership changes on populated volumes without a backup and risk review. If Railway remounts the volume as root on each deployment, this model is not viable without a supported volume-ownership mechanism or separate storage design; rollback instead of claiming a fix.

Do not set `RAILWAY_RUN_UID=0` as a blanket workaround without an explicit security review; it keeps the application privileged.\n\nThe standard Docker images and Docker Compose mounts are unchanged.

## Refresh and backfill

The MCP container's existing `supercronic` scheduler uses `0 3 * * *` UTC, invokes the Zagros ingestion command with a daily CVE cap of 1000, then refreshes source data. **This is a bounded incremental refresh, not guaranteed full historical backfill.** Railway sleeping can stop the process, making the in-container cron unreliable. A dedicated always-on or Railway-scheduled job needs the appropriate plan/resources and a verified single-run entrypoint. Do not claim freshness until successful run logs, source provenance, and updated index counts are observed.

## Evidence and known limitations

2026-10-28 observed Railway deployment outcomes:
- Zagros UI `0c9a563f`: `SUCCESS`; `/healthz` returned a successful Railway health check after setting `PORT=8788`.
- Helix `749f1589`: `SUCCESS` with successful health check.
- MCP `a2fbdf44`: previously `SUCCESS`, then `SLEEPING` on Free plan; logs showed scheduler initialized and listening on port 8789.

**Not yet independently verified (including non-root mounted-volume write capability after redeploy):** cross-service application queries, volume persistence across redeploy, reliable cron execution while sleeping, live authenticated remote access, and security review of root startup. The PR must not be merged on healthcheck success alone while required security CI fails.

## Development verification

```powershell
python -m unittest scripts/test_railway_deployment.py
```

Check GitHub CI and Security on the exact PR head commit. Static tests are not a Railway runtime substitute.

References:
- https://docs.railway.com/volumes
- https://docs.railway.com/deployments/healthchecks
- https://docs.railway.com/builds/dockerfiles
- https://docs.railway.com/infrastructure-as-code/reference
