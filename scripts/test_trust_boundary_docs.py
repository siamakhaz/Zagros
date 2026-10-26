"""Regression checks for documented remote trust boundaries (issue #49).

These are static repository checks, not live authentication verification.
"""
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class TrustBoundaryDocumentationTests(unittest.TestCase):
    def test_traefik_override_is_tls_routing_without_bundled_auth(self):
        override = (ROOT / "docker/compose.traefik.yml").read_text(encoding="utf-8")
        deployment = (ROOT / "docs/DEPLOYMENT.md").read_text(encoding="utf-8")
        self.assertIn("traefik.http.routers.zagros-mcp.tls=true", override)
        self.assertNotRegex(
            override,
            r"traefik\.http\.routers\.zagros-mcp\.middlewares\s*=",
        )
        self.assertIn("does not configure authentication or authorization", deployment)

    def test_entrypoint_warning_and_operator_proof_are_documented(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        deployment = (ROOT / "docs/DEPLOYMENT.md").read_text(encoding="utf-8")
        self.assertIn("does not add authentication or authorization", readme)
        self.assertIn("Validate the remote authentication boundary", deployment)
        self.assertIn('"method":"initialize"', deployment)
        self.assertIn("gateway logs show the rejection", deployment)
        self.assertIn("host-header validation does not prove authentication", deployment.lower())

    def test_client_approval_is_not_misrepresented_as_server_enforcement(self):
        threat = (ROOT / "docs/THREAT-MODEL.md").read_text(encoding="utf-8")
        mcp = (ROOT / "src/mcp.rs").read_text(encoding="utf-8")
        self.assertIn("does not enforce user approval", threat)
        self.assertIn("server-side cooldowns", threat)
        self.assertIn("clients should request user approval", mcp)


if __name__ == "__main__":
    unittest.main()
