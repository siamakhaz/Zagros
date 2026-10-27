"""Static checks for local/remote developer onboarding endpoint guidance.

These checks validate documentation against checked-in routes. They do not
replace the clean-client walkthrough required by issue #32.
"""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GUIDE = (ROOT / "docs/ONBOARD-DEVELOPER.md").read_text(encoding="utf-8")
SERVER = (ROOT / "src/bin/zagros-mcp-http.rs").read_text(encoding="utf-8")


class DeveloperOnboardingEndpointTests(unittest.TestCase):
    def test_documented_paths_exist_in_native_http_server(self):
        self.assertIn('.route("/health", axum::routing::get(health))', SERVER)
        self.assertIn('"/mcp",', SERVER)
        self.assertIn('GET /health', GUIDE)
        self.assertIn('/mcp', GUIDE)

    def test_health_probe_uses_selected_base_url(self):
        self.assertIn('$ZagrosBaseUrl = "http://127.0.0.1:8789"', GUIDE)
        self.assertIn('Invoke-RestMethod -Uri "$ZagrosBaseUrl/health"', GUIDE)
        self.assertNotIn('Invoke-RestMethod http://127.0.0.1:8789/health', GUIDE)

    def test_client_configuration_distinguishes_local_and_remote(self):
        self.assertIn('"url": "http://127.0.0.1:8789/mcp"', GUIDE)
        self.assertIn('https://zagros.example.org/mcp', GUIDE)
        self.assertIn('illustrative only', GUIDE)
        self.assertIn("not the client's `localhost`", GUIDE)

    def test_remote_access_keeps_authentication_boundary(self):
        self.assertIn('gateway credentials', GUIDE)
        self.assertIn('do not bypass the gateway', GUIDE)
        self.assertIn('does **not** provide application-level user authentication', GUIDE)


if __name__ == "__main__":
    unittest.main()
