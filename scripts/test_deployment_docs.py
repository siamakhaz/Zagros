"""Guard against deployment documentation drifting from shipped Compose and Toolkit files."""
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONFIG = (ROOT / "docs/CONFIGURATION.md").read_text(encoding="utf-8")
MCP = (ROOT / "docs/MCP.md").read_text(encoding="utf-8")
COMPOSE = (ROOT / "docker/compose.yml").read_text(encoding="utf-8")
WINDOWS_GATEWAY = (ROOT / "docs/DOCKER-MCP-WINDOWS.md").read_text(encoding="utf-8")
LINUX_GATEWAY = (ROOT / "docs/DOCKER-MCP-LINUX.md").read_text(encoding="utf-8")


def toolkit_hosts(path: str) -> set[str]:
    yaml = (ROOT / path).read_text(encoding="utf-8")
    section = yaml.split("allowHosts:\n", 1)[1]
    hosts = []
    for line in section.splitlines():
        match = re.fullmatch(r"  - ([^\s#]+)", line)
        if not match:
            break
        hosts.append(match.group(1))
    return set(hosts)


class DeploymentDocsTests(unittest.TestCase):
    def test_compose_files_mentioned_in_config_exist(self):
        paths = set(re.findall(r"docker/compose(?:\.[\w-]+)?\.yml", CONFIG))
        self.assertIn("docker/compose.yml", paths)
        for path in paths:
            with self.subTest(path=path):
                self.assertTrue((ROOT / path).is_file(), path)

    def test_helix_image_and_storage_match_compose(self):
        helix = COMPOSE.split("\n  helix:\n", 1)[1].split("\n  mcp:\n", 1)[0]
        image = re.search(r"^    image: ([^\s#]+)", helix, re.MULTILINE)
        self.assertIsNotNone(image)
        self.assertIn(f"`{image.group(1)}`", CONFIG)
        self.assertIn("source: ${ZAGROS_DATA_DIR:-../data}", helix)
        self.assertIn("host bind-mounted", CONFIG)
        self.assertIn("mem_limit: 512m", helix)
        self.assertIn("`512m`", CONFIG)

    def test_native_mcp_and_optional_gateway_are_distinguished(self):
        self.assertIn("Neither binary requires Docker MCP Gateway", MCP)
        self.assertIn("supported Compose deployment runs the HTTP server directly", MCP)
        for guide in (WINDOWS_GATEWAY, LINUX_GATEWAY):
            self.assertIn("optional", guide.lower())
            self.assertIn("ONBOARD-ADMIN.md", guide)
            self.assertIn("ONBOARD-DEVELOPER.md", guide)
        self.assertIn("Unsupported for the Linux-based Zagros container image", WINDOWS_GATEWAY)
        self.assertNotIn("The `zagros-mcp` server in this project uses Docker MCP", WINDOWS_GATEWAY)

    def test_toolkit_hosts_are_documented(self):
        for path in ("docker/server.yaml", "docker/zagros-server.yaml"):
            with self.subTest(path=path):
                hosts = toolkit_hosts(path)
                self.assertTrue(hosts)
                for host in hosts:
                    self.assertIn(f"`{host}`", CONFIG)
                    self.assertIn(host, MCP)


if __name__ == "__main__":
    unittest.main()
