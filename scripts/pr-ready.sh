#!/usr/bin/env bash
# Mark a PR ready only once the ship route is done (#2955): a brief that said
# to stop at the draft opened one with no lessons, no review comment and no
# closing line (#2921). Each refusal below replaces a line a session had to
# remember. Run it from the PR's own worktree: the review route and the
# lessons commit are read from the local branch.
#
# Called via `just pr-ready <pr> <issue>`.

set -euo pipefail

pr="${1:?usage: pr-ready.sh <pr> <issue>}"
issue="${2:?usage: pr-ready.sh <pr> <issue>}"

lib_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/diff-scope.sh
source "$lib_dir/lib/diff-scope.sh"
cd "$(git rev-parse --show-toplevel)"

refuse() {
  echo "✗ #$pr is not ready: $*" >&2
  exit 1
}

git fetch -q origin main 2>/dev/null || true
view="$(gh pr view "$pr" --json headRefOid,body,author,comments)"

[ "$(jq -r .headRefOid <<<"$view")" = "$(git rev-parse HEAD)" ] ||
  refuse "this checkout's last commit is not the PR's; push it, or run from the PR's worktree."

# `gh pr checks` fails with "no checks reported" just after a push.
checks="$(gh pr checks "$pr" --json name,bucket 2>&1)" || true
jq -e 'type == "array" and length > 0' <<<"$checks" >/dev/null 2>&1 ||
  refuse "no checks on the last commit yet ($checks)."
waiting="$(jq -r '[.[] | select(.bucket != "pass" and .bucket != "skipping") | "\(.name) (\(.bucket))"] | join(", ")' <<<"$checks")"
[ -z "$waiting" ] || refuse "checks not passed on the last commit: $waiting."

# A review comment is one by the PR's author whose first line names a review,
# which `just pr-open --review-file` posts; documentation only needs none.
reviews="$(jq -r '.author.login as $a | [.comments[] | select(.author.login == $a and (.body | split("\n")[0] | test("review"; "i")))] | length' <<<"$view")"
[[ "$(diff_scope_review_route)" == none:* ]] || [ "$reviews" -gt 0 ] ||
  refuse "no review comment (a comment by the PR's author whose first line names the review)."

body="$(jq -r .body <<<"$view")"
grep -qiE "(^|[^[:alnum:]])(close[sd]?|fix(e[sd])?|resolve[sd]?):? +#$issue([^0-9]|$)" <<<"$body" ||
  refuse "the body does not close #$issue; add \"Closes #$issue\"."

base="$(git merge-base origin/main HEAD)"
git diff --quiet "$base" HEAD -- docs/lessons.md docs/lessons-archive.md &&
  refuse "no lessons commit on this branch; run the /ship retro first."

gh pr ready "$pr"
