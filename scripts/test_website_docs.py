"""Guard website CI against unsupported or unpatched Node.js versions.

Astro 7.4 requires Node >=22.23.4. Using only "22" in setup-node
can select a stale cached 22.x patch and fail the website build.
"""

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = (
    ROOT / ".github/workflows/ci.yml",
    ROOT / ".github/workflows/pages.yml",
    ROOT / ".github/workflows/security.yml",
)
MINIMUM_NODE = (22, 23, 4)  # 2026-10-21 Node.js security release


def version_tuple(value: str):
    """Accept an explicit three-component version, not an ambiguous major pin."""
    if not re.fullmatch(r"\d+\.\d+\.\d+", value):
        return None
    return tuple(map(int, value.split(".")))


def workflow_node_version(text: str):
    versions = re.findall(
        r"(?m)^\s+node-version:\s*['\"]?([^'\"\s#]+)['\"]?\s*$", text
    )
    return version_tuple(versions[0]) if len(versions) == 1 else None


class WebsiteNodeVersionTests(unittest.TestCase):
    def test_all_node_workflows_use_supported_security_patch(self):
        lockfile = json.loads(
            (ROOT / "website/package-lock.json").read_text(encoding="utf-8")
        )
        astro_engine = lockfile["packages"]["node_modules/astro"]["engines"]["node"]
        match = re.search(r">=\s*(\d+\.\d+\.\d+)", astro_engine)
        self.assertIsNotNone(match, astro_engine)
        minimum = max(MINIMUM_NODE, version_tuple(match.group(1)))
        for path in WORKFLOWS:
            with self.subTest(workflow=path.name):
                actual = workflow_node_version(path.read_text(encoding="utf-8"))
                self.assertIsNotNone(actual, "Pin an explicit Node patch version")
                self.assertGreaterEqual(actual, minimum)

    def test_major_only_pin_is_rejected(self):
        self.assertIsNone(workflow_node_version("    node-version: 22\n"))

    def test_previous_patch_is_rejected(self):
        old = workflow_node_version("    node-version: '22.23.3'\n")
        self.assertLess(old, MINIMUM_NODE)

    def test_supported_patch_is_accepted(self):
        self.assertEqual(
            workflow_node_version("    node-version: '22.23.4'\n"), MINIMUM_NODE
        )


if __name__ == "__main__":
    unittest.main()
