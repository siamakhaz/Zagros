# Contributing to Zagros

Thank you for contributing to Zagros.

## Development flow

1. Create a focused branch from `main`.
2. Keep changes small and explain security/data-source implications.
3. Run:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-targets
   cd website && npm ci && npm run build
   ```
4. Update documentation for user-visible behavior.
5. Open a pull request using the repository template.

Security vulnerabilities should be reported privately according to `SECURITY.md`, not in public issues.
