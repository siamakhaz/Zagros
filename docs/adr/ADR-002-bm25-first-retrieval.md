# ADR-002 — BM25-first retrieval before vector/graph search

**Status:** Accepted  
**Date:** 2026-09-13

## Context

Security queries often contain exact identifiers, product names, control IDs, and technical phrases where deterministic lexical matching is valuable.

## Decision

Use BM25 with explicit field weights, exact-match bonuses, phrase bonuses, and a minimum relevance threshold as the first production retrieval method. Add vector and graph retrieval later behind measured regression benchmarks.

## Consequences

Current retrieval is transparent, deterministic, fast, and easy to test. Semantic recall remains limited until hybrid retrieval is introduced.

## Alternatives considered

Embedding-first retrieval and graph-only retrieval were deferred because they add complexity and weaker explainability before a quality baseline exists.
