"""CLI reference contracts checked against the current Rust entry point.

These tests do not require network access, HelixDB, or compiled binaries.
"""
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = (ROOT / "docs" / "CLI.md").read_text(encoding="utf-8")
MAIN = (ROOT / "src" / "main.rs").read_text(encoding="utf-8")


def section(command: str) -> str:
    return DOC.split(f"## `{command}`", 1)[1].split("\n## ", 1)[0]


class CliDocumentationContractTests(unittest.TestCase):
    def test_ingest_limit_matches_unbounded_usize_argument(self):
        self.assertRegex(
            MAIN,
            r"Ingest\s*\{\s*#\[arg\(long, default_value_t = 50\)\]\s*limit: usize",
        )
        self.assertIn("Non-negative integer (no CLI maximum)", section("ingest"))
        self.assertNotIn("1–1000", section("ingest"))

    def test_source_all_output_uses_real_progress_messages(self):
        source = section("source")
        self.assertIn('for src in ["cwe", "asvs", "capec", "attack"]', MAIN)
        for label, noun in [
            ("CWE", "weaknesses"),
            ("ASVS", "requirements"),
            ("CAPEC", "attack patterns"),
            ("ATT&CK", "techniques"),
        ]:
            self.assertIn(
                f'println!("{label}: {{n}} {noun} stored; {{total}} total knowledge nodes")',
                MAIN,
            )
            self.assertIn(
                f"{label}: <count> {noun} stored; <cumulative total> total knowledge nodes",
                source,
            )
        self.assertNotRegex(source, r"\| ~?\d{3,4} \|")

    def test_status_examples_reflect_empty_and_populated_index(self):
        status = section("status")
        self.assertIn('println!("  {src:<12}: {n}")', MAIN)
        self.assertIn("CVE nodes    : <count>", status)
        self.assertIn("  total       : <count>", status)
        self.assertIn("Knowledge    : (none — run 'source all' to populate)", status)
        self.assertNotIn("Knowledge nodes by source:", status)

    def test_interactive_prompt_and_result_format(self):
        interactive = section("interactive")
        self.assertIn('print!("cve> ")', MAIN)
        self.assertIn('"{}. {} | {:.3}"', MAIN)
        self.assertIn("cve> :limit 5", interactive)
        self.assertNotIn("| updated", interactive)

    def test_usage_error_is_distinguished_from_runtime_error(self):
        self.assertIn("derive(Parser)", MAIN)
        self.assertIn("| 1 | Application error", DOC)
        self.assertIn("| 2 | CLI argument/usage error reported by Clap", DOC)


if __name__ == "__main__":
    unittest.main()
