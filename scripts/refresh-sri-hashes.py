#!/usr/bin/env python3
"""Recompute SHA-384 integrity hashes in all HTML files under dist/.

Walks every `<link>/<script integrity="sha384-…">` tag, computes the
SHA-384 of the file the tag references, and rewrites the attribute.
Files whose contents weren't touched recompute to the same value
(no-op).

Scans index.html and any prerendered HTML files (dist/prerendered/*.html)
so that SRI hashes stay valid after sentry-cli inject modifies JS files.

Why this exists
───────────────
Trunk computes integrity hashes at build time. Anything that modifies
dist/ files AFTER trunk's build (`sentry-cli sourcemaps inject`,
`sed`-based placeholder injection, etc.) leaves the integrity attributes
stale, which causes the browser to refuse the modified file:

    Failed to find a valid digest in the 'integrity' attribute …
    The resource has been blocked.

Run this AFTER any post-build modification step and BEFORE wrangler
publish. The companion `verify-sri-hashes.py` script asserts that all
hashes match — run it last to catch any modification step that slipped
in without calling refresh.

Usage
─────
    scripts/refresh-sri-hashes.py [DIST_DIR]

Defaults to `crates/intrada-web/dist`. Exits 0 on success.
"""

from __future__ import annotations

import base64
import hashlib
import pathlib
import re
import sys

DEFAULT_DIST = "crates/intrada-web/dist"

TAG_RE = re.compile(
    r'<(?:link|script)\b[^>]*\bintegrity="sha384-[^"]+"[^>]*>', re.IGNORECASE
)
SRC_RE = re.compile(r'(?:href|src)="(/?[^"#?]+)"', re.IGNORECASE)


def refresh_file(html_path: pathlib.Path, dist: pathlib.Path) -> tuple[int, bool]:
    """Refresh SRI hashes in a single HTML file. Returns (count, changed)."""
    html = html_path.read_text()

    def patch(match: re.Match[str]) -> str:
        tag = match.group(0)
        src = SRC_RE.search(tag)
        if not src:
            return tag
        rel = src.group(1).lstrip("/")
        file = dist / rel
        if not file.is_file():
            return tag
        digest = base64.b64encode(hashlib.sha384(file.read_bytes()).digest()).decode()
        return re.sub(
            r'integrity="sha384-[^"]+"',
            f'integrity="sha384-{digest}"',
            tag,
        )

    new_html, count = TAG_RE.subn(patch, html)
    changed = new_html != html
    if changed:
        html_path.write_text(new_html)
    return count, changed


def main() -> int:
    dist = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else DEFAULT_DIST)
    index = dist / "index.html"
    if not index.is_file():
        print(f"refresh-sri-hashes: {index} not found", file=sys.stderr)
        return 1

    html_files = [index] + sorted(
        f for f in dist.rglob("*.html") if f != index
    )

    total_count = 0
    total_changed = 0
    for html_path in html_files:
        count, changed = refresh_file(html_path, dist)
        total_count += count
        if changed:
            total_changed += 1
            rel = html_path.relative_to(dist)
            print(f"refresh-sri-hashes: refreshed {count} hashes in {rel}")

    if total_changed == 0:
        print(
            f"refresh-sri-hashes: all {total_count} integrity hashes already "
            f"match across {len(html_files)} file(s)"
        )
    else:
        print(
            f"refresh-sri-hashes: refreshed {total_count} hashes across "
            f"{total_changed}/{len(html_files)} file(s)"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
