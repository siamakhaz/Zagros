"""Static regression checks for the opt-in Railway configuration (issue #97)."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


class RailwayDeploymentTests(unittest.TestCase):
    def test_railway_images_do_not_declare_docker_volumes(self):
        for path in ("Dockerfile.railway", "docker/Dockerfile.helix.railway"):
            data = (ROOT / path).read_text(encoding="utf-8")
            self.assertIn("USER ", data)
            self.assertNotRegex(data, r"(?im)^\s*VOLUME\s")
            self.assertIn("/data", data)

    def test_railway_images_run_nonroot_by_default(self):
        for path, user in (("Dockerfile.railway", "zagros"),
                           ("docker/Dockerfile.helix.railway", "helix")):
            content = (ROOT / path).read_text(encoding="utf-8")
            last_user = re.findall(r"(?m)^USER\s+(\S+)\s*$", content)
            self.assertEqual(last_user[-1], user)
            self.assertNotIn("railway-volume-entrypoint", content)
            self.assertNotIn("USER root", content)
        helix = (ROOT / "docker/Dockerfile.helix.railway").read_text(encoding="utf-8")
        self.assertNotIn("ca-certificates=20250419~deb12u1", helix)

    def test_persistent_volume_ids_are_explicit(self):
        mcp = (ROOT / "Dockerfile.railway").read_text(encoding="utf-8")
        helix = (ROOT / "docker/Dockerfile.helix.railway").read_text(encoding="utf-8")
        guide = (ROOT / "docs/RAILWAY.md").read_text(encoding="utf-8")
        self.assertIn("groupadd --system --gid 10001 zagros", mcp)
        self.assertIn("useradd --system --create-home --uid 10001 --gid zagros zagros", mcp)
        self.assertIn("groupadd --system --gid 10002 helix", helix)
        self.assertIn("useradd --system --uid 10002 --gid helix", helix)
        self.assertIn("UID/GID 10001", guide)
        self.assertIn("UID/GID 10002", guide)
        self.assertIn("numeric ownership", guide)

    def test_docker_compose_storage_is_preserved(self):
        compose = (ROOT / "docker/compose.yml").read_text(encoding="utf-8")
        self.assertIn("zagros-data:/data", compose)
        self.assertIn("target: /data", compose)

    def test_private_services_and_distinct_volumes(self):
        config = (ROOT / ".railway/railway.ts").read_text(encoding="utf-8")
        for name in ("zagros-helix", "zagros-mcp"):
            self.assertIn(f'service("{name}"', config)
        self.assertIn("Dockerfile.railway", config)
        self.assertIn("docker/Dockerfile.helix.railway", config)
        self.assertIn("zagros-helix.railway.internal:8080", config)
        self.assertEqual(config.count('volumeMounts: { "/data":'), 2)
        self.assertNotIn("domains:", config)
        self.assertNotIn("RAILWAY_RUN_UID", config)

    def test_operator_risk_gates(self):
        guide = (ROOT / "docs/RAILWAY.md").read_text(encoding="utf-8")
        for term in ("railway config plan", "RAILWAY_RUN_UID=0",
                     "authentication", "unverified", "persistent",
                     "Do not create public domains."):
            self.assertIn(term, guide)


if __name__ == "__main__":
    unittest.main()
