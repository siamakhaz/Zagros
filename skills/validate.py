#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKILLS = ROOT / "skills"
SKILL_DIR = SKILLS / ".apm" / "skills" / "cybersecurity-expert"
APM = SKILLS / "apm.yml"
META = SKILLS / "skill-meta.yml"
EVALS = SKILL_DIR / "evals" / "evals.json"
MCP_DOC = ROOT / "docs" / "MCP.md"

REQUIRED_FILES = [
    APM,
    META,
    SKILL_DIR / "SKILL.md",
    SKILL_DIR / "LICENSE",
    EVALS,
]
SEMVER = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$"
)


def fail(message: str, errors: list[str]) -> None:
    errors.append(message)


def yaml_scalar(text: str, key: str) -> str | None:
    match = re.search(rf"(?m)^\s*{re.escape(key)}:\s*[\"']?([^\"'\n#]+)", text)
    return match.group(1).strip() if match else None


def inline_list(text: str, key: str) -> list[str]:
    match = re.search(rf"(?m)^\s*{re.escape(key)}:\s*\[([^\]]*)\]", text)
    if not match:
        return []
    return [item.strip().strip("\"'") for item in match.group(1).split(",") if item.strip()]


def main() -> int:
    errors: list[str] = []

    for path in REQUIRED_FILES:
        if not path.is_file():
            fail(f"missing required file: {path.relative_to(ROOT)}", errors)

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    apm_text = APM.read_text(encoding="utf-8-sig")
    meta_text = META.read_text(encoding="utf-8-sig")
    mcp_text = MCP_DOC.read_text(encoding="utf-8-sig")

    version = yaml_scalar(apm_text, "version")
    if not version or not SEMVER.fullmatch(version):
        fail("skills/apm.yml version must be valid SemVer", errors)

    for key in ("name", "description", "author", "license", "type"):
        if not yaml_scalar(apm_text, key):
            fail(f"skills/apm.yml missing required metadata: {key}", errors)

    contract_version = yaml_scalar(meta_text, "contract_version")
    if contract_version != "1":
        fail("skills/skill-meta.yml contract_version must equal 1", errors)

    for key in ("owner", "status"):
        if not yaml_scalar(meta_text, key):
            fail(f"skills/skill-meta.yml missing required metadata: {key}", errors)

    minimum_core = yaml_scalar(meta_text, "minimum_zagros_core")
    if not minimum_core or not SEMVER.fullmatch(minimum_core):
        fail("minimum_zagros_core must be valid SemVer", errors)

    required_tools = inline_list(meta_text, "required_mcp_tools")
    if not required_tools:
        fail("required_mcp_tools must declare at least one tool", errors)
    for tool in required_tools:
        if not re.search(rf"(?<![A-Za-z0-9_]){re.escape(tool)}(?![A-Za-z0-9_])", mcp_text):
            fail(f"required MCP tool is not documented in docs/MCP.md: {tool}", errors)

    try:
        payload = json.loads(EVALS.read_text(encoding="utf-8-sig"))
    except json.JSONDecodeError as exc:
        fail(f"eval JSON is invalid: {exc}", errors)
        payload = {}

    evals = payload.get("evals")
    if not isinstance(evals, list) or not evals:
        fail("evals.json must contain a non-empty evals array", errors)
        evals = []

    seen: set[object] = set()
    for index, case in enumerate(evals, start=1):
        if not isinstance(case, dict):
            fail(f"eval #{index} must be an object", errors)
            continue
        case_id = case.get("id")
        if case_id is None:
            fail(f"eval #{index} missing id", errors)
        elif case_id in seen:
            fail(f"duplicate eval id: {case_id}", errors)
        else:
            seen.add(case_id)
        if not isinstance(case.get("prompt"), str) or not case["prompt"].strip():
            fail(f"eval {case_id!r} missing prompt", errors)
        if not isinstance(case.get("expected_output"), str) or not case["expected_output"].strip():
            fail(f"eval {case_id!r} missing expected_output", errors)

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    print(
        f"Skill Contract v1 validation passed: version={version}, "
        f"evals={len(evals)}, required_mcp_tools={len(required_tools)}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
