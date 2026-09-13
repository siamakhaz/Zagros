#!/usr/bin/env bash
# zagros-skill standalone installer (Linux / macOS / WSL / git-bash).
# Installs the cybersecurity-expert skill + Zagros MCP wiring. No APM required.
#
# Quick install (self-hosted Zagros at http://localhost:8789/mcp):
#   curl -fsSL https://github.com/<org>/zagros/releases/download/zagros-skill-v0.1.0/install.sh | bash
# With options (note the `-s --` separator):
#   curl -fsSL <release>/install.sh | bash -s -- --global
#   curl -fsSL <release>/install.sh | bash -s -- --dir /path/to/project --force
#   curl -fsSL <release>/install.sh | bash -s -- --mcp-url https://mcp.example.com/mcp
#
# Server layout expected under $BASE_URL:
#   install.sh  install.ps1  zagros-skill.tar.gz  zagros-skill.tar.gz.sha256
#
# Options:
#   --dir PATH       project directory to install into (default: current directory)
#   --global         install to user scope (~/.agents/skills + ~/.config/opencode/opencode.json)
#   --url URL        full tarball URL or local file path (default: $BASE_URL/zagros-skill.tar.gz)
#   --mcp-url URL    Zagros MCP endpoint (default: http://localhost:8789/mcp)
#   --mcp-name NAME  MCP entry name (default: Zagros)
#   --sha256 HASH    expected tarball hash (default: fetched from <tarball>.sha256 for http(s))
#   --skip-verify    skip hash verification (not recommended for http(s))
#   --force          overwrite an existing Zagros MCP entry (default: keep existing)
#   -h, --help       show this help
set -euo pipefail

# ------------------------------------------------------ EDIT ME (fork) ----
# Point at your published Release assets. Env override: ZAGROS_SKILL_BASE_URL
# (legacy CCSKILL_BASE_URL still honoured).
BASE_URL="${ZAGROS_SKILL_BASE_URL:-${CCSKILL_BASE_URL:-https://github.com/<org>/zagros/releases/download/zagros-skill-v0.1.0}}"
# ----------------------------------------------------------------------------
TARBALL="zagros-skill.tar.gz"
SKILL="cybersecurity-expert"
MCP_NAME="${ZAGROS_MCP_NAME:-${CCSKILL_MCP_NAME:-Zagros}}"
MCP_URL="${ZAGROS_MCP_URL:-${CCSKILL_MCP_URL:-http://localhost:8789/mcp}}"

DIR="$PWD"
GLOBAL=0
URL=""
EXPECT_SHA=""
SKIP_VERIFY=0
FORCE=0

usage() { sed -n '2,/^set -euo/p' "$0" | sed 's/^# \{0,1\}//'; }

while [ $# -gt 0 ]; do
  case "$1" in
    --dir) DIR="$2"; shift 2 ;;
    --global) GLOBAL=1; shift ;;
    --url) URL="$2"; shift 2 ;;
    --mcp-url) MCP_URL="$2"; shift 2 ;;
    --mcp-name) MCP_NAME="$2"; shift 2 ;;
    --sha256) EXPECT_SHA="$2"; shift 2 ;;
    --skip-verify) SKIP_VERIFY=1; shift ;;
    --force) FORCE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1 (see --help)" >&2; exit 2 ;;
  esac
done

need() { command -v "$1" >/dev/null 2>&1 || { echo "Missing required tool: $1" >&2; exit 1; }; }
need curl; need tar; need python3
if command -v sha256sum >/dev/null 2>&1; then SHA_CMD="sha256sum"; else need shasum; SHA_CMD="shasum -a 256"; fi

if [ "$GLOBAL" -eq 1 ]; then
  SKILL_DEST="$HOME/.agents/skills/$SKILL"
  OPENCODE_JSON="$HOME/.config/opencode/opencode.json"
else
  SKILL_DEST="$DIR/.agents/skills/$SKILL"
  OPENCODE_JSON="$DIR/opencode.json"
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# --- fetch tarball -----------------------------------------------------------
if [ -z "$URL" ]; then SRC="$BASE_URL/$TARBALL"; else SRC="$URL"; fi
echo "Fetching $SRC ..."
case "$SRC" in
  http://*|https://*)
    curl -fsSL -o "$TMP/$TARBALL" "$SRC"
    if [ -n "$EXPECT_SHA" ]; then :;
    elif [ "$SKIP_VERIFY" -eq 0 ]; then
      echo "Fetching $SRC.sha256 ..."
      EXPECT_SHA="$(curl -fsSL "$SRC.sha256" | awk '{print $1}')"
    fi
    ;;
  file://*) curl -fsSL -o "$TMP/$TARBALL" "$SRC" ;;
  *) [ -f "$SRC" ] || { echo "Local file not found: $SRC" >&2; exit 1; }
     cp "$SRC" "$TMP/$TARBALL" ;;
esac

if [ -n "$EXPECT_SHA" ]; then
  ACTUAL="$( $SHA_CMD "$TMP/$TARBALL" | awk '{print $1}')"
  # shellcheck disable=SC2001
  EXPECT_NORM="$(echo "$EXPECT_SHA" | tr '[:upper:]' '[:lower:]')"
  ACTUAL_NORM="$(echo "$ACTUAL" | tr '[:upper:]' '[:lower:]')"
  [ "$EXPECT_NORM" = "$ACTUAL_NORM" ] || { echo "SHA256 MISMATCH (expected $EXPECT_NORM, got $ACTUAL_NORM)" >&2; exit 1; }
  echo "Hash verified."
elif [ "$SKIP_VERIFY" -eq 1 ]; then
  echo "Skipping hash verification (--skip-verify)."
else
  echo "No hash available; use --sha256 or host $TARBALL.sha256 (or --skip-verify)."
fi

# --- install skill files -----------------------------------------------------
tar -xzf "$TMP/$TARBALL" -C "$TMP"
[ -f "$TMP/$SKILL/SKILL.md" ] || { echo "Tarball is missing $SKILL/SKILL.md" >&2; exit 1; }
if [ -d "$SKILL_DEST" ]; then echo "Updating existing skill at $SKILL_DEST"; else echo "Installing skill to $SKILL_DEST"; fi
mkdir -p "$(dirname "$SKILL_DEST")"
rm -rf "$SKILL_DEST"
cp -r "$TMP/$SKILL" "$SKILL_DEST"

# --- wire Zagros MCP into opencode.json --------------------------------------
export CCSKILL_MCP_NAME="$MCP_NAME" CCSKILL_MCP_URL="$MCP_URL"
mkdir -p "$(dirname "$OPENCODE_JSON")"
python3 - "$OPENCODE_JSON" "$FORCE" <<'PYEOF'
import json, os, shutil, sys
cfg, force = sys.argv[1], sys.argv[2] == "1"
name, url = os.environ["CCSKILL_MCP_NAME"], os.environ["CCSKILL_MCP_URL"]
entry = {"type": "remote", "enabled": True, "url": url}
if os.path.exists(cfg):
    try:
        with open(cfg, encoding="utf-8") as f:
            data = json.load(f)
    except Exception as e:  # e.g. JSONC comments: do not clobber
        print(f"WARN: {cfg} is not plain JSON ({e}); leaving it untouched.")
        print(f'ACTION NEEDED: add manually under "mcp": {json.dumps({name: entry})} ')
        sys.exit(0)
    if not isinstance(data, dict):
        data = {}
else:
    data = {}
mcp = data.get("mcp")
if not isinstance(mcp, dict):
    mcp = {}
    data["mcp"] = mcp
if name in mcp and not force:
    print(f"MCP '{name}' already present in {cfg}; keeping it (use --force to overwrite).")
else:
    if os.path.exists(cfg):
        shutil.copy2(cfg, cfg + ".bak")
        print(f"Backed up {cfg} -> {cfg}.bak")
    mcp[name] = entry
    with open(cfg, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        f.write("\n")
    print(f"Wired MCP '{name}' into {cfg}")
PYEOF

echo
echo "Done. Skill: $SKILL_DEST"
echo "Next: restart opencode, then verify with:  opencode mcp list   (expect Zagros: connected)"
