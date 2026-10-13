"""Regression test: a broken local Markdown link must fail."""
import tempfile
import unittest
from pathlib import Path
from check_docs import check

class DocumentationValidationTests(unittest.TestCase):
    def test_missing_link_fails(self):
        with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parents[1]) as folder:
            doc = Path(folder) / "test.md"
            doc.write_text("[missing](does-not-exist.md)\n", encoding="utf-8")
            self.assertTrue(any("missing referenced file" in error for error in check(doc)))

    def test_code_examples_are_not_checked_as_links(self):
        with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parents[1]) as folder:
            doc = Path(folder) / "test.md"
            doc.write_text("```md\n[example](missing.md)\n```\n", encoding="utf-8")
            self.assertEqual(check(doc), [])

if __name__ == "__main__":
    unittest.main()
