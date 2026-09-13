# ADR-003 — Local-first deployment architecture

**Status:** Accepted  
**Date:** 2026-09-13

## Context

Zagros handles security evidence that may be sensitive and is intended to support organizations that do not want to send their security corpus to a hosted RAG service.

## Decision

Make local/self-hosted deployment the default. Bind HelixDB, MCP, and UI to loopback by default and require explicit operator action for LAN or public exposure.

## Consequences

The safest path is also the easiest default. Remote deployments require extra configuration, authentication, TLS, and network controls documented separately.

## Alternatives considered

A hosted-first SaaS architecture was rejected because it conflicts with local control, inspectability, and minimized data exposure.
