#!/usr/bin/env python3
"""Assert every SRI hash in dist/index.html matches the on-disk file.

Walks every `<link>/<script integrity="sha384-…">` tag, recomputes
SHA-384 from the file the tag references, and exits 1 with a pointed
error message if any mismatch is found. Pure check — never modifies
files.

Why this exists
───────────────
Belt-and-braces guard. Run AFTER `refresh-sri-hashes.py` (and any other
dist/ modification step) so the deploy fails loudly if some new step
shipped between recompute and wrangler. Without this, a stale
integrity attribute means the browser blocks the file at runtime, the
inline `<script type="module">` import silently never resolves, and the
page renders blank — no CI signal at all.

Production blank-page incident 2026-05-09 (PR #594) was caught by user
DevTools, not CI. This script is the gap-filler.

Usage
─────
    scripts/verify-sri-hashes.py [DIST_DIR]

Defaults to `crates/intrada-web/dist`. Exits 0 if all hashes match;
exits 1 with a human-readable mismatch report otherwise.
"""

from __future__ import annotations

import base64
import hashlib
import pathlib
import re
import sys

DEFAULT_DIST = "crates/intrada-web/dist"

TAG_RE = re.compile(
    r'<(?:link|script)\b[^>]*\bintegrity="sha384-([^"]+)"[^>]*>', re.IGNORECASE
)
SRC_RE = re.compile(r'(?:href|src)="(/?[^"#?]+)"', re.IGNORECASE)


def main() -> int:
    dist = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else DEFAULT_DIST)
    index = dist / "index.html"
    if not index.is_file():
        print(f"verify-sri-hashes: {index} not found", file=sys.stderr)
        return 1

    html = index.read_text()
    mismatches: list[str] = []
    checked = 0
    for match in TAG_RE.finditer(html):
        declared = match.group(1)
        src = SRC_RE.search(match.group(0))
        if not src:
            continue
        rel = src.group(1).lstrip("/")
        file = dist / rel
        if not file.is_file():
            mismatches.append(f"  {src.group(1)}: referenced but not on disk")
            continue
        actual = base64.b64encode(hashlib.sha384(file.read_bytes()).digest()).decode()
        if declared != actual:
            mismatches.append(
                f"  {src.group(1)}: declared sha384-{declared[:12]}…, "
                f"actual sha384-{actual[:12]}…"
            )
        checked += 1

    if mismatches:
        print("verify-sri-hashes: SRI integrity mismatch — would ship a blank page:")
        for line in mismatches:
            print(line)
        print(
            "\nLikely cause: a step modified dist/ between the refresh-sri-hashes "
            "step and this verify step. Either move it BEFORE the refresh, or "
            "re-run refresh after it.",
            file=sys.stderr,
        )
        return 1
    print(f"verify-sri-hashes: {checked} integrity hashes match on-disk files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
