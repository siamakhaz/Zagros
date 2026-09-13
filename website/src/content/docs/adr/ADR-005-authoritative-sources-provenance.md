# ADR-005 — Authoritative sources with explicit provenance

**Status:** Accepted
**Date:** 2026-09-13

## Context
Zagros security conclusions depend on evidence quality, freshness, and traceability.

## Decision
Prefer authoritative upstream sources and persist source identity, version, canonical and retrieval URLs, timestamps, license, content hash, and trust tier.

## Consequences
Claims remain traceable, conflicts can be surfaced explicitly, and stale or transformed evidence is easier to detect. Ingestion must preserve additional metadata.

## Alternatives considered
Convenience mirrors and undifferentiated web search were rejected as primary evidence because they weaken authority and reproducibility.
