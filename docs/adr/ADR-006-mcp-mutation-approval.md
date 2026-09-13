# ADR-006 — Approval-gated MCP mutation and network tools

**Status:** Accepted
**Date:** 2026-09-13

## Context
Search and lookup are read-only, while synchronization and backfill operations access external networks and modify persistent data.

## Decision
Keep read/search tools distinct from mutation/network tools and require explicit user approval before synchronization, backfill, or source refresh.

## Consequences
Investigations remain safe by default while controlled updates are still possible. Clients must preserve this approval boundary.

## Alternatives considered
Automatic refresh during investigation was rejected because it creates hidden side effects and changes evidence state.
