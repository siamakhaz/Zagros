# ADR-001 — HelixDB as the primary storage backend

**Status:** Accepted  
**Date:** 2026-09-13

## Context

Zagros needs local storage for CVE and security-knowledge records, future graph relationships, and later vector retrieval while remaining self-hostable.

## Decision

Use HelixDB as the primary persistent store for Zagros Core. Keep retrieval logic in Zagros rather than treating the database as the sole ranking engine.

## Consequences

Zagros gains a single local store suitable for typed nodes and future graph/vector work. The project accepts current HelixDB limitations, including the documented non-atomic delete-then-insert upsert window and lack of built-in authentication.

## Alternatives considered

SQLite/PostgreSQL were simpler relational options, while dedicated vector databases offered mature embedding search. They were not chosen because Zagros is intentionally targeting graph + vector evolution in one local-first store.
