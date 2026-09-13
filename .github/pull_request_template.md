## Summary

Describe the change and why it is needed.

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] Website/docs build if affected
- [ ] Docker/Compose checks if affected

## Security and data

- [ ] No secrets, private endpoints, or personal machine paths were added.
- [ ] New/changed external data has provenance, licensing, size limits, and trust classification.
- [ ] Retrieved external content remains untrusted data, not executable instructions.
- [ ] User-facing or contract changes are documented.

## Breaking changes

Describe any CLI, MCP, schema, deployment, or skill-contract compatibility impact.
