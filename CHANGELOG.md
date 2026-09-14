# Changelog

All notable changes to Zagros are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses Semantic Versioning.

## [Unreleased]

### Added
- Structured trusted-source provenance, source manifests, and daily source refresh.
- Stable Zagros Skill Contract, harness guidance, trust policy, evaluation cases, and multi-skill layout.
- Deterministic retrieval benchmarks, corpus integrity/freshness metrics, ADRs, threat model, interface-status documentation, and investigation demo.
- CI, deterministic HelixDB-to-CLI-to-MCP E2E smoke testing, Dependency Review, CodeQL, Trivy, Dependabot, and OpenSSF Scorecard.
- CODEOWNERS, issue/PR templates, Private Vulnerability Reporting, Secret Scanning, and Push Protection.
- Release automation for binaries, containers, skill packages, checksums, SPDX SBOMs, BuildKit provenance, attestations, and keyless container signing.

### Changed
- Default Docker deployment is local-safe and no longer requires Traefik.
- Skill packaging is reproducible when `SOURCE_DATE_EPOCH` is set.
- All external GitHub Actions references are pinned to immutable commit SHAs.
- `main` is protected with required PR/CODEOWNERS review, strict status checks, conversation resolution, and force-push/deletion protection.
- Repository history was sanitized before the public transition.

### Security
- Public Security workflow, including CodeQL, dependency audits, repository scanning, and container scanning, is green.
- OpenSSF Scorecard is enabled and succeeds on the public repository.
- Final public-history hygiene scan found no tracked secret-like files or matching credential/private-key patterns in reachable `main` history.
