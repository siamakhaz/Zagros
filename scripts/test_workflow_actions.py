"""Keep the CodeQL workflow compatible across independently pinned action steps."""

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/security.yml"
DEPENDABOT = ROOT / ".github/dependabot.yml"

CODEQL_STEP = re.compile(
    r"^\s*- uses: github/codeql-action/(init|autobuild|analyze)@([0-9a-f]{40})\b",
    re.MULTILINE,
)


def validate_codeql_pins(workflow: str) -> bool:
    """Return whether all three CodeQL steps exist once and share one commit."""
    matches = CODEQL_STEP.findall(workflow)
    return (
        sorted(step for step, _ in matches) == ["analyze", "autobuild", "init"]
        and len({sha for _, sha in matches}) == 1
    )


class CodeQLWorkflowTests(unittest.TestCase):
    def test_current_workflow_has_consistent_pins(self):
        self.assertTrue(validate_codeql_pins(WORKFLOW.read_text(encoding="utf-8")))

    def test_mismatched_version_is_rejected(self):
        good = "\n".join(
            f"- uses: github/codeql-action/{step}@{'a' * 40}"
            for step in ("init", "autobuild", "analyze")
        )
        bad = good.replace("autobuild@" + "a" * 40, "autobuild@" + "b" * 40)
        self.assertTrue(validate_codeql_pins(good))
        self.assertFalse(validate_codeql_pins(bad))

    def test_missing_step_is_rejected(self):
        incomplete = "\n".join(
            f"- uses: github/codeql-action/{step}@{'a' * 40}"
            for step in ("init", "analyze")
        )
        self.assertFalse(validate_codeql_pins(incomplete))

    def test_dependabot_groups_codeql_steps(self):
        config = DEPENDABOT.read_text(encoding="utf-8")
        self.assertIn("codeql-action:", config)
        self.assertIn('github/codeql-action/*', config)


if __name__ == "__main__":
    unittest.main()
