#!/usr/bin/env python3
"""Build the standalone zagros-skill distributable.

Reads the version from skills/apm.yml, packs
.apm/skills/cybersecurity-expert (+ META.json) into
dist/zagros-skill.tar.gz, and writes dist/zagros-skill.tar.gz.sha256.

Usage:
    python build-dist.py            # from skills/
Publish dist/* as versioned file attachments (e.g. a GitHub Release
named zagros-skill-v<version>), e.g.:
    https://github.com/<org>/zagros/releases/download/zagros-skill-v0.1.0/{install.sh,install.ps1,zagros-skill.tar.gz,zagros-skill.tar.gz.sha256}
"""
import datetime
import gzip
import hashlib
import io
import json
import os
import sys
import tarfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
PKG = HERE / ".apm" / "skills" / "cybersecurity-expert"
SKILL = "cybersecurity-expert"
DIST = HERE / "dist"
TARBALL = DIST / "zagros-skill.tar.gz"
DEFAULT_MCP_URL = "http://localhost:8789/mcp"


def source_date_epoch() -> int:
    raw = os.environ.get("SOURCE_DATE_EPOCH")
    return int(raw) if raw else int(datetime.datetime.now(datetime.timezone.utc).timestamp())


def read_version() -> str:
    for line in (HERE / "apm.yml").read_text(encoding="utf-8").splitlines():
        if line.startswith("version:"):
            return line.split(":", 1)[1].strip().strip("\"'")
    raise SystemExit("version: not found in apm.yml")


def mcp_url() -> str:
    return os.environ.get("ZAGROS_MCP_URL") or os.environ.get(
        "CCSKILL_MCP_URL", DEFAULT_MCP_URL
    )


def main() -> None:
    if not (PKG / "SKILL.md").exists():
        raise SystemExit(f"skill source missing: {PKG}")
    version = read_version()
    epoch = source_date_epoch()
    built_at = datetime.datetime.fromtimestamp(epoch, datetime.timezone.utc).isoformat(timespec="seconds")
    DIST.mkdir(exist_ok=True)
    meta = {
        "name": "zagros-skill",
        "skill": SKILL,
        "version": version,
        "built_at": built_at,
        "mcp": {"name": "Zagros", "url": mcp_url()},
        "provenance": "Independently authored cybersecurity study notes; ISC2 CC influenced topic coverage, but no official ISC2 courseware or exam content is included",
        "layout": f"{SKILL}/SKILL.md (+ evals/, references/) at tarball root",
    }
    files = sorted(p for p in PKG.rglob("*") if p.is_file())
    with open(TARBALL, "wb") as raw_out:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw_out, mtime=epoch) as gz:
            with tarfile.open(fileobj=gz, mode="w", format=tarfile.PAX_FORMAT) as tar:
                for src in files:
                    rel = src.relative_to(PKG.parent)
                    info = tar.gettarinfo(str(src), arcname=str(rel).replace("\\", "/"))
                    info.uid = info.gid = 0
                    info.uname = info.gname = ""
                    info.mtime = epoch
                    with open(src, "rb") as f:
                        tar.addfile(info, f)
                blob = json.dumps(meta, indent=2, sort_keys=True).encode() + b"\n"
                info = tarfile.TarInfo("META.json")
                info.size = len(blob)
                info.mtime = epoch
                info.uid = info.gid = 0
                info.uname = info.gname = ""
                tar.addfile(info, io.BytesIO(blob))
    digest = hashlib.sha256(TARBALL.read_bytes()).hexdigest()
    (DIST / (TARBALL.name + ".sha256")).write_text(f"{digest}  {TARBALL.name}\n", encoding="utf-8")
    print(f"version : {version}")
    print(f"files   : {len(files)} skill files + META.json")
    print(f"tarball : {TARBALL} ({TARBALL.stat().st_size} bytes)")
    print(f"sha256  : {digest}")


if __name__ == "__main__":
    sys.exit(main())
