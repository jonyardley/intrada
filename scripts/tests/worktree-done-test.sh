#!/usr/bin/env bash
# Self-test for scripts/worktree-done.sh (#2918). A real repo in a temp folder
# holds one worktree per case, a fake `gh` answers `pr list` from fixtures,
# honouring --head and --state and running the script's own --jq, and a fake
# `just` stands in for the simulator clean. Each refusal gets a case where only
# its check decides, then the removal that should pass.
# WORKTREE_DONE points it at a doctored copy for mutation testing.
set -euo pipefail
source "$(dirname "$0")/../lib/contains.sh"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="${WORKTREE_DONE:-$root/scripts/worktree-done.sh}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
tmp="$(cd "$tmp" && pwd -P)"
mkdir -p "$tmp/bin" "$tmp/fixtures"

# ── The fakes ───────────────────────────────────────────────────────────────

cat >"$tmp/bin/gh" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail
if [ -n "${FAIL_GH:-}" ]; then
  echo "gh: Could not resolve host: api.github.com" >&2
  exit 1
fi
[ "$1 $2" = "pr list" ] || { echo "fake gh: unexpected $*" >&2; exit 2; }
shift 2
head="" state="open" jqf="."
while [ $# -gt 0 ]; do
  case "$1" in
    --head) head="$2"; shift 2 ;;
    --state) state="$2"; shift 2 ;;
    --jq) jqf="$2"; shift 2 ;;
    --json) shift 2 ;;
    *) shift ;;
  esac
done
jq -c --arg h "$head" --arg s "$state" \
  '[.[] | select(.branch == $h and ($s == "all" or .state == ($s | ascii_upcase)))]' \
  "$FIXTURES/prs.json" | jq -r "$jqf"
FAKE
chmod +x "$tmp/bin/gh"

cat >"$tmp/bin/just" <<'FAKE'
#!/usr/bin/env bash
printf '%s\n' "$*" >>"$FIXTURES/just.txt"
FAKE
chmod +x "$tmp/bin/just"

export FIXTURES="$tmp/fixtures"
export PATH="$tmp/bin:$PATH"
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t

# ── A repo with one worktree per case ───────────────────────────────────────

git init -q -b main "$tmp/main"
git -C "$tmp/main" commit -q --allow-empty -m base

wt() {
  git -C "$tmp/main" worktree add -q -b "$1" "$tmp/$1" main
  git -C "$tmp/$1" commit -q --allow-empty -m "$1 work"
}
head_of() { git -C "$tmp/$1" rev-parse HEAD; }

wt dirty
touch "$tmp/dirty/stray.txt"
wt moved
moved_merged="$(head_of moved)"
git -C "$tmp/moved" commit -q --allow-empty -m "after the merge"
wt unmerged
wt offline
wt done

pr() { # branch number state headRefOid
  jq -c --arg b "$1" --argjson n "$2" --arg s "$3" --arg o "$4" \
    '. + [{branch: $b, number: $n, state: $s, headRefOid: $o}]' \
    "$FIXTURES/prs.json" >"$FIXTURES/prs.tmp"
  mv "$FIXTURES/prs.tmp" "$FIXTURES/prs.json"
}

echo '[]' >"$FIXTURES/prs.json"
pr dirty 3001 MERGED "$(head_of dirty)"
pr moved 3002 MERGED "$moved_merged"
pr unmerged 3003 OPEN "$(head_of unmerged)"
pr offline 3004 MERGED "$(head_of offline)"
pr done 3005 MERGED "$(head_of done)"
: >"$FIXTURES/just.txt"

pass=0
fail=0

ok() { pass=$((pass + 1)); }
bad() { fail=$((fail + 1)); printf '✗ %s\n' "$1" >&2; }

check() {
  local haystack="$1" desc="$2" needle="$3"
  if contains "$needle" "$haystack"; then ok; else
    bad "$desc"; printf '    expected to find: %s\n    in: %s\n' "$needle" "$haystack" >&2; fi
}

# Run the script in a folder; sets out (both streams) and code.
run() {
  local dir="$1"
  shift
  code=0
  out="$(cd "$dir" && env "$@" "$script" 2>&1)" || code=$?
}

refused() { # folder desc needle [env...]
  local dir="$1" desc="$2" needle="$3"
  shift 3
  run "$dir" "$@"
  if [ "$code" -ne 0 ]; then ok; else bad "$desc: exited 0"; fi
  check "$out" "$desc: names why" "$needle"
  if [ -d "$dir" ]; then ok; else bad "$desc: the folder was removed"; fi
}

# ── Refusals ────────────────────────────────────────────────────────────────

refused "$tmp/main" "the main checkout is refused" "this is the main checkout"
refused "$tmp/dirty" "a dirty tree is refused" "uncommitted or untracked files"
refused "$tmp/moved" "a HEAD past the merged PR's head is refused" "is not the last commit of the merged PR"
refused "$tmp/unmerged" "an open PR is not a merged one" "no merged PR for unmerged"
refused "$tmp/offline" "GitHub not answering is refused" "GitHub could not be read" FAIL_GH=1
if [ -s "$FIXTURES/just.txt" ]; then bad "a refusal cleaned the simulator"; else ok; fi

# ── The removal ─────────────────────────────────────────────────────────────

run "$tmp/done"
if [ "$code" -eq 0 ]; then ok; else bad "a merged, clean, pushed worktree is removed: exited $code"; fi
check "$out" "says what it removed" "removed $tmp/done (PR #3005 merged"
if [ -d "$tmp/done" ]; then bad "the folder is still there"; else ok; fi
if git -C "$tmp/main" rev-parse --verify -q refs/heads/done >/dev/null; then ok; else bad "the branch was deleted"; fi
check "$(cat "$FIXTURES/just.txt")" "cleans the simulator" "ios-test-sim-clean"
if contains "$tmp/done" "$(git -C "$tmp/main" worktree list --porcelain)"; then
  bad "git still lists the worktree"; else ok; fi

printf '\n%s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
