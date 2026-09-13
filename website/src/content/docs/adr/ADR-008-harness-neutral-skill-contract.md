# ADR-008 — Harness-neutral Skill Contract

**Status:** Accepted
**Date:** 2026-09-13

## Context

Zagros Skills must remain portable across agent clients.

## Decision

Keep generic skill behavior harness-neutral. Store Zagros MCP mappings and client-specific setup in separate references.

## Consequences

Skill behavior can be versioned independently and reused across clients without duplicating core guidance.

## Alternatives considered

Per-client behavioral variants were rejected because they would duplicate guidance and drift over time.
