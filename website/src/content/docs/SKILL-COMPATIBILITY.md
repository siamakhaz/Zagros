# Skill Versioning and Compatibility

The Zagros skill package is versioned independently from Zagros Core.

Current package:
- Skill package: `zagros-skill 0.1.0`
- Skill Contract: `1`
- Minimum Zagros Core: `0.1.0`
- Lifecycle: beta

## Independent SemVer

The skill package version comes from `skills/apm.yml` and follows Semantic Versioning.

- PATCH: wording, examples, eval additions, or fixes that do not change required behavior.
- MINOR: backward-compatible capabilities, references, metadata, or optional integrations.
- MAJOR: breaking behavioral, package-layout, metadata, or installation changes.

The skill version does not need to match the Zagros Core version. Compatibility is declared explicitly in `skills/skill-meta.yml`.

## Core and skill compatibility

`minimum_zagros_core` is the oldest Core version expected to provide the MCP behavior required by this skill.
`required_mcp_tools` lists the MCP tools the skill expects when Zagros-backed retrieval is enabled.

Before upgrading either side:
1. Check this file and `skill-meta.yml`.
2. Confirm the installed Core meets `minimum_zagros_core`.
3. Confirm the required tools still exist in `docs/MCP.md`.
4. Run `python skills/validate.py`.
5. Re-run the skill evals after any behavioral or tool-contract change.

A newer Core may be used unless its release notes declare a breaking MCP change.

## Upgrade procedure

1. Record the currently installed skill version and any local harness configuration.
2. Obtain the new released package and verify its published SHA256 checksum.
3. Review `CHANGELOG.md`, this compatibility file, and the package metadata.
4. Replace the installed skill package using the harness/APM installation mechanism.
5. Preserve the user-selected Zagros MCP URL; do not silently replace it.
6. Restart or reload the harness.
7. Verify Zagros connectivity and required MCP tools.
8. Run representative evals before relying on the upgraded skill for production investigations.

If the new skill requires a newer Core, upgrade Core first.

## Uninstall procedure

1. Remove the installed `cybersecurity-expert` skill from the harness/APM-managed location.
2. Remove the Zagros MCP entry only if it is not shared by other skills or workflows.
3. Remove package-specific cached files created by the harness, if any.
4. Do not delete Zagros Core data or configuration unless the user explicitly intends to uninstall Core too.
5. Verify the harness no longer advertises the skill.

The standalone installers may create `opencode.json.bak`; retain or remove that backup according to local policy.

## Microsoft APM compatibility

The package is structured as an APM-style skill package with `apm.yml`, `skill-meta.yml`, and content under `.apm/skills/`.
The package metadata remains the source of truth for package identity, version, targets, and MCP dependency declaration.

APM implementations and harness integrations can differ in command syntax. Therefore Zagros does not hard-code a single vendor CLI command as part of the Skill Contract. An APM-compatible installer must preserve the package layout, install the declared skill, and expose the declared MCP dependency without changing the skill's behavioral contract.

For environments that do not consume APM packages directly, use the harness-neutral examples in `skills/HARNESSES.md` or the released standalone installers.

## Compatibility matrix

| Skill package | Contract | Minimum Core | Required Zagros MCP tools |
|---|---:|---:|---|
| 0.1.x | 1 | 0.1.0 | `search_cves`, `get_cve`, `search_knowledge`, `index_status` |

Write-capable tools such as `sync_cves`, `backfill_cves`, and `sync_knowledge_source` are optional and always remain approval-gated.
