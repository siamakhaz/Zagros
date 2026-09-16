#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

echo "[zagros] waiting for Docker..."
for _ in $(seq 1 60); do
  if docker info >/dev/null 2>&1; then
    break
  fi
  sleep 2
done

docker info >/dev/null 2>&1 || {
  echo "[zagros] Docker did not become ready."
  exit 1
}

export ZAGROS_AUTO_REFRESH=false

echo "[zagros] starting HelixDB, MCP, and UI..."
docker compose -f docker/compose.yml up -d --build

echo "[zagros] waiting for MCP health..."
for _ in $(seq 1 60); do
  if curl -fsS http://127.0.0.1:8789/health >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
curl -fsS http://127.0.0.1:8789/health >/dev/null

echo "[zagros] seeding a practical live-demo corpus..."
docker compose -f docker/compose.yml --profile cli run --rm cli backfill --limit 100
docker compose -f docker/compose.yml --profile cli run --rm cli source all

echo "[zagros] verifying UI..."
for _ in $(seq 1 60); do
  if curl -fsS http://127.0.0.1:8788/api/status >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
curl -fsS http://127.0.0.1:8788/api/status

echo
echo "[zagros] live demo ready."
echo "[zagros] open the forwarded 'Zagros UI' port (8788) from the Ports panel."
echo "[zagros] CVE search: docker compose -f docker/compose.yml --profile cli run --rm cli search \"remote code execution\" --top-k 5"
echo "[zagros] knowledge search: docker compose -f docker/compose.yml --profile cli run --rm cli know \"out-of-bounds write\" --top-k 5"
