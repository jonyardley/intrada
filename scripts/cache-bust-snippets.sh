#!/usr/bin/env bash
# Trunk post_build hook — content-hash the wasm-bindgen snippets folder.
#
# Why
# ───
# wasm-bindgen emits each `#[wasm_bindgen(inline_js = "…")]` block as a
# separate file at:
#
#     dist/snippets/<crate>-<wasm-bindgen-hash>/inlineN.js
#
# - The folder hash (e.g. `intrada-web-45068482bf5d8aa1`) is **stable
#   across builds** — wasm-bindgen derives it from the crate identity,
#   not the snippet contents.
# - The file numbering (`inline0.js`, `inline1.js`, …) is determined by
#   declaration order across the crate's source. Add or remove an
#   `inline_js` block and the numbering shifts.
#
# Net effect: a build that adds, removes, or reorders an `inline_js`
# block produces snippets at the **same folder path** but with
# **different contents**. Any cache that pins the path (browser HTTP
# cache, Cloudflare, WKWebView data store, service worker) keeps serving
# the old files, so the new `intrada-web-<hash>.js` module imports a
# binding (e.g. `set_now_playing`) that the cached snippet no longer
# exports — and the page errors with:
#
#     SyntaxError: Importing binding name 'set_now_playing' is not found.
#
# Tracked at jonyardley/intrada#440 and #435.
#
# What
# ────
# After Trunk has finished its build, this script:
#
#   1. Hashes the contents of dist/snippets/<crate>-<wb-hash>/
#   2. Renames the folder to dist/snippets/<crate>-<content-hash>/
#   3. Rewrites every `snippets/<old>/` reference in:
#        - dist/*.js (wasm-bindgen shim — primary site)
#        - dist/*.html (defensive)
#        - dist/*.wasm (wasm-bindgen embeds the path in the WASM
#          import section as a UTF-8 string)
#   4. Recomputes SHA-384 integrity hashes for every file referenced
#      by an `integrity="sha384-..."` attribute in index.html. Trunk
#      emits SRI for the shim and the .wasm; rewriting their contents
#      invalidates the digest, and browsers then block the resource
#      ("Failed to find a valid digest in the 'integrity' attribute").
#
# Folder name now changes whenever the snippet contents change, so the
# next deploy busts every cache layer naturally — no runtime/protocol
# changes, no service worker, no header tuning needed.
#
# Idempotent: re-running on an already-content-hashed folder produces
# the same name (same content → same hash) and skips step 3+4.

set -euo pipefail

DIST="${TRUNK_STAGING_DIR:-dist}"
SNIPPETS_DIR="$DIST/snippets"

if [[ ! -d "$SNIPPETS_DIR" ]]; then
    # No snippets in this build — nothing to do.
    exit 0
fi

shopt -s nullglob

renamed_any=0

for old_path in "$SNIPPETS_DIR"/*/; do
    old_dir="$(basename "$old_path")"

    # Compute a content hash from every file in the folder. We sort the
    # paths first so the hash is deterministic regardless of filesystem
    # iteration order.
    content_hash="$(
        find "$old_path" -type f -print0 \
            | LC_ALL=C sort -z \
            | xargs -0 cat \
            | shasum -a 256 \
            | cut -c1-16
    )"

    # Strip any trailing -<hex> suffix from the existing folder name and
    # append our content hash. wasm-bindgen currently emits 16-char
    # hashes; allow 16+ to be tolerant of future changes. Also matches
    # our own previous content-hash run so subsequent runs are
    # idempotent.
    base="$(printf '%s' "$old_dir" | sed -E 's/-[0-9a-f]{16,}$//')"
    new_dir="${base}-${content_hash}"

    if [[ "$old_dir" == "$new_dir" ]]; then
        continue
    fi

    mv "$old_path" "$SNIPPETS_DIR/$new_dir"

    # Rewrite imports in JS and HTML files at the dist root. The wasm-
    # bindgen shim JS is the primary site (it does
    # `import { … } from './snippets/<old>/inlineN.js';`); HTML scan is
    # cheap insurance against any future direct references.
    while IFS= read -r -d '' f; do
        # sed -i.bak is portable across BSD/GNU; we drop the backup
        # immediately afterwards.
        sed -i.bak "s|snippets/${old_dir}/|snippets/${new_dir}/|g" "$f"
        rm -f "${f}.bak"
    done < <(find "$DIST" -maxdepth 2 -type f \( -name '*.js' -o -name '*.html' \) -print0)

    # Also rewrite the WASM binary's embedded import path strings.
    # wasm-bindgen bakes the snippet path into the WASM module's import
    # section as a UTF-8 string. If we rename the folder without
    # rewriting the WASM, `WebAssembly.instantiate()` looks up imports
    # under the OLD path which the JS shim no longer provides — and
    # the whole WASM fails to load with: "module is not an object or
    # function". The replacement is byte-for-byte (both old and new
    # folder names are `<crate>-` + 16-char hex = identical length),
    # so the length-prefixed string stays valid.
    # Use perl (-0777 slurp) for binary-safe in-place editing — sed's
    # line-buffered behaviour can mangle non-text bytes.
    while IFS= read -r -d '' wasm; do
        perl -0777 -i -pe "s|snippets/\Q${old_dir}\E/|snippets/${new_dir}/|g" "$wasm"
    done < <(find "$DIST" -maxdepth 2 -type f -name '*.wasm' -print0)

    printf '[cache-bust-snippets] %s → %s\n' "$old_dir" "$new_dir"
    renamed_any=1
done

if [[ "$renamed_any" -eq 0 ]]; then
    printf '[cache-bust-snippets] no snippet folder needed renaming\n'
    exit 0
fi

# Recompute SHA-384 integrity hashes for every file in dist that's
# referenced by an `integrity="sha384-..."` attribute in index.html.
# Trunk emits SRI hashes for the wasm-bindgen JS shim, the .wasm, and
# the snippet files. Modifying any of those (which the loop above does
# for the shim and the wasm) leaves index.html with stale digests, and
# the browser blocks the resource:
#   "Failed to find a valid digest in the 'integrity' attribute …
#    The resource has been blocked."
# This walks every <link>/<script integrity=...> in index.html and
# replaces the hash with whatever the on-disk file currently hashes to.
INDEX="$DIST/index.html"
if [[ -f "$INDEX" ]]; then
    python3 - "$DIST" "$INDEX" <<'PY'
import base64, hashlib, pathlib, re, sys

dist, index = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
html = index.read_text()
tag_re = re.compile(r'<(?:link|script)\b[^>]*\bintegrity="sha384-[^"]+"[^>]*>', re.I)
src_re = re.compile(r'(?:href|src)="(/?[^"#?]+)"', re.I)


def patch(m):
    tag = m.group(0)
    src = src_re.search(tag)
    if not src:
        return tag
    rel = src.group(1).lstrip('/')
    file = dist / rel
    if not file.is_file():
        return tag
    digest = base64.b64encode(hashlib.sha384(file.read_bytes()).digest()).decode()
    return re.sub(r'integrity="sha384-[^"]+"', f'integrity="sha384-{digest}"', tag)


new_html, count = tag_re.subn(patch, html)
if new_html != html:
    index.write_text(new_html)
    print(f'[cache-bust-snippets] refreshed {count} integrity hashes in index.html')
PY
fi
