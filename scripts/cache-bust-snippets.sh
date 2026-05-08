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
#   3. Rewrites every `snippets/<old>/` reference in dist/*.js and
#      dist/*.html to use the new folder name.
#   4. For any non-HTML file we modified in step 3 (the wasm-bindgen
#      JS shim), recomputes its SHA-384 base64 digest and patches the
#      stale `integrity="sha384-…"` value in dist/*.html. Trunk
#      computes integrity attributes during its build phase — before
#      `post_build` hooks run — so without this fix the browser blocks
#      the modified shim with `Failed to find a valid digest in the
#      'integrity' attribute…`.
#
# Folder name now changes whenever the snippet contents change, so the
# next deploy busts every cache layer naturally — no runtime/protocol
# changes, no service worker, no header tuning needed.
#
# Idempotent: re-running on an already-content-hashed folder produces
# the same name (same content → same hash).

set -euo pipefail

DIST="${TRUNK_STAGING_DIR:-dist}"
SNIPPETS_DIR="$DIST/snippets"

if [[ ! -d "$SNIPPETS_DIR" ]]; then
    # No snippets in this build — nothing to do.
    exit 0
fi

shopt -s nullglob

# Compute a file's SHA-384 in the base64 form Trunk uses for SRI
# (`integrity="sha384-<base64>"`). openssl is universally available on
# both macOS and the Linux CI runners.
sri_digest() {
    openssl dgst -sha384 -binary "$1" | openssl base64 -A
}

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

    # Track integrity hash updates we need to make to index.html after
    # rewriting imports. Stored as parallel "old<TAB>new" lines.
    sri_updates="$(mktemp)"

    while IFS= read -r -d '' f; do
        # Snapshot the SRI hash before modification — this is what Trunk
        # currently has in index.html for any file it preloads.
        old_sri=""
        if [[ "$f" != *.html ]]; then
            old_sri="$(sri_digest "$f")"
        fi

        # sed -i.bak is portable across BSD/GNU; drop the backup right
        # after.
        sed -i.bak "s|snippets/${old_dir}/|snippets/${new_dir}/|g" "$f"
        rm -f "${f}.bak"

        if [[ -n "$old_sri" ]]; then
            new_sri="$(sri_digest "$f")"
            if [[ "$old_sri" != "$new_sri" ]]; then
                printf '%s\t%s\n' "$old_sri" "$new_sri" >> "$sri_updates"
            fi
        fi
    done < <(find "$DIST" -maxdepth 2 -type f \( -name '*.js' -o -name '*.html' \) -print0)

    # Apply integrity updates to every HTML file at the dist root.
    if [[ -s "$sri_updates" ]]; then
        while IFS= read -r -d '' html_file; do
            while IFS=$'\t' read -r old_sri new_sri; do
                # Escape sed-special chars in the base64 digests
                # (`/`, `+`, `=`); none of `&` / `\` ever appear in
                # base64 output. Pipe is our chosen sed delimiter.
                old_escaped="$(printf '%s' "$old_sri" | sed 's/[\&|]/\\&/g')"
                new_escaped="$(printf '%s' "$new_sri" | sed 's/[\&|]/\\&/g')"
                sed -i.bak "s|sha384-${old_escaped}|sha384-${new_escaped}|g" "$html_file"
                rm -f "${html_file}.bak"
            done < "$sri_updates"
        done < <(find "$DIST" -maxdepth 1 -type f -name '*.html' -print0)
    fi

    rm -f "$sri_updates"

    printf '[cache-bust-snippets] %s → %s\n' "$old_dir" "$new_dir"
    renamed_any=1
done

if [[ "$renamed_any" -eq 0 ]]; then
    printf '[cache-bust-snippets] no snippet folder needed renaming\n'
fi
