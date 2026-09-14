# Zagros Roadmap

## Current — Public open-source baseline

- Public repository with sanitized reachable history.
- Trusted-source provenance and source integrity.
- CI, deterministic E2E, CodeQL/Trivy/dependency scanning, and OpenSSF Scorecard.
- Secret scanning, push protection, Private Vulnerability Reporting, CODEOWNERS, and protected `main`.
- Stable Core + Skills documentation and deterministic retrieval regression benchmarks.
- Before the first tagged release: complete one clean fresh-clone Docker acceptance test.

## Next — Graph + vector retrieval

- Graph relationships between CVE, CWE, CAPEC, ATT&CK and controls.
- Embeddings/vector search and hybrid retrieval.
- Reciprocal Rank Fusion across lexical, vector, and graph retrieval.
- Exact/evidence-chain tooling such as `get_rule`, `explain_weakness`, `map_to_attack`, and source-status queries.
- Expand retrieval evaluation with larger real-world corpora and source-diverse relevance cases.
- Add additional authoritative sources where licensing and provenance requirements are satisfied.

## Later

- AST/IaC/SBOM ingestion.
- Assisted security reasoning with explicit evidence boundaries.
- Additional independently maintained Zagros Skills.

Roadmap items are directional, not commitments. Experimental capabilities must be labeled until their contracts are stable.
