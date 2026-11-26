#!/usr/bin/env bash
# Self-test for scripts/pr-ready.sh (#2955): one case per refusal, each the
# passing fixture with one thing changed, plus the passing cases. The fake
# `gh` mirrors the real failures: an unknown PR fails with a GraphQL line,
# and `gh pr checks` on a commit with no checks fails with "no checks
# reported". Each case asserts its own reason, so deleting any one refusal
# line fails its case alone.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/pr-ready.sh"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/fixtures" "$work/calls" "$work/repo"

cat >"$work/bin/gh" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail
case "$1 $2" in
  "pr view")
    if [ ! -f "$FIXTURES/view.json" ]; then
      echo "GraphQL: Could not resolve to a PullRequest with the number of $3. (repository.pullRequest)" >&2
      exit 1
    fi
    head="$(cat "$FIXTURES/head" 2>/dev/null || git rev-parse HEAD)"
    jq --arg h "$head" '.headRefOid = $h' "$FIXTURES/view.json"
    ;;
  "pr checks")
    if [ ! -s "$FIXTURES/checks.json" ]; then
      echo "no checks reported on the 'branch' branch" >&2
      exit 1
    fi
    cat "$FIXTURES/checks.json"
    ;;
  "api "*)
    if [ ! -f "$FIXTURES/required.json" ]; then
      echo "gh: Not Found (HTTP 404)" >&2
      exit 1
    fi
    jq -r "$4" "$FIXTURES/required.json"
    ;;
  "issue view")
    if [ ! -f "$FIXTURES/issue-$3" ]; then
      echo "GraphQL: Could not resolve to an issue or pull request with the number of $3. (repository.issue)" >&2
      exit 1
    fi
    cat "$FIXTURES/issue-$3"
    ;;
  "pr ready")
    echo "$3" >>"$CALLS/ready.log"
    ;;
  *)
    echo "fake gh: unhandled invocation: $*" >&2
    exit 1
    ;;
esac
FAKE
chmod +x "$work/bin/gh"

export FIXTURES="$work/fixtures"
export CALLS="$work/calls"
export PATH="$work/bin:$PATH"

repo="$work/repo"
g() { git -C "$repo" "$@"; }
g init -q -b main
g config user.email "test@example.com"
g config user.name "Test"
mkdir -p "$repo/docs" "$repo/scripts"
echo "# Lessons" >"$repo/docs/lessons.md"
g add -A && g commit -q -m init
g remote add origin "$repo"

branch() {
  g checkout -q main
  g checkout -q -b "$1"
  for f in "${@:2}"; do
    mkdir -p "$(dirname "$repo/$f")"
    echo "$1" >>"$repo/$f"
    g add "$f"
  done
  g commit -q -m "$1"
}
branch code scripts/x.sh docs/lessons.md
branch docs docs/guide.md docs/lessons.md
branch nolessons scripts/x.sh
branch behind scripts/x.sh docs/lessons.md
GIT_COMMITTER_DATE="2026-01-01T00:00:00Z" branch early scripts/x.sh docs/lessons.md .claude/skills/s/SKILL.md
GIT_COMMITTER_DATE="2026-01-01T00:00:00Z" branch late scripts/x.sh docs/lessons.md
mkdir -p "$repo/.claude/skills/s"
echo fix >"$repo/.claude/skills/s/SKILL.md"
echo fix >>"$repo/scripts/x.sh"
g add -A && GIT_COMMITTER_DATE="2026-01-03T00:00:00Z" g commit -q -m "review fix"

# Lessons waiting on issues (#2968): only the list under the heading counts.
waits_on() {
  g checkout -q main
  g checkout -q -b "$1"
  printf '# Lessons\n\n- A rule naming (issue #9).\n\n## Lessons\n\n- 2026-01-01, #1: a lesson %s.\n' "$2" >"$repo/docs/lessons.md"
  g commit -q -am "$1"
}
waits_on wait-open "(issue #8)"
waits_on wait-own "(issue #42)"
waits_on wait-closed "(issue #9)"
waits_on wait-unread "(issue #5)"
printf OPEN >"$FIXTURES/issue-8"
printf CLOSED >"$FIXTURES/issue-9"

pass_checks='[{"name":"Test","bucket":"pass"},{"name":"Alert","bucket":"skipping"}]'
review_comment='{"author":{"login":"owner"},"createdAt":"2099-01-01T00:00:00Z","body":"**Self-review by the reviewer agent**\nNo findings."}'
good_body='Closes #42.'
req_ok='{"contexts":["Test"],"checks":[{"context":"Test","app_id":1}]}'
req="$req_ok"

# Sets the fixtures from the passing case with one change, then runs.
case_run() {
  local on="$1" checks="$2" comments="$3" body="$4"
  g checkout -q "$on"
  rm -f "$FIXTURES/head" "$CALLS/ready.log"
  printf '%s' "$checks" >"$FIXTURES/checks.json"
  jq -n --argjson c "[$comments]" --arg b "$body" \
    '{author: {login: "owner"}, body: $b, comments: $c}' >"$FIXTURES/view.json"
  [ -z "${5:-}" ] || printf '%s' "$5" >"$FIXTURES/head"
  rm -f "$FIXTURES/required.json"
  [ -z "$req" ] || printf '%s' "$req" >"$FIXTURES/required.json"
  req="$req_ok"
  set +e
  out="$(cd "$repo" && "$script" 7 42 2>&1)"
  run_status=$?
  set -e
}

pass=0
fail=0

expect_refused() {
  if [ "$run_status" -ne 0 ] && grep -qF -- "$2" <<<"$out" && [ ! -f "$CALLS/ready.log" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s: expected a refusal naming "%s"\n    status=%s output=%s\n' "$1" "$2" "$run_status" "$out" >&2
  fi
}

expect_ready() {
  if [ "$run_status" -eq 0 ] && grep -qx 7 "$CALLS/ready.log" 2>/dev/null; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s: expected gh pr ready 7\n    status=%s output=%s\n' "$1" "$run_status" "$out" >&2
  fi
}

case_run code "$pass_checks" "$review_comment" "$good_body"
expect_ready "code change, every box met"

case_run docs "$pass_checks" "" "$good_body"
expect_ready "documentation only, no review comment"

case_run code "$pass_checks" "$review_comment" "Fixes #42"
expect_ready "Fixes counts as closing"

case_run code "$pass_checks" "$review_comment" "$good_body" "0000000000000000000000000000000000000000"
expect_refused "local commit is not the PR's" "last commit is not the PR's"

case_run code "" "$review_comment" "$good_body"
expect_refused "no checks yet" "no checks on the last commit"

case_run code '[{"name":"Test","bucket":"pass"},{"name":"Native iOS","bucket":"pending"}]' "$review_comment" "$good_body"
expect_refused "a pending check" "Native iOS (pending)"

case_run code '[{"name":"Clippy","bucket":"fail"}]' "$review_comment" "$good_body"
expect_refused "a failed check" "Clippy (fail)"

req='{"contexts":["Test","MSRV"],"checks":[{"context":"Test"}]}'
case_run code "$pass_checks" "$review_comment" "$good_body"
expect_refused "a required check not reported" "not reported on the last commit: MSRV"

req='{"checks":[{"context":"Test"},{"context":"Clippy"}]}'
case_run code "$pass_checks" "$review_comment" "$good_body"
expect_refused "a required check named only under checks" "not reported on the last commit: Clippy"

req=""
case_run code "$pass_checks" "$review_comment" "$good_body"
expect_refused "main's protection unreadable" "could not read main's required checks"

case_run code "$pass_checks" "" "$good_body"
expect_refused "code change, no comments" "no review comment"

case_run code "$pass_checks" '{"author":{"login":"codecov"},"body":"## Codecov Report\nPlease review."}' "$good_body"
expect_refused "only another account's comment" "no review comment"

case_run code "$pass_checks" '{"author":{"login":"owner"},"body":"Thanks.\nThe review is below."}' "$good_body"
expect_refused "review named only below the first line" "no review comment"

case_run code "$pass_checks" "$review_comment" "Part of #42."
expect_refused "body does not close" "does not close #42"

case_run code "$pass_checks" "$review_comment" "Closes #420."
expect_refused "body closes a longer number" "does not close #42"

case_run code "$pass_checks" "$review_comment" "Part of #42.
<!-- Closes #42 -->"
expect_refused "closing line inside a comment" "does not close #42"

case_run code "$pass_checks" "$review_comment" "Part of #42.
\`\`\`
Closes #42
\`\`\`"
expect_refused "closing line inside a code block" "does not close #42"

case_run nolessons "$pass_checks" "$review_comment" "$good_body"
expect_ready "no lessons commit, since lessons are optional"

case_run wait-open "$pass_checks" "$review_comment" "$good_body"
expect_ready "a lesson waiting on an open issue, and a rule above the list naming a closed one"

case_run wait-own "$pass_checks" "$review_comment" "$good_body"
expect_refused "a lesson waiting on the issue this PR closes" "waits on #42, which this PR closes; archive it with \"issue:\" in front, or with \"fix:\" and the file"

case_run wait-closed "$pass_checks" "$review_comment" "$good_body"
expect_refused "a lesson waiting on a closed issue" "waits on #9, which has closed; merge main first, and if it is still there, archive it with \"issue:\" in front, or with \"fix:\" and the file"

case_run wait-unread "$pass_checks" "$review_comment" "$good_body"
expect_refused "a lesson waiting on an issue that cannot be read" "could not read #5, which a lesson waits on"

reviewed_on() { jq -c --arg t "$1" '.createdAt = $t' <<<"$review_comment"; }
case_run early "$pass_checks" "$(reviewed_on 2026-01-02T00:00:00Z)" "$good_body"
expect_ready "lessons and an agent instruction fix committed before the review"

case_run late "$pass_checks" "$(reviewed_on 2026-01-02T00:00:00Z)" "$good_body"
expect_refused "agent instructions edited after the review" "after the newest review comment: .claude/skills/s/SKILL.md; review"

case_run late "$pass_checks" "$(reviewed_on 2026-01-02T00:00:00Z),$(reviewed_on 2026-01-04T00:00:00Z)" "$good_body"
expect_ready "a newer review comment clears the late edit"

g remote set-url origin "$work/nowhere"
case_run code "$pass_checks" "$review_comment" "$good_body"
expect_refused "main cannot be fetched" "could not fetch main"
g remote set-url origin "$repo"

# Main moves on after the branch point; only a CI change makes the branch stale.
on_main() {
  g checkout -q main
  mkdir -p "$(dirname "$repo/$1")"
  echo main >>"$repo/$1"
  g add "$1" && g commit -q -m "main: $1"
}
on_main docs/guide.md
case_run behind "$pass_checks" "$review_comment" "$good_body"
expect_ready "main changed only a doc since the branch point"

on_main .github/workflows/ci.yml
case_run behind "$pass_checks" "$review_comment" "$good_body"
expect_refused "main changed a workflow since the branch point" "main changed CI since this branch started (.github/workflows/ci.yml)"

for args in "7x 42" "7 42;true" " 42"; do
  rm -f "$CALLS/ready.log"
  set +e
  out="$(cd "$repo" && "$script" "${args% *}" "${args##* }" 2>&1)"
  run_status=$?
  set -e
  expect_refused "arguments \"$args\"" "both numbers"
done

rm -f "$FIXTURES/view.json" "$CALLS/ready.log"
set +e
out="$(cd "$repo" && "$script" 7 42 2>&1)"
run_status=$?
set -e
expect_refused "an unknown PR" "Could not resolve"

printf '\n%s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
