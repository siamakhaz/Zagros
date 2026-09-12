# Changelog

All notable changes to Zagros are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses Semantic Versioning.

## [Unreleased]

### Added
- Trusted-source provenance groundwork.
- Daily source refresh and deployment hardening.
- CI, security scanning, E2E smoke-test, and release automation.

### Changed
- Default Docker deployment is local-safe and no longer requires Traefik.
- Skill packaging is reproducible when `SOURCE_DATE_EPOCH` is set.

### Security
- Added dependency, CodeQL, repository, and container scanning workflows.
