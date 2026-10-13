# Zagros — Remaining Public Release TODO

## Current state

Repository: `D:\Projects\RAG\Zagros`

GitHub: `https://github.com/siamakhaz/Zagros`

Current public main:
`685ea7c5c9ac0f4c099a2fc173b4e799b902ff85`

- P1 complete
- P2 complete
- P3 implementation complete
- Public history privacy rewrite complete
- Active reachable history is clean
- Git author uses GitHub noreply
- Secret scanning enabled
- Push protection enabled
- Dependabot security updates enabled
- Branch protection restored
- Force pushes disabled
- Forks: 0

GitHub Support purge ticket: `#4757775`
## 1. Post-rewrite GitHub verification

Verify all hosted checks against current public HEAD:

- [x] Skill Contract
- [x] Rust quality
- [x] Website
- [x] Compose validation
- [x] Docker build - mcp
- [x] Docker build - cli
- [x] Docker build - ui
- [x] helix-cli-mcp
- [ ] dependency-audit — public HEAD fails on RUSTSEC-2026-0285; local lockfile updated to rustls 0.23.45 and `cargo audit` now passes
- [ ] Trivy repository scan — public HEAD fails DS-0002 on `docker/Dockerfile.helix`; local non-root runtime fix passes targeted Trivy config scan
- [x] Container scan - mcp
- [x] Container scan - cli
- [x] Container scan - ui
- [x] CodeQL
- [x] Dependency Review
- [x] OpenSSF Scorecard
- [x] E2E smoke

Do not rely on runs from before the history rewrite.
## 2. Final public fresh-clone Docker acceptance

Clone directly from public GitHub into a new temporary folder.

- [x] Public clone HEAD matches GitHub main
- [x] `docker compose -f docker/compose.yml config` passes
- [x] Default stack builds without Traefik or personal environment
- [x] Helix builds from pinned source
- [x] Helix `/healthz` reports ready
- [x] MCP `/health` is healthy
- [x] UI `/api/status` connects
- [x] CLI image builds explicitly from fresh clone
- [x] CLI `status` works
- [x] Real source ingestion works, preferably ASVS
- [x] Provenance manifest is written correctly
- [x] Default data path is root `/data`, not `docker/data`
- [x] Fresh clone remains Git-clean
- [x] Acceptance stack is cleaned up
- [x] Normal Zagros stack is restored and healthy
## 3. Final release documentation

After public fresh-clone acceptance passes:

- [x] Mark the Docker fresh-clone release gate complete
- [x] Update website documentation mirror if applicable
- [x] Record that public-history privacy sanitation was completed
- [x] Record that Git authors use GitHub noreply
- [x] Do not document removed PII values
- [x] Run stale wording scan
- [x] Run `git diff --check`
- [x] Build website successfully
- [x] Commit final documentation locally

Suggested commit:
`docs: complete public release readiness checklist`

Do not push unless explicitly requested.
## 4. GitHub Support purge follow-up

Ticket: `#4757775`

- [ ] Confirm affected PR refs were dereferenced
- [ ] Confirm server-side garbage collection completed
- [ ] Confirm cached commit views were removed
- [ ] Verify known old pre-rewrite commit SHA is no longer accessible
- [ ] Confirm no old refs reappeared
- [ ] Delete private recovery bundle only after purge is verified

Private recovery bundle:
`D:\Projects\RAG\Zagros-pre-privacy-rewrite-20260914.bundle`

Keep this bundle private.
## 5. First public release

After all release checks are green:

- [ ] Decide release version, likely `v0.2.0`
- [ ] Review CHANGELOG
- [ ] Verify release workflow
- [ ] Verify checksums
- [ ] Verify SBOM
- [ ] Verify provenance
- [ ] Verify signing/cosign
- [ ] Verify packaged skills/artifacts
- [ ] Create release tag only with explicit authorization
- [ ] Verify GitHub Release artifacts after publishing

## Working rules

- Use Remote Desktop Commander / MyRDC for Legion work
- Do not push unless explicitly asked
- Distinguish commit from push
- Verify before claiming success
- PowerShell on Legion: do not use `&&`; use `;`
