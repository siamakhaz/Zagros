# ADR-004 — Separate Zagros Core and Zagros Skills

**Status:** Accepted  
**Date:** 2026-09-13

## Context

Retrieval/storage infrastructure and agent behavior evolve at different rates and should not force a single versioning or trust boundary.

## Decision

Maintain Zagros Core and Zagros Skills as separate but compatible layers. Core provides ingestion, storage, retrieval, CLI, MCP, and UI. Skills provide harness-neutral investigation behavior and evidence-handling guidance.

## Consequences

Skills can evolve independently, work without one specific harness, and declare minimum Core compatibility. Core remains useful without a skill package.

## Alternatives considered

Embedding all behavioral instructions into the MCP server was rejected because it couples agent policy to transport/runtime implementation and reduces portability.
