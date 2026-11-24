#!/usr/bin/env bash
# Mark a PR ready only once the ship route is done (#2955): a brief that said
# to stop at the draft opened one with no lessons, no review comment and no
# closing line (#2921). Each refusal below replaces a line a session had to
# remember. Run it from the PR's own worktree: the review route and the
# lessons commit are read from the local branch.
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

# The review route and the lessons commit both diff against main.
git fetch -q origin main || refuse "could not fetch main."
view="$(gh pr view "$pr" --json headRefOid,body,author,comments)"

[ "$(jq -r .headRefOid <<<"$view")" = "$(git rev-parse HEAD)" ] ||
  refuse "this checkout's last commit is not the PR's; push it, or run from the PR's worktree."

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

# The retro comes before the review (#2962), so an edit to agent instructions
# after the newest review comment, a review fix say, gets a second look.
base="$(git merge-base origin/main HEAD)"
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

lessons=0
git diff --quiet "$base" HEAD -- docs/lessons.md docs/lessons-archive.md || lessons=$?
case "$lessons" in
  1) ;;
  0) refuse "no lessons commit on this branch; write the retro, the route's first step." ;;
  *) refuse "could not compare the lessons files (git diff exited $lessons)." ;;
esac

gh pr ready "$pr"
