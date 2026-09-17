#!/usr/bin/env bash
# Self-test for scripts/handover.sh (#1986): the opener has to carry the facts
# a cold session cannot reconstruct, and must refuse rather than guess when it
# cannot read them. The fake `gh` holds real JSON and applies whatever `-q`
# expression the script passes with real jq, so a broken jq program fails here
# instead of being swallowed by a fake that echoes text.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/fixtures" "$work/repo/.claude" "$work/repo/scripts/lib"

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

cp "$root/scripts/handover.sh" "$work/repo/scripts/handover.sh"
cp "$root/scripts/lib/claim-branch.sh" "$work/repo/scripts/lib/claim-branch.sh"
script="$work/repo/scripts/handover.sh"

cat >"$work/repo/.claude/settings.json" <<'JSON'
{ "model": "sonnet[1m]", "effortLevel": "medium" }
JSON

cat >"$work/fixtures/issue_1986.json" <<'JSON'
{
  "title": "Cut the cold context bill",
  "state": "OPEN",
  "url": "https://github.com/jonyardley/intrada/issues/1986",
  "labels": [{ "name": "harness" }, { "name": "tier-2" }],
  "parent": { "number": 1967, "title": "Spend less on the harness" },
  "comments": [{ "body": "Claimed: branch `cold-context`, by a session in its own worktree." }]
}
JSON

cat >"$work/fixtures/issue_1990.json" <<'JSON'
{
  "title": "An issue nobody has claimed",
  "state": "OPEN",
  "url": "https://github.com/jonyardley/intrada/issues/1990",
  "labels": [],
  "comments": []
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
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "✗ $name: expected to find '$needle'" >&2
    printf '%s\n' "$haystack" | sed 's/^/    /' >&2
  fi
}
refute() {
  local name=$1 haystack=$2 needle=$3
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    fail=$((fail + 1))
    echo "✗ $name: did not expect '$needle'" >&2
  else
    pass=$((pass + 1))
  fi
}

# ── an explicit number ───────────────────────────────────────────────────────

out="$(bash "$script" 1986)"
check "names the issue and title" "$out" "Pick up #1986 in intrada: Cut the cold context bill"
check "carries the model from settings, without the window suffix" "$out" "/model sonnet"
check "carries the effort from settings" "$out" "/effort medium"
refute "strips the context window off the model" "$out" "sonnet[1m]"
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
refute "does not tell it to branch fresh over the work" "$out" "Make your own worktree"

# ── an unclaimed issue says so, rather than implying a claim ──────────────────

out="$(bash "$script" 1990)"
check "an unclaimed issue names the claim step" "$out" "not claimed yet, so run just claim 1990"
refute "an issue with no epic says nothing about one" "$out" "epic #"

# ── nothing claimed and nothing committed starts a worktree ──────────────────

git checkout -q --no-track -b clean-slate origin/main
out="$(bash "$script" 1990)"
check "a cold branch sends the next session to its own worktree" "$out" "Make your own worktree"
refute "a cold branch does not point at this one" "$out" "carry on there"
check "a cold branch is level with main" "$out" "0 commit(s) ahead of origin/main"
git checkout -q cold-context

# ── pushed is the commits that are not on the remote, not the ref existing ───

git update-ref refs/remotes/origin/cold-context HEAD~1
out="$(bash "$script" 1986)"
check "counts the commits the remote does not have" "$out" "1 commit(s) not pushed"
git update-ref refs/remotes/origin/cold-context HEAD
out="$(bash "$script" 1986)"
check "says pushed when the remote has them all" "$out" "ahead of origin/main, pushed"
refute "does not still say not pushed" "$out" "not pushed"
git update-ref -d refs/remotes/origin/cold-context

# ── unreadable settings ask for the rung by hand, rather than guessing ───────

mv "$work/repo/.claude/settings.json" "$work/settings.json"
out="$(bash "$script" 1986)"
check "says the settings could not be read" "$out" "Set /model and /effort by hand"
refute "guesses no rung" "$out" "/model sonnet"
mv "$work/settings.json" "$work/repo/.claude/settings.json"

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
