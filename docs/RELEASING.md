# Releasing Zagros

Zagros uses Semantic Versioning for tagged releases.

## Version policy

- **PATCH**: backwards-compatible fixes, documentation corrections, dependency updates, and internal changes.
- **MINOR**: backwards-compatible features, new sources, new MCP tools, or additive schema/API changes.
- **MAJOR**: breaking CLI, MCP, persisted-schema, deployment, or stable skill-contract changes.

Before 1.0, breaking changes may occur in a minor release, but they must be called out in the changelog and release notes.

The version in `Cargo.toml` is the Zagros Core release version. Release tags must exactly match it as `vMAJOR.MINOR.PATCH`.
## Release checklist

1. Update `CHANGELOG.md` and move relevant entries from **Unreleased** into the new version.
2. Update `Cargo.toml` and `Cargo.lock`.
3. Ensure the skill version in `skills/apm.yml` is intentionally set; skills may version independently later.
4. Merge with CI/security workflows green.
5. Create and push the exact version tag.
6. The release workflow rebuilds binaries, containers, and the skill from the tagged commit.
7. Verify GitHub Release checksums, container digests, SBOM/provenance attestations, and skill checksum.

Example:

```bash
git tag v0.2.0
git push origin v0.2.0
```

Do not manually upload locally built binaries as official release assets.
## Release outputs

A release produces:

- Linux x86_64 archive containing Zagros binaries.
- Windows x86_64 archive containing Zagros binaries.
- `zagros-skill.tar.gz` and its SHA-256 file.
- `SHA256SUMS` for release archives.
- Versioned MCP, CLI, and UI images in GitHub Container Registry.
- Immutable image digest files.
- Build provenance and SBOM metadata from BuildKit.
- Downloadable `zagros-source.spdx.json` release SBOM.

Production documentation should prefer an exact version or immutable digest over `:latest`.

## Reproducibility

The skill builder honors `SOURCE_DATE_EPOCH`; release CI sets it to the tagged commit timestamp. Rebuilding the same skill source with the same epoch and MCP URL must produce the same SHA-256 digest.

## Immutable-digest deployment example

For production, resolve the published digest from the GitHub Release digest file and run the exact image rather than a mutable tag:

```bash
docker pull ghcr.io/OWNER/zagros-mcp@sha256:DIGEST
docker run --rm -p 127.0.0.1:8789:8789 \
  ghcr.io/OWNER/zagros-mcp@sha256:DIGEST
```

Replace `OWNER` and `DIGEST` with the values published by the release workflow. The version tag remains convenient for discovery; the digest is the immutable deployment reference.

