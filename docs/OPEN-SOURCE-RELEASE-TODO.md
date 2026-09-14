
# Zagros Open-Source Release TODO

Goal: prepare Zagros for public open-source release as a local-first, secure, trusted source of truth for AI-agent security investigations.

Project scope:
- **Zagros Core** â€” ingestion, HelixDB, retrieval/RAG, CLI, MCP servers, UI.
- **Zagros Skills** â€” agent guidance for using Zagros correctly during security investigations.

## P0 â€” Release blockers

- [x] Resolve skill copyright/provenance before publishing.
  - Skill content is independently authored from the maintainer's personal cybersecurity study notes and practical experience.
  - ISC2 CC influenced topic coverage, but no official ISC2 courseware or exam content is included.
  - Added explicit non-affiliation/provenance wording and third-party notices.

- [x] Reconcile repository licensing.
  - Zagros-authored Core, documentation, and Skills use Apache-2.0.
  - Standalone skill packages include the Apache-2.0 license.
  - Third-party security datasets retain their upstream terms; see `THIRD_PARTY_NOTICES.md`.

- [x] Update stale documentation.
  - VISION.md now reflects the local-first trusted-source mission, the Core + Skills model, and all seven current MCP tools.
  - Website copies of README, VISION, and SECURITY-REVIEW were synchronized.

- [x] Refresh the security review against current code.
  - F-01, F-02, F-04, F-05, and F-06 are fixed.
  - F-03 is mitigated/documented as a HelixDB v3 limitation.
  - No unresolved High-severity finding remains from the previous review.

## P1 â€” Trusted-source guarantees

- [x] Store publisher/source, source version, canonical URL, retrieval URL, license, retrieved timestamp, upstream timestamp, content hash, and trust tier per record/chunk.
- [x] Define trust tiers and claim-specific source-selection/conflict policy.
- [x] Document freshness/staleness behavior.
- [x] Make source updates explicit and auditable.
- [x] Validate upstream source origins.
- [x] Enforce response/download size limits.
- [x] Preserve raw-source provenance where licensing permits: retrieval URLs + raw SHA-256 manifests; optional snapshots via `ZAGROS_PRESERVE_RAW_SOURCES`.
- [x] Distinguish source evidence from Zagros metadata and agent interpretation.
- [x] Require source IDs/URLs in investigation outputs.
- [x] Document prompt-injection handling for retrieved security text.
- [x] Document behavior when authoritative sources disagree.

## P1 â€” Deployment and usability

- [x] Provide one canonical 5-minute local Docker quick start.
- [x] Provide separate cloud/self-hosted deployment guidance.
- [x] Make Traefik optional rather than required by the default Compose setup.
- [x] Add .env.example with safe placeholders.
- [x] Verify Windows, Linux, and macOS/WSL installation paths.
- [x] Document recovery/no-backup policy, upgrade, and uninstall.
- [x] Document bounded CPU/RAM/PID requirements for the current corpus; Zagros data is rebuildable and disk use is corpus-dependent.
- [x] Clarify which ports are local-only and which may be exposed.
- [x] Add authentication guidance for internet-facing MCP deployments.
- [x] Add/verify health and readiness checks for runtime services.

## P1 â€” CI and release engineering

- [x] Add CI for cargo fmt --all -- --check.
- [x] Add CI for cargo clippy --all-targets --all-features -- -D warnings.
- [x] Add CI for cargo test --all-targets.
- [x] Add CI for website npm ci + npm run build.
- [x] Add Docker image build tests for MCP, CLI, and UI.
- [x] Add end-to-end smoke test: HelixDB -> seed -> CLI query -> MCP query.
- [x] Add Rust/npm/container/GitHub Actions dependency scanning.
- [x] Pin important release dependencies/actions where practical.
- [x] Define semantic versioning and CHANGELOG policy.
- [x] Create reproducible GitHub Releases with checksums.
- [x] Publish container images with immutable version tags/digests.
- [x] Automate skill package release and checksum verification.

## P1 â€” Repository and contributor readiness

- [x] Add CONTRIBUTING.md.
- [x] Add CODE_OF_CONDUCT.md.
- [x] Add CHANGELOG.md.
- [x] Add ROADMAP.md.
- [x] Add THIRD_PARTY_NOTICES.md.
- [x] Add issue templates: bug, feature, data-source request.
- [x] Add pull-request template with tests/docs/security checklist.
- [x] Enable GitHub private vulnerability reporting. Enabled after the repository became public on 2026-09-14.
- [x] Add branch protection and required CI checks. `main` now requires PR review, CODEOWNERS approval, conversation resolution, strict required checks, and blocks force-push/deletion.
- [x] Sanitize public Git history. Rewrote reachable history to remove the historical `.vscode/mcp.json`, personal Windows path, and obsolete private deployment domain; post-rewrite scans found no matching token/private-key patterns or those historical identifiers. A pre-rewrite bundle is retained outside the repository for recovery.
- [x] Remove/ignore generated artifacts that should not be versioned; Graphify caches are untracked while intentional reports remain.

## P1 â€” Supply-chain security

- [x] Add Rust dependency audit.
- [x] Add npm dependency audit.
- [x] Add Dependabot or Renovate.
- [x] Add CodeQL.
- [x] Add container vulnerability scanning.
- [x] Generate SBOMs for releases.
- [x] Use immutable Docker digests for production examples.
- [x] Sign release artifacts where practical.
- [x] Publish SHA256 checksums for binaries and skill packages.

## P1 Review â€” 2026-09-12

Status legend: **Done** = implemented and documented; **Partial** = useful pieces exist but release requirement is not fully closed; **Open** = not implemented or not yet verified.

### Trusted-source guarantees

| Item | Status | Review |
|---|---|---|
| Full provenance fields per record/chunk | **Done** | CVE and Knowledge records persist structured provenance in HelixDB and MCP returns publisher, source/version, canonical/retrieval URLs, license, timestamps, SHA-256 content hash, and trust tier. Legacy rows load as `legacy_unknown` until refreshed. |
| Trust tiers and source-selection policy | **Done** | `TrustTier` is implemented; current corpus sources are authoritative. `SOURCE-TRUST-POLICY.md` defines claim-specific authority and conflict handling. |
| Freshness/staleness behavior | **Done** | Daily refresh runs in the MCP container; `index_status` exposes refresh state/last attempt/last success and refresh history is persisted. |
| Explicit/auditable source updates | **Done** | Scheduled refreshes and successful manual MCP sync/backfill operations append to `/data/refresh-history.jsonl`. |
| Validate upstream source origins | **Done** | Knowledge sources use fixed canonical URLs; dynamic CVE links are restricted to the CVE Project raw GitHub origin. |
| Enforce response/download size limits | **Done** | Limits exist for delta.json, deltaLog, individual CVEs, CWE ZIP/decompressed XML, ASVS CSV, CAPEC XML, and ATT&CK STIX. |
| Preserve raw-source provenance | **Done** | Exact retrieval URLs and raw SHA-256 hashes are recorded in source manifests; raw snapshots are optional via `ZAGROS_PRESERVE_RAW_SOURCES`. |
| Separate evidence from interpretation | **Done** | MCP guidance and the cybersecurity skill require Evidence / Analysis / Recommendation separation and prohibit presenting inference as retrieved fact. |
| Require source IDs/URLs in investigation outputs | **Done** | MCP/skill guidance requires record IDs and canonical source URLs for evidence-backed claims. |
| Prompt-injection handling | **Done** | MCP responses and docs explicitly classify retrieved third-party text as untrusted data, never instructions. |
| Authoritative-source disagreement policy | **Done** | Claim-specific precedence and explicit conflict reporting are documented in `SOURCE-TRUST-POLICY.md` and agent guidance. |

### Deployment and usability

| Item | Status | Review |
|---|---|---|
| Canonical 5-minute local Docker quick start | **Done** | Base Compose is self-contained and local-safe; MCP/UI default to loopback bindings. |
| Cloud/self-hosted deployment guide | **Done** | `DEPLOYMENT.md` documents local/cloud deployment, reverse proxies, authentication boundary, recovery, upgrade, and uninstall. |
| Traefik optional in default Compose | **Done** | Base Compose has no Traefik dependency; `docker/compose.traefik.yml` is an optional override. |
| `.env.example` | **Done** | Safe local defaults and optional Traefik settings are provided. |
| Windows/Linux/macOS or WSL verification | **Done** | Docker Compose is the supported cross-platform path; Windows, Linux, macOS, and WSL2 behavior is documented. |
| Recovery/no-backup/upgrade/uninstall docs | **Done** | Zagros is documented as a rebuildable derived index; recovery, upgrade, and uninstall procedures are documented. |
| Resource requirements | **Done** | CPU, RAM, and PID ceilings are enforced for each Compose service; disk use is corpus-dependent because the index is rebuildable. |
| Port exposure documentation | **Done** | README/configuration document HelixDB, MCP, and UI bindings and intended exposure. |
| Authentication guidance for public MCP | **Done** | `DEPLOYMENT.md` requires an authenticated reverse proxy/access gateway for non-local MCP exposure; host allowlisting is explicitly not authentication. |
| Health/readiness checks | **Done** | MCP `/health` verifies HelixDB connectivity; UI `/api/status` remains the UI readiness check. |

### CI and release engineering

**Done:** CI now gates Rust formatting, clippy, tests, website build, Compose validation, and all three Docker image builds. A deterministic HelixDB -> seed -> CLI -> MCP E2E workflow runs on pull requests, main, schedule, and manual dispatch. Security automation covers Rust/npm audits, CodeQL, container scanning, Dependabot, and live-source checks. Tagged releases build Linux/Windows binaries, the deterministic skill package, signed GHCR images, checksums, BuildKit provenance/SBOM attestations, and a downloadable SPDX SBOM. Important release actions are pinned by commit SHA. The E2E workflow pins HelixDB v3.1.1 by immutable commit and builds it from source on Ubuntu. Local Linux-container verification was attempted on 2026-09-12, but Docker Desktop terminated the build twice with `unexpected EOF`; no Helix compile error was observed. Hosted validation is now green on the public repository: CI run 34847953201, Security run 34847953062, OpenSSF Scorecard run 34847953135, and E2E smoke run 34847952996 all succeeded on commit `875d1b8`.

### Repository and contributor readiness

- **Done:** `THIRD_PARTY_NOTICES.md`, `SECURITY.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `ROADMAP.md`, issue templates, and PR template.
- **Done:** GitHub Private Vulnerability Reporting is enabled.
- **Done:** `main` branch protection is active with one approval, CODEOWNERS review, stale-review dismissal, conversation resolution, strict required status checks, and force-push/deletion disabled.
- **Done:** repository-history hygiene. Reachable history was rewritten to remove the historical `.vscode/mcp.json`, personal Windows path, and obsolete private deployment domain. Post-rewrite scans found no matching token/private-key patterns or those historical identifiers. A recovery bundle is retained locally outside the repository.
- **Done:** generated-file hygiene for P1. Build outputs and Graphify caches are ignored/untracked; intentional Graphify reports remain versioned.

### Supply-chain security

**Done for repository automation:** Rust and npm audits, Dependabot, CodeQL, Trivy container scanning, release SBOM generation, BuildKit SBOM/provenance attestations, keyless Cosign signing, SHA256 release checksums, and immutable GHCR digest publication are implemented. Runtime images were hardened until local Trivy scans reported zero HIGH/CRITICAL findings. Production release documentation demonstrates digest-pinned deployment. Hosted CI, E2E, Security/CodeQL, dependency/container scanning, and OpenSSF Scorecard have been confirmed green on the public repository.

---

## P3 — Public-release hardening

- [x] Add `.github/CODEOWNERS` for repository and security-sensitive ownership.
- [x] Pin every external GitHub Actions `uses:` reference to a full immutable commit SHA; verification found 49 external references and zero unpinned references.
- [x] Add the official GitHub Dependency Review pull-request gate, failing on moderate-or-higher newly introduced vulnerabilities.
- [x] Add OpenSSF Scorecard with SARIF/code-scanning upload and immutable action pins.
- [x] Make the sanitized repository public.
- [x] Enable Secret Scanning and Push Protection.
- [x] Enable Dependabot security updates.
- [x] Enable Private Vulnerability Reporting.
- [x] Protect `main` with required pull-request review, CODEOWNERS review, resolved conversations, strict required checks, and force-push/deletion disabled.
- [x] Fix CodeQL workflow permissions and verify the public Security workflow succeeds.
- [x] Verify public hosted CI, E2E, Security, and OpenSSF Scorecard workflows all succeed on commit `875d1b8`.
- [x] Final public-history scan found zero tracked secret-like files and zero matching credential/private-key patterns in reachable `main` history.

---

## P2 — Skills ecosystem

- [x] Define a stable Zagros Skill Contract. Contract v1 is documented in `docs/SKILL-CONTRACT.md` and enforced by `skills/validate.py`.
- [x] Keep generic security guidance harness-neutral. `SKILL.md` remains generic; harness/Zagros wiring is kept outside the core guidance.
- [x] Keep Zagros-specific MCP mappings in a separate reference. Current mapping: `skills/.apm/skills/cybersecurity-expert/references/zagros-mcp.md`.
- [x] Document Microsoft APM install, upgrade, uninstall, and compatibility. See `skills/COMPATIBILITY.md`.
- [x] Add skill evaluation cases with expected behavior. The suite now has 12 cases including injection, source conflict, evidence/inference, stale data, and approval boundaries.
- [x] Add examples for OpenCode, Copilot, Claude-compatible clients, and generic MCP clients. See `skills/HARNESSES.md`.
- [x] Define how community-contributed skills are reviewed and trusted. See `docs/SKILL-TRUST-POLICY.md`.
- [x] Version skills independently from Zagros Core when needed. Skill SemVer and Core compatibility are explicit in package metadata and `skills/COMPATIBILITY.md`.
- [x] Rename `skill/` to `skills/` for future multi-skill growth and update release/docs/install paths.

## P2 — Quality and project confidence

- [x] Add architecture decision records for major design choices. See docs/adr/.
- [x] Add threat model for local, LAN, and public-cloud deployments. See docs/THREAT-MODEL.md.
- [x] Add retrieval-quality benchmarks and regression tests. Deterministic offline benchmark: 32 cases with Recall@1, Recall@5, MRR, and irrelevant-query rejection gates.
- [x] Add corpus integrity/freshness metrics. `index_status` now reports per-source count thresholds, freshness age/state, and overall health.
- [x] Publish a roadmap (`ROADMAP.md`).
- [x] Label experimental vs stable interfaces/features clearly. See docs/INTERFACE-STATUS.md.
- [x] Add screenshots/demo flow for a real security investigation. See `docs/DEMO.md` and the public `/demo/` page.
- [x] State non-goals clearly: Zagros is an evidence source, not compliance certification or autonomous remediation.

## P2 — Graphify cleanup

- [x] Add `.graphifyignore`.
- [x] Exclude generated/unparseable inputs: target, website build/cache directories, node_modules, skill dist, Graphify outputs/caches, data, and `*.astro` parser-noise files.
- [x] Investigate/remove stale references to `C:/Projects/Zagros`; none remain in rebuilt public Graphify outputs.
- [x] Rebuild Graphify with `--force` after cleanup: 1,810 nodes / 2,312 edges / 188 communities.
- [x] Run multigraph diagnostics: zero duplicate/collapsed/dangling/self-loop edges.
- [x] Define public Graphify artifacts: keep `GRAPH_REPORT.md`, `graph.json`, `graph.html`, and `manifest.json`; exclude `cost.json`, caches, and dated backups.
## Current verified baseline — 2026-09-14

- [x] Graphify refreshed after cleanup: 1,810 nodes / 2,312 edges / 188 communities.
- [x] Multigraph diagnostics clean: no duplicate, collapsed, dangling, or self-loop edges.
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo test --all-targets` passes: 9 tests passed across library/MCP/retrieval suites; E2E seed test intentionally ignored without HelixDB.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [x] Astro website builds successfully: 37 static pages including the investigation demo and quality-metrics docs.
- [x] Tracked-file/history scans found no obvious committed credentials/private keys or generated build caches.
- [x] Graphify Astro/generated parser noise is excluded through `.graphifyignore`; only zero-node JSON warnings for tooling/eval metadata remain.

## First public-release gate

Do not tag the first public release until:
1. [x] Skill licensing/provenance is resolved.
2. [x] Current security review has no unresolved High findings.
3. [ ] Fresh clone -> local Docker deployment works without a personal Traefik environment.
4. [x] CI and end-to-end tests are green.
5. [x] Documentation accurately describes the current implementation.

**Remaining release gate:** perform the clean fresh-clone Docker acceptance test, then prepare/tag the first public release.
