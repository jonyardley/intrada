#!/usr/bin/env bash
# Mark a PR ready only once the ship route is done (#2955): a brief that said
# to stop at the draft opened one with no review comment and no closing line
# (#2921). Each refusal below replaces a line a session had to remember. Run it
# from the PR's own worktree: the review route is read from the local branch.
# A branch needs no lessons commit, since lessons are written only when
# something went wrong (#3034).
#
# Called via `just pr-ready <pr> <issue>`.

set -euo pipefail

pr="${1:-}"
issue="${2:-}"
[[ "$pr" =~ ^[0-9]+$ && "$issue" =~ ^[0-9]+$ ]] || {
  echo "✗ usage: pr-ready.sh <pr> <issue>, both numbers." >&2
  exit 1
}

lib_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/diff-scope.sh
source "$lib_dir/lib/diff-scope.sh"
cd "$(git rev-parse --show-toplevel)"

refuse() {
  echo "✗ #$pr is not ready: $*" >&2
  exit 1
}

# The review route diffs against main.
git fetch -q origin main || refuse "could not fetch main."
view="$(gh pr view "$pr" --json headRefOid,body,author,comments)"

[ "$(jq -r .headRefOid <<<"$view")" = "$(git rev-parse HEAD)" ] ||
  refuse "this checkout's last commit is not the PR's; push it, or run from the PR's worktree."

# A branch that predates a new check never ran it, so main went red after a green merge (#2198).
base="$(git merge-base origin/main HEAD)" || refuse "could not find where this branch left main."
stale="$(git diff --name-only "$base" origin/main -- .github/workflows | paste -sd, - | sed 's/,/, /g')" ||
  refuse "could not compare main's CI files with the branch point."
[ -z "$stale" ] || refuse "main changed CI since this branch started ($stale); merge main into the branch (never rebase) and push."

# `gh pr checks` fails with "no checks reported" just after a push.
checks="$(gh pr checks "$pr" --json name,bucket 2>&1)" || true
jq -e 'type == "array" and length > 0' <<<"$checks" >/dev/null 2>&1 ||
  refuse "no checks on the last commit yet ($checks)."
waiting="$(jq -r '[.[] | select(.bucket != "pass" and .bucket != "skipping") | "\(.name) (\(.bucket))"] | join(", ")' <<<"$checks")"
[ -z "$waiting" ] || refuse "checks not passed on the last commit: $waiting."

# A check that has not reported yet is absent, not pending; main's protection
# names the ones that must be there.
required="$(gh api 'repos/{owner}/{repo}/branches/main/protection/required_status_checks' \
  --jq '[.contexts[]?, .checks[]?.context] | unique | .[]')" && [ -n "$required" ] ||
  refuse "could not read main's required checks."
missing="$(jq -r --arg req "$required" '($req | split("\n")) - [.[].name] | join(", ")' <<<"$checks")"
[ -z "$missing" ] || refuse "required checks not reported on the last commit: $missing."

# A review comment is one by the PR's author whose first line names a review,
# which `just pr-open --review-file` posts; documentation only needs none.
mine='.author.login as $a | [.comments[] | select(.author.login == $a and (.body | split("\n")[0] | test("review"; "i")))]'
reviews="$(jq -r "$mine | length" <<<"$view")"
[[ "$(diff_scope_review_route)" == none:* ]] || [ "$reviews" -gt 0 ] ||
  refuse "no review comment (a comment by the PR's author whose first line names the review)."

# An edit to agent instructions after the newest review comment, such as a
# review fix, gets a second look (#2962).
if [ "$reviews" -gt 0 ]; then
  reviewed="$(jq -r "$mine | map(.createdAt | fromdateiso8601) | max" <<<"$view")" ||
    refuse "could not read the review comment's time."
  late=""
  while IFS= read -r line; do
    case "$line" in
      @*) at="${line#@}" ;;
      ?*) [ "$at" -le "$reviewed" ] || ! diff_scope_is_agents "$line" || late="$late$line"$'\n' ;;
    esac
  done < <(git log --format=@%ct --name-only "$base"..HEAD)
  late="$(sort -u <<<"$late" | grep -v '^$' | paste -sd, - | sed 's/,/, /g')" || true
  [ -z "$late" ] || refuse "agent instructions changed after the newest review comment: $late; review them again."
fi

body="$(jq -r .body <<<"$view" | perl -0pe 's/<!--.*?-->//gs; s/```.*?```//gs')"
grep -qiE "(^|[^[:alnum:]])(close[sd]?|fix(e[sd])?|resolve[sd]?):? +#$issue([^0-9]|$)" <<<"$body" ||
  refuse "the body does not close #$issue; add \"Closes #$issue\"."

# A printed lesson waiting on a closed issue stayed printed until the cap
# (#2968), so archive it now: the one this PR closes, or one closed already.
waits="$(awk '/^## Lessons$/ { on = 1; next } on && /^- / && match($0, /\(issue #[0-9]+\)/) { print substr($0, RSTART + 8, RLENGTH - 9) }' docs/lessons.md | sort -un)" ||
  refuse "could not read docs/lessons.md."
for n in $waits; do
  [ "$n" != "$issue" ] || refuse "a lesson in docs/lessons.md waits on #$n, which this PR closes; archive it with \"issue:\" in front, or with \"fix:\" and the file when this PR moves it into a doc."
  state="$(gh issue view "$n" --json state --jq .state 2>&1)" || refuse "could not read #$n, which a lesson waits on ($state)."
  [ "$state" != CLOSED ] || refuse "a lesson in docs/lessons.md waits on #$n, which has closed; merge main first, and if it is still there, archive it with \"issue:\" in front, or with \"fix:\" and the file when this PR moves it into a doc."
done

gh pr ready "$pr"
