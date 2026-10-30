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

    def test_railway_volume_bootstrap_drops_privileges(self):
        entry = (ROOT / "docker/railway-volume-entrypoint.sh").read_text(encoding="utf-8")
        self.assertIn('chown -R "$service_user:$service_user" /data', entry)
        self.assertIn('exec runuser -u "$service_user" -- "$@"', entry)
        for path, user in (("Dockerfile.railway", "zagros"),
                           ("docker/Dockerfile.helix.railway", "helix")):
            content = (ROOT / path).read_text(encoding="utf-8")
            self.assertIn("util-linux", content)
            self.assertIn("docker/railway-volume-entrypoint.sh", content)
            self.assertIn(f'"{user}"', content)
        helix = (ROOT / "docker/Dockerfile.helix.railway").read_text(encoding="utf-8")
        self.assertNotIn("ca-certificates=20250419~deb12u1", helix)

    def test_cron_configuration_is_private_and_ephemeral(self):
        entry = (ROOT / "docker/mcp-entrypoint.sh").read_text(encoding="utf-8")
        self.assertIn('cron_file="$(mktemp /tmp/zagros-crontab.XXXXXX)"', entry)
        self.assertNotIn('/data/zagros-crontab', entry)
        self.assertIn('printf', entry)
        self.assertIn('/usr/local/bin/supercronic "$cron_file" &', entry)
        self.assertIn('exec /usr/local/bin/zagros-mcp-http "$@"', entry)
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
