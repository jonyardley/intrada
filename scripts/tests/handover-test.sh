#!/usr/bin/env bash
# Self-test for scripts/handover.sh (#1986): the opener has to carry the facts
# a cold session cannot reconstruct, and must refuse rather than guess when it
# cannot read them. The fake `gh` holds real JSON and applies whatever `-q`
# expression the script passes with real jq, so a broken jq program fails here
# instead of being swallowed by a fake that echoes text.
set -euo pipefail
source "$(dirname "$0")/../lib/contains.sh"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/fixtures" "$work/repo/scripts/lib"

cat >"$work/bin/gh" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail

cmd="$1"; shift
sub="${1:-}"

args=("$@")
q=""
state=""
for i in "${!args[@]}"; do
  if [ "${args[$i]}" = "-q" ]; then q="${args[$((i + 1))]}"; fi
  if [ "${args[$i]}" = "--state" ]; then state="${args[$((i + 1))]}"; fi
done

emit() {
  if [ -n "$q" ]; then printf '%s' "$1" | jq -r "$q"; else printf '%s' "$1"; fi
}

case "$cmd $sub" in
  "repo view")
    echo "jonyardley/intrada"
    ;;
  "pr list")
    prs="$(cat "$FIXTURES/prs.json")"
    if [ "$state" = "open" ]; then
      prs="$(printf '%s' "$prs" | jq '[.[] | select(.state == "OPEN")]')"
    fi
    emit "$prs"
    ;;
  "issue view")
    number="$2"
    file="$FIXTURES/issue_$number.json"
    [ -f "$file" ] || exit 1
    emit "$(cat "$file")"
    ;;
  *)
    echo "unexpected gh call: $cmd $sub" >&2
    exit 1
    ;;
esac
FAKE
chmod +x "$work/bin/gh"

# The opener's read is `just issue N`, which runs issue-read.sh against the
# fake gh, so the read test sees what the real recipe prints.
cat >"$work/bin/just" <<FAKE
#!/usr/bin/env bash
set -euo pipefail
[ "\$1" = issue ] || { echo "unexpected just call: \$*" >&2; exit 1; }
exec bash "$work/repo/scripts/issue-read.sh" "\$2"
FAKE
chmod +x "$work/bin/just"

cp "$root/scripts/handover.sh" "$work/repo/scripts/handover.sh"
cp "$root/scripts/issue-read.sh" "$work/repo/scripts/issue-read.sh"
cp "$root/scripts/lib/claim-branch.sh" "$work/repo/scripts/lib/claim-branch.sh"
script="$work/repo/scripts/handover.sh"

cat >"$work/fixtures/issue_1986.json" <<'JSON'
{
  "title": "Cut the cold context bill",
  "state": "OPEN",
  "url": "https://github.com/jonyardley/intrada/issues/1986",
  "labels": [{ "name": "harness" }, { "name": "tier-2" }],
  "parent": { "number": 1967, "title": "Spend less on the harness" },
  "comments": [{ "author": { "login": "jonyardley" }, "body": "Claimed: branch `cold-context`, by a session in its own worktree." }]
}
JSON

cat >"$work/fixtures/issue_1990.json" <<'JSON'
{
  "number": 1990,
  "author": { "login": "jonyardley" },
  "title": "An issue nobody has claimed",
  "state": "OPEN",
  "url": "https://github.com/jonyardley/intrada/issues/1990",
  "body": "What you would notice: the opener shows this body.",
  "labels": [],
  "comments": []
}
JSON

cat >"$work/fixtures/issue_1992.json" <<'JSON'
{
  "title": "An issue only a stranger has claimed",
  "state": "OPEN",
  "url": "https://github.com/jonyardley/intrada/issues/1992",
  "labels": [],
  "comments": [{ "author": { "login": "a-stranger" }, "body": "Claimed: branch `cold-context`, by a stranger." }]
}
JSON

cat >"$work/fixtures/issue_1993.json" <<'JSON'
{
  "title": "An issue claimed on another branch",
  "state": "OPEN",
  "url": "https://github.com/jonyardley/intrada/issues/1993",
  "labels": [],
  "comments": [{ "author": { "login": "jonyardley" }, "body": "Claimed: branch `other-work`, by a session in its own worktree." }]
}
JSON

echo '[]' >"$work/fixtures/prs.json"

cd "$work/repo"
git init -q .
git config user.email t@t.t
git config user.name t
echo one >a.txt
git add -A
git -c commit.gpgsign=false commit -qm base
git update-ref refs/remotes/origin/main HEAD
git checkout -qb cold-context
echo two >b.txt
git add -A
git -c commit.gpgsign=false commit -qm "add b"

export PATH="$work/bin:$PATH"
export FIXTURES="$work/fixtures"

pass=0
fail=0
check() {
  local name=$1 haystack=$2 needle=$3
  if contains "$needle" "$haystack"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "✗ $name: expected to find '$needle'" >&2
    printf '%s\n' "$haystack" | sed 's/^/    /' >&2
  fi
}
# The whole line, since a substring match passes a longer line that holds it.
check_line() {
  local name=$1 haystack=$2 line=$3
  if grep -qxF -- "$line" <<<"$haystack"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "✗ $name: expected the line '$line'" >&2
    printf '%s\n' "$haystack" | sed 's/^/    /' >&2
  fi
}
refute() {
  local name=$1 haystack=$2 needle=$3
  if contains "$needle" "$haystack"; then
    fail=$((fail + 1))
    echo "✗ $name: did not expect '$needle'" >&2
  else
    pass=$((pass + 1))
  fi
}

# ── an explicit number ───────────────────────────────────────────────────────

out="$(bash "$script" 1986)"
check "names the issue and title" "$out" "Pick up #1986 in intrada: Cut the cold context bill"
refute "asks for no model change" "$out" "/model"
refute "asks for no effort change" "$out" "/effort"
check "names the epic the issue sits under" "$out" "epic #1967: Spend less on the harness"
check "names the claimed branch" "$out" "claimed on branch cold-context"
check "counts commits ahead of main" "$out" "branch cold-context, 1 commit(s) ahead of origin/main"
check "says the branch is not pushed" "$out" "not pushed"
check "says there is no PR" "$out" "no PR yet"
check "lists the files already touched" "$out" "b.txt"
check "carries the labels" "$out" "labels: harness, tier-2"

# ── work in hand resumes in this worktree, rather than abandoning it ─────────

check "sends the next session to this worktree" "$out" "  cd $(pwd -P)"
check "says the lease releases at session end" "$out" "lease releases when this session ends"
refute "does not tell it to branch fresh over the work" "$out" "just worktree-new"

# ── an unclaimed issue says so, rather than implying a claim ──────────────────

out="$(bash "$script" 1990)"
check "an unclaimed issue says so" "$out" "not claimed yet"
refute "an issue with no epic says nothing about one" "$out" "epic #"

# ── the opener's read shows the body of an issue with no comments (#2969) ────

check_line "the read is the owner-only issue recipe" "$out" "  just issue 1990"
read_out="$(printf '%s\n' "$out" | grep -E '^  just issue ' | while read -ra words; do "${words[@]}"; done || true)"
check "the read shows the body when there are no comments" "$read_out" "the opener shows this body"

# ── a new issue starts from fresh main, even on a branch with commits (#2969) ─

check "a new issue names a fresh worktree" "$out" "just worktree-new"
check_line "a new issue is claimed in the new worktree" "$out" "  just claim 1990"
refute "a new issue does not carry on in this worktree" "$out" "carry on there"
refute "a new issue does not point at this worktree" "$out" "cd $(pwd -P) &&"
refute "a new issue does not list this branch's files" "$out" "Files already touched"

# ── nothing claimed and nothing committed starts a worktree ──────────────────

git checkout -q --no-track -b clean-slate origin/main
out="$(bash "$script" 1990)"
check "a cold branch sends the next session to its own worktree" "$out" "Start from fresh main"
refute "a cold branch does not point at this one" "$out" "carry on there"
check_line "a cold branch names the claim step" "$out" "  just claim 1990"
git checkout -q cold-context

# ── a stranger's claim comment is not a claim ─────────────────────────────────

out="$(bash "$script" 1992)"
check "a stranger's claim is ignored" "$out" "not claimed yet"
refute "a stranger's claim does not carry on here" "$out" "carry on there"
check_line "a stranger's claim still asks for a claim" "$out" "  just claim 1992"

# ── an issue claimed on another branch goes to that branch's worktree ────────

out="$(bash "$script" 1993)"
check "names the other branch" "$out" "claimed on branch other-work"
check "sends the session to that branch's worktree" "$out" "Work in that"
refute "another branch's claim does not carry on here" "$out" "carry on there"
refute "another branch's claim does not start fresh" "$out" "just worktree-new"
refute "another branch's claim is not claimed again" "$out" "just claim 1993"

# ── a merged branch's own issue also starts from fresh main (#2969) ──────────

cat >"$work/fixtures/prs.json" <<'JSON'
[
  { "number": 1995, "title": "Cut the cold context bill (#1986)", "state": "MERGED", "isDraft": false, "url": "https://github.com/jonyardley/intrada/pull/1995" }
]
JSON
out="$(bash "$script" 1986)"
check "a merged branch names a fresh worktree" "$out" "just worktree-new"
refute "a merged branch does not carry on there" "$out" "carry on there"
refute "a merged branch does not point at this worktree" "$out" "cd $(pwd -P) &&"
check_line "a merged branch's claim is posted afresh" "$out" '  Claimed: branch `<new-branch>`'
check_line "a merged branch's issue is claimed after that" "$out" "  just claim 1986"

# Merged even when a closed PR is listed first.
cat >"$work/fixtures/prs.json" <<'JSON'
[
  { "number": 1980, "title": "An earlier attempt (#1986)", "state": "CLOSED", "isDraft": false, "url": "https://github.com/jonyardley/intrada/pull/1980" },
  { "number": 1995, "title": "Cut the cold context bill (#1986)", "state": "MERGED", "isDraft": false, "url": "https://github.com/jonyardley/intrada/pull/1995" }
]
JSON
out="$(bash "$script" 1986)"
refute "any merged PR ends the branch's work" "$out" "carry on there"
echo '[]' >"$work/fixtures/prs.json"

# ── pushed is the commits that are not on the remote, not the ref existing ───

git update-ref refs/remotes/origin/cold-context HEAD~1
out="$(bash "$script" 1986)"
check "counts the commits the remote does not have" "$out" "1 commit(s) not pushed"
git update-ref refs/remotes/origin/cold-context HEAD
out="$(bash "$script" 1986)"
check "says pushed when the remote has them all" "$out" "ahead of origin/main, pushed"
refute "does not still say not pushed" "$out" "not pushed"
git update-ref -d refs/remotes/origin/cold-context

# ── the number defaults to an open PR's title ────────────────────────────────

cat >"$work/fixtures/prs.json" <<'JSON'
[
  { "number": 1980, "title": "An earlier attempt (#1986)", "state": "CLOSED", "isDraft": false, "url": "https://github.com/jonyardley/intrada/pull/1980" },
  { "number": 1991, "title": "Cut the cold context bill (#1986)", "state": "OPEN", "isDraft": true, "url": "https://github.com/jonyardley/intrada/pull/1991" }
]
JSON
out="$(bash "$script")"
check "derives the number from the PR title" "$out" "Pick up #1986"
check "names the open PR and its state" "$out" "PR #1991, draft"
refute "skips the closed PR that comes first" "$out" "PR #1980"

# ── it refuses rather than guesses ───────────────────────────────────────────

echo '[]' >"$work/fixtures/prs.json"
if out="$(bash "$script" 2>&1)"; then
  fail=$((fail + 1))
  echo "✗ no number and no PR should exit non-zero" >&2
else
  pass=$((pass + 1))
  check "says why it cannot work out the number" "$out" "handover needs an issue number"
fi

if out="$(bash "$script" 4242 2>&1)"; then
  fail=$((fail + 1))
  echo "✗ an unreadable issue should exit non-zero" >&2
else
  pass=$((pass + 1))
  check "says which issue it could not read" "$out" "cannot read issue #4242"
fi

if out="$(bash "$script" nineteen-eighty-six 2>&1)"; then
  fail=$((fail + 1))
  echo "✗ a non-numeric argument should exit non-zero" >&2
else
  pass=$((pass + 1))
  check "names the usage for a non-numeric argument" "$out" "Usage:"
  check "the usage names the one argument it takes" "$out" "[issue-number]"
fi

if out="$(bash "$script" 1986 1990 2>&1)"; then
  fail=$((fail + 1))
  echo "✗ a second argument should exit non-zero" >&2
else
  pass=$((pass + 1))
  check "names the usage for a second argument" "$out" "Usage:"
fi

git update-ref -d refs/remotes/origin/main
if out="$(bash "$script" 1986 2>&1)"; then
  fail=$((fail + 1))
  echo "✗ a missing origin/main should exit non-zero" >&2
else
  pass=$((pass + 1))
  check "says origin/main is missing" "$out" "no origin/main ref"
  check "names the fix for a missing origin/main" "$out" "git fetch origin"
fi
git update-ref refs/remotes/origin/main "$(git rev-parse HEAD~1)"

if [ "$fail" -eq 0 ]; then
  echo "✓ handover ($pass checks)"
else
  echo "✗ handover: $fail failed, $pass passed" >&2
fi
exit "$fail"
