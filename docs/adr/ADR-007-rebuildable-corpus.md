# ADR-007 — Rebuildable corpus; backups are optional

**Status:** Accepted
**Date:** 2026-09-13

## Context
Zagros indexes public authoritative sources and can regenerate derived database state from upstream data and recorded configuration.

## Decision
Treat the indexed corpus as rebuildable derived data rather than requiring traditional database backups. Preserve configuration, manifests, provenance, and release artifacts; operators may still back up data for faster recovery.

## Consequences
Recovery stays simple and backup infrastructure is not a release requirement. Rebuild time and temporary unavailability are accepted tradeoffs.

## Alternatives considered
Mandatory database backup/restore was rejected because the indexed corpus is not the system of record.
