#!/usr/bin/env python3
"""Check local Markdown references and simple Markdown hygiene."""
import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
SOURCES = [ROOT / "README.md", *sorted((ROOT / "docs").rglob("*.md")), *sorted((ROOT / "website/src/content/docs").rglob("*.md"))]
LINK = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")
FENCE = re.compile(r"^\s*(`{3,}|~{3,})")
def check(path: Path) -> list[str]:
    failures = []
    text = path.read_text(encoding="utf-8-sig")
    fence = None
    for line_no, line in enumerate(text.splitlines(), 1):
        match = FENCE.match(line)
        if match:
            marker = match.group(1)
            if fence is None:
                fence = marker
            elif marker[0] == fence[0] and len(marker) >= len(fence):
                fence = None
            continue
        if fence:
            continue
        for m in LINK.finditer(line):
            target = m.group(1).strip().split(" ", 1)[0].strip("<>")
            url = urlsplit(target)
            if url.scheme or target.startswith(("#", "/", "//", "mailto:", "data:")):
                continue
            local = unquote(url.path).replace("\\", "/")
            candidate = (path.parent / local).resolve()
            if not candidate.is_relative_to(ROOT):
                failures.append(f"{path.relative_to(ROOT)}:{line_no}: reference escapes repository: {target}")
            elif local and not candidate.exists():
                # Published copies contain site routes and repository-relative
                # links; resolve both using the source-of-truth docs and site corpus.
                published = ROOT / "website/src/content/docs"
                valid = False
                if path.is_relative_to(published):
                    original = ROOT / "docs" / path.name
                    if original.exists():
                        orig_target = (original.parent / local).resolve()
                        valid = orig_target.is_relative_to(ROOT) and orig_target.exists()
                    if not valid and local.endswith("/"):
                        slug = Path(local.rstrip("/")).name.lower()
                        valid = any(f.stem.lower() == slug for f in published.glob("*.md"))
                    if not valid:
                        repo_rel = local.removeprefix("../")
                        valid = (ROOT / repo_rel).exists()
                if not valid:
                    failures.append(f"{path.relative_to(ROOT)}:{line_no}: missing referenced file: {target}")
    if fence:
        failures.append(f"{path.relative_to(ROOT)}: unclosed fenced code block")
    return failures

def main() -> int:
    failures = [err for path in SOURCES for err in check(path)]
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"Documentation checks passed: {len(SOURCES)} Markdown files")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
