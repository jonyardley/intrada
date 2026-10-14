#!/usr/bin/env bash
# Self-test for scripts/pr-open.sh (#1702 step 3): every issue number in the
# PR title must have a claim naming the current branch, or the PR does not
# open. The fake `gh issue view` holds real comment JSON and applies
# whatever `-q` expression the script passes with real jq, so a broken jq
# program fails this suite instead of being ignored by a fake that just
# echoes text.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/pr-open.sh"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/fixtures" "$work/calls" "$work/repo"

cat >"$work/bin/gh" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail

cmd="$1"; shift
sub="${1:-}"

case "$cmd $sub" in
  "repo view")
    echo "jonyardley/intrada"
    ;;
  "issue view")
    number="$2"
    shift 2
    q=""
    args=("$@")
    for i in "${!args[@]}"; do
      if [ "${args[$i]}" = "-q" ]; then q="${args[$((i + 1))]}"; fi
    done
    body="$(cat "$FIXTURES/comments_$number.json" 2>/dev/null || echo '{"comments":[]}')"
    if [ -n "$q" ]; then
      printf '%s' "$body" | jq -r "$q"
    else
      printf '%s' "$body"
    fi
    ;;
  "pr create")
    echo "$*" >>"$CALLS/pr_create.log"
    echo "https://github.com/jonyardley/intrada/pull/1"
    ;;
  *)
    echo "fake gh: unhandled invocation: $cmd $sub $*" >&2
    exit 1
    ;;
esac
FAKE
chmod +x "$work/bin/gh"

export FIXTURES="$work/fixtures"
export CALLS="$work/calls"
export PATH="$work/bin:$PATH"

git -C "$work/repo" init -q -b main
git -C "$work/repo" config user.email "test@example.com"
git -C "$work/repo" config user.name "Test"
git -C "$work/repo" commit -q --allow-empty -m init
git -C "$work/repo" checkout -q -b my-branch

pass=0
fail=0

run() {
  ( cd "$work/repo" && "$script" --title "$1" --body "irrelevant" )
}

# ── No claim at all ──────────────────────────────────────────────────────────

echo '{"comments":[]}' >"$work/fixtures/comments_42.json"
rm -rf "$CALLS" && mkdir -p "$CALLS"
set +e
out="$(run "Fix the thing (#42)" 2>&1)"
run_status=$?
set -e
if [ "$run_status" -ne 0 ] && printf '%s' "$out" | grep -qF "no claim comment" && [ ! -f "$CALLS/pr_create.log" ]; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  printf '✗ no claim: expected refusal before gh pr create\n    status=%s output=%s\n' "$run_status" "$out" >&2
fi

# ── Claimed on a different branch ───────────────────────────────────────────

cat >"$work/fixtures/comments_42.json" <<'JSON'
{"comments":[{"body":"Claimed: branch `other-branch`, doing it."}]}
JSON
rm -rf "$CALLS" && mkdir -p "$CALLS"
set +e
out="$(run "Fix the thing (#42)" 2>&1)"
run_status=$?
set -e
if [ "$run_status" -ne 0 ] && printf '%s' "$out" | grep -qF "claimed on \`other-branch\`" && [ ! -f "$CALLS/pr_create.log" ]; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  printf '✗ wrong branch: expected refusal naming other-branch\n    status=%s output=%s\n' "$run_status" "$out" >&2
fi

# ── Claimed on this branch, legacy no-backtick comment format: opens the PR ─

cat >"$work/fixtures/comments_42.json" <<'JSON'
{"comments":[{"body":"Claimed. Branch: my-branch (doing it)."}]}
JSON
rm -rf "$CALLS" && mkdir -p "$CALLS"
run "Fix the thing (#42)" >/dev/null
if grep -qF -- "--title Fix the thing (#42)" "$CALLS/pr_create.log" 2>/dev/null; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  printf '✗ legacy format: expected gh pr create to run with the same args\n' >&2
fi

# ── Opens as a draft unless told otherwise ──────────────────────────────────

rm -rf "$CALLS" && mkdir -p "$CALLS"
run "Fix the thing (#42)" >/dev/null
if grep -qF -- "--draft" "$CALLS/pr_create.log" 2>/dev/null; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  printf '✗ draft: expected gh pr create to get --draft\n' >&2
fi

# ── Claimed on this branch, current backtick format: opens the PR ──────────

cat >"$work/fixtures/comments_42.json" <<'JSON'
{"comments":[{"body":"Claimed: branch `my-branch`, doing it."}]}
JSON
rm -rf "$CALLS" && mkdir -p "$CALLS"
run "Fix the thing (#42)" >/dev/null
if grep -qF -- "--title Fix the thing (#42)" "$CALLS/pr_create.log" 2>/dev/null; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  printf '✗ claimed on this branch: expected gh pr create to run with the same args\n' >&2
fi

# ── One claim naming two branches (#2669): either opens, a third is refused ─

cat >"$work/fixtures/comments_42.json" <<'JSON'
{"comments":[{"body":"Claimed: branch `first-branch` (one half) and branch `my-branch` (the other), in parallel."}]}
JSON
rm -rf "$CALLS" && mkdir -p "$CALLS"
run "Fix the thing (#42)" >/dev/null
if grep -qF -- "--title Fix the thing (#42)" "$CALLS/pr_create.log" 2>/dev/null; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  printf '✗ second named branch: expected gh pr create to run\n' >&2
fi

cat >"$work/fixtures/comments_42.json" <<'JSON'
{"comments":[{"body":"Claimed: branch `first-branch` and branch `second-branch`, in parallel."}]}
JSON
rm -rf "$CALLS" && mkdir -p "$CALLS"
set +e
out="$(run "Fix the thing (#42)" 2>&1)"
run_status=$?
set -e
if [ "$run_status" -ne 0 ] && printf '%s' "$out" | grep -qF "claimed on \`first-branch\`, \`second-branch\`" && [ ! -f "$CALLS/pr_create.log" ]; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  printf '✗ unnamed third branch: expected refusal naming both claimed branches\n    status=%s output=%s\n' "$run_status" "$out" >&2
fi

# ── Body length (#2593): a Tier 1 or 2 body over 350 words is refused ──────

words() {
  local i
  for ((i = 0; i < $1; i++)); do printf '%s ' "$2"; done
}

body_run() {
  rm -rf "$CALLS" && mkdir -p "$CALLS"
  set +e
  out="$(cd "$work/repo" && "$script" --title "Tidy the docs" --body "$1" 2>&1)"
  run_status=$?
  set -e
}

expect_refused() {
  if [ "$run_status" -ne 0 ] && printf '%s' "$out" | grep -qF "350" && [ ! -f "$CALLS/pr_create.log" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s: expected a refusal naming 350\n    status=%s output=%s\n' "$1" "$run_status" "$out" >&2
  fi
}

expect_opened() {
  if [ "$run_status" -eq 0 ] && [ -f "$CALLS/pr_create.log" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s: expected gh pr create to run\n    status=%s output=%s\n' "$1" "$run_status" "$out" >&2
  fi
}

long="$(words 360 note.)"
short="$(words 300 note.)"

body_run "## What this fixes
$long"
expect_refused "360 words, no Tier 3 heading"

body_run "## What this fixes
$long
## Where this could bite
none"
expect_opened "360 words with Where this could bite"

body_run "## What this fixes
$long
## What changed where
none"
expect_opened "360 words with What changed where"

body_run "## What this fixes
$long
## Where this could bite
<!-- say what could go wrong -->"
expect_refused "360 words with an empty Tier 3 heading"

body_run "## What this fixes
$short
<!-- $(words 200 hint) -->"
expect_opened "template comments are not counted"

body_run "## What this fixes
$short
## Checklist
$(words 100 item)"
expect_opened "the checklist is not counted"

rm -rf "$CALLS" && mkdir -p "$CALLS"
set +e
out="$(cd "$work/repo" && SKIP_BODY_CHECK=1 "$script" --title "Tidy the docs" --body "$long" 2>&1)"
run_status=$?
set -e
expect_opened "SKIP_BODY_CHECK=1"

printf '%s\n' "$long" >"$work/body.md"
rm -rf "$CALLS" && mkdir -p "$CALLS"
set +e
out="$(cd "$work/repo" && "$script" --title "Tidy the docs" --body-file "$work/body.md" 2>&1)"
run_status=$?
set -e
expect_refused "--body-file is counted too"

# ── Sentence length (#2593): a warning, never a refusal ───────────────────

body_run "## What this fixes
Short one. $(words 30 word) end."
expect_opened "a long sentence still opens"
if printf '%s' "$out" | grep -qF "over 25 words"; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  printf '✗ long sentence: expected a warning naming 25 words\n    output=%s\n' "$out" >&2
fi

expect_no_warning() {
  if printf '%s' "$out" | grep -qF "over 25 words"; then
    fail=$((fail + 1))
    printf '✗ %s: expected no warning\n    output=%s\n' "$1" "$out" >&2
  else
    pass=$((pass + 1))
  fi
}

body_run "## What this fixes
$(words 20 word).
Another short sentence."
expect_no_warning "short sentences"

body_run "## $(words 8 heading)
$(words 20 word) end."
expect_no_warning "a heading is not part of the sentence below it"

body_run "## What this fixes
\`\`\`
$(words 30 code)
\`\`\`"
expect_no_warning "a fenced block is not prose"

body_run "## What this fixes
- $(words 15 bullet)
- $(words 15 bullet)"
expect_no_warning "each bullet is its own unit"

printf '\n%s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
