# Skill Trust and Review Policy

Zagros does not treat a contributed skill as trusted merely because it is installable.

## Trust states

- **maintained** — reviewed and maintained by the Zagros project.
- **community-reviewed** — third-party contribution that passed contract, provenance, security, and evaluation review.
- **experimental** — useful but not yet covered by a stable compatibility promise.
- **rejected/quarantined** — fails provenance, safety, licensing, or behavior requirements.

## Review requirements

A contributed skill must pass:

1. **Provenance and license review** — authorship and third-party material are identified; redistribution terms are compatible.
2. **Security review** — no credential harvesting, hidden network destinations, unsafe shell instructions, or unapproved writes.
3. **Contract validation** — required metadata/files and contract version are valid.
4. **Behavior review** — evals demonstrate intended behavior and important refusal/approval boundaries.
5. **Integration review** — declared MCP/tool dependencies match the actual references and compatibility metadata.

New remote MCP endpoints are security-sensitive and require explicit review of authentication, data handling, write capabilities, and returned-content trust boundaries.

## Publication

Community skills are not promoted to maintained status automatically. Trust status must be explicit in `skill-meta.yml`. Material behavior or dependency changes require renewed review.
