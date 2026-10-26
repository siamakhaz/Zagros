"""Regression checks for the optional Windows Toolkit setup and its CLI guidance.

These are static contract checks, not a claim that Docker Desktop or a clean
Windows machine was exercised. See issue #32 for the remaining live walkthrough.
"""
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SETUP = (ROOT / "scripts/setup.ps1").read_text(encoding="utf-8")
CONFIG = (ROOT / "docs/CONFIGURATION.md").read_text(encoding="utf-8")
CLI = (ROOT / "src/main.rs").read_text(encoding="utf-8")
MCP = (ROOT / "src/bin/zagros-mcp.rs").read_text(encoding="utf-8")
DOCKERFILE = (ROOT / "Dockerfile").read_text(encoding="utf-8")


class ToolkitSetupDocsTests(unittest.TestCase):
    def test_seed_uses_cli_binary_not_stdio_transport(self):
        self.assertIn("Commands::Backfill", CLI)
        self.assertNotIn("Commands::Backfill", MCP)
        self.assertRegex(SETUP, r'\$cliBinary\s*=\s*Join-Path \$repoRoot "target" "release" "zagros.exe"')
        self.assertRegex(SETUP, r'\$mcpBinary\s*=\s*Join-Path \$repoRoot "target" "release" "zagros-mcp.exe"')
        self.assertIn("& $cliBinary backfill --limit $SeedLimit", SETUP)
        self.assertNotIn("& $mcpBinary backfill", SETUP)
        self.assertIn('Fail "backfill failed with exit code $LASTEXITCODE."', SETUP)

    def test_setup_builds_and_checks_both_binaries(self):
        self.assertIn("cargo build --release --locked --bin zagros --bin zagros-mcp", SETUP)
        self.assertIn("@($mcpBinary, $cliBinary)", SETUP)
        for command in ("status", "ingest", "search"):
            with self.subTest(command=command):
                self.assertRegex(SETUP, rf"\$cliBinary {command}\b")

    def test_documented_toolkit_and_container_roles(self):
        self.assertIn("optional Docker Desktop MCP Toolkit", CONFIG)
        self.assertIn("zagros.exe backfill --limit $SeedLimit", CONFIG)
        self.assertNotIn("zagros-mcp backfill --limit $SeedLimit", CONFIG)
        for binary in ("zagros", "zagros-mcp", "zagros-mcp-http"):
            with self.subTest(binary=binary):
                self.assertRegex(DOCKERFILE, rf"COPY --from=builder[^\n]*/{re.escape(binary)}\s+/usr/local/bin/{re.escape(binary)}\b")


if __name__ == "__main__":
    unittest.main()
