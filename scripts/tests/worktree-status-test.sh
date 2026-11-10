#!/usr/bin/env bash
# Self-test for scripts/worktree-status.sh (#2783). A real repo in a temp
# folder holds one worktree per state, a fake `gh` answers the GraphQL query
# from per-branch fixtures, and a fake lease script says which worktree a
# session holds. Each rule of whose move it is gets a row, then the count and
# GitHub failing.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/worktree-status.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bin" "$tmp/fixtures"

# ── The fake gh: one alias per -f bN / -F nN argument ───────────────────────

cat >"$tmp/bin/gh" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail
echo call >>"$FIXTURES/calls.txt"
if [ -n "${FAIL_GH:-}" ]; then
  echo "gh: Could not resolve host: api.github.com" >&2
  exit 1
fi
printf '%s\n' "$*" >"$FIXTURES/query.txt"
repo='{}'
while [ $# -gt 0 ]; do
  case "$1" in
    -f | -F)
      key="${2%%=*}" val="${2#*=}"
      case "$key" in
        b*) repo=$(jq -c --arg a "w${key#b}" --arg b "$val" --slurpfile p "$FIXTURES/prs.json" \
          '. + {($a): {nodes: [($p[0][$b] // empty)]}}' <<<"$repo") ;;
        n*) repo=$(jq -c --arg a "i${key#n}" --arg n "$val" --slurpfile i "$FIXTURES/issues.json" \
          '. + {($a): ($i[0][$n] // null)}' <<<"$repo") ;;
      esac
      shift 2
      ;;
    *) shift ;;
  esac
done
jq -c '{data: {repository: .}}' <<<"$repo"
FAKE
chmod +x "$tmp/bin/gh"

# ── The fake lease script: one worktree is held ─────────────────────────────

cat >"$tmp/bin/lease" <<'FAKE'
#!/usr/bin/env bash
printf '%-42s %-28s %s\n' "$MAIN_NAME" main "in use by aaaaaaaa (main, never blocks)"
printf '%-42s %-28s %s\n' held-1005 held-1005 "LEASED by 3f2a91c0 (pid 42) since today"
printf '%-42s %-28s %s\n' merged-1001 merged-1001 "free (stale lease by bbbbbbbb)"
FAKE
chmod +x "$tmp/bin/lease"

export FIXTURES="$tmp/fixtures"
export PATH="$tmp/bin:$PATH"
export WORKTREE_LEASE="$tmp/bin/lease"
export MAIN_NAME=main
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t

# ── A repo with a remote main and one worktree per state ────────────────────

git init -q --bare "$tmp/origin.git"
git init -q -b main "$tmp/main"
cd "$tmp/main"
git commit -q --allow-empty -m base
git remote add origin "$tmp/origin.git"
git push -q origin main
git fetch -q origin

wt() { git worktree add -q -b "$1" "$tmp/$1" main; }
commit() { git -C "$tmp/$1" commit -q --allow-empty -m "$2"; }
head_of() { git -C "$tmp/$1" rev-parse HEAD; }

wt merged-1001
wt draft-1002 && commit draft-1002 work
wt ready-1003 && commit ready-1003 work
wt dirty-1004 && touch "$tmp/dirty-1004/stray.txt"
wt held-1005 && commit held-1005 work
wt nopr && commit nopr work
wt unpushed-1007 && commit unpushed-1007 pushed
pushed_oid="$(head_of unpushed-1007)"
commit unpushed-1007 "not pushed"
wt closed-1008 && commit closed-1008 work
wt atmain
git worktree add -q --detach "$tmp/detached-1010" main
wt extended-1011 && commit extended-1011 merged
merged_oid="$(head_of extended-1011)"
commit extended-1011 "after the merge"
wt greendraft-1012 && commit greendraft-1012 work
wt changes-1013 && commit changes-1013 work
wt behind-1014 && commit behind-1014 one
behind_base="$(head_of behind-1014)"
commit behind-1014 two
behind_oid="$(head_of behind-1014)"
git -C "$tmp/behind-1014" reset -q --hard "$behind_base"
wt titled

pr() { # branch number state isDraft rollup review oid [closingIssue] [title]
  local closing='[]'
  [ -n "${8:-}" ] && closing="[{\"number\":$8,\"title\":\"Issue $8 title\"}]"
  jq -c --arg b "$1" --argjson n "$2" --arg s "$3" --argjson d "$4" --arg r "$5" \
    --arg rv "$6" --arg o "$7" --argjson c "$closing" --arg t "${9:-PR $2 title}" \
    '. + {($b): {number: $n, title: $t, state: $s, isDraft: $d,
      reviewDecision: (if $rv == "" then null else $rv end), headRefOid: $o,
      closingIssuesReferences: {nodes: $c},
      commits: {nodes: [{commit: {statusCheckRollup:
        (if $r == "" then null else {state: $r} end)}}]}}}' \
    "$FIXTURES/prs.json" >"$FIXTURES/prs.tmp"
  mv "$FIXTURES/prs.tmp" "$FIXTURES/prs.json"
}

echo '{}' >"$FIXTURES/prs.json"
pr merged-1001 2001 MERGED false SUCCESS "" "$(head_of merged-1001)" 1001
pr draft-1002 2002 OPEN true FAILURE "" "$(head_of draft-1002)"
pr ready-1003 2003 OPEN false SUCCESS APPROVED "$(head_of ready-1003)"
pr dirty-1004 2004 MERGED false SUCCESS "" "$(head_of dirty-1004)"
pr held-1005 2005 OPEN true PENDING "" "$(head_of held-1005)"
pr unpushed-1007 2007 OPEN true SUCCESS "" "$pushed_oid"
pr closed-1008 2008 CLOSED false FAILURE "" "$(head_of closed-1008)"
pr extended-1011 2011 MERGED false SUCCESS "" "$merged_oid"
pr greendraft-1012 2012 OPEN true SUCCESS "" "$(head_of greendraft-1012)"
pr changes-1013 2013 OPEN false SUCCESS CHANGES_REQUESTED "$(head_of changes-1013)"
pr behind-1014 2014 OPEN false SUCCESS "" "$behind_oid"
pr titled 2015 MERGED false SUCCESS "" "$(head_of titled)" "" "Fix the thing (#1015)"
cat >"$FIXTURES/issues.json" <<'JSON'
{"1002":{"title":"Issue 1002 title"},"1003":{"title":"Issue 1003 title"},
 "1010":{"title":"Issue 1010 title"}}
JSON

pass=0
fail=0

check() {
  local haystack="$1" desc="$2" needle="$3"
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s\n    expected to find: %s\n' "$desc" "$needle" >&2
  fi
}

refute() {
  local haystack="$1" desc="$2" needle="$3"
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    fail=$((fail + 1))
    printf '✗ %s\n    did not expect: %s\n' "$desc" "$needle" >&2
  else
    pass=$((pass + 1))
  fi
}

# The block for one worktree: its "- name" line and the indented lines after.
row() {
  printf '%s\n' "$1" | awk -v n="- $2" '
    $0 == n { on = 1; print; next }
    on && /^    / { print; next }
    { on = 0 }'
}

# ── Every rule, with GitHub answering ───────────────────────────────────────

: >"$FIXTURES/calls.txt"
out="$("$script")"
calls="$(wc -l <"$FIXTURES/calls.txt" | tr -d ' ')"
if [ "$calls" -eq 1 ]; then pass=$((pass + 1)); else
  fail=$((fail + 1)); printf '✗ reads GitHub in one call, made %s\n' "$calls" >&2; fi
check "$(cat "$FIXTURES/query.txt")" "asks by branch name" "headRefName"
refute "$out" "leaves the main checkout out" "- main"

r="$(row "$out" held-1005)"
check "$r" "a held worktree is an agent's, whatever its PR says" "an agent: session 3f2a91c0 is working in it"

r="$(row "$out" dirty-1004)"
check "$r" "uncommitted work nobody holds is yours" "you: 1 uncommitted file and no session holds it"
check "$r" "says the PR merged" "PR #2004 merged"

r="$(row "$out" unpushed-1007)"
check "$r" "a commit on neither the PR nor main is yours" "you: 1 commit not on the PR or main"

r="$(row "$out" merged-1001)"
check "$r" "a merged PR with a clean tree is finished" "finished, safe to remove"
check "$r" "takes the issue from the PR's closing reference" "#1001 Issue 1001 title"
refute "$r" "a stale lease is not a session" "session bbbbbbbb"

r="$(row "$out" atmain)"
check "$r" "no PR and nothing beyond main is finished" "finished, safe to remove"
check "$r" "says it has no PR" "no PR, nothing beyond main"
check "$r" "says when there is no issue" "no issue, branch atmain"

r="$(row "$out" detached-1010)"
check "$r" "a detached worktree takes its issue from the folder name" "#1010 Issue 1010 title"
check "$r" "a detached worktree at main is finished" "finished, safe to remove"

r="$(row "$out" closed-1008)"
check "$r" "a PR closed unmerged is yours" "you: PR closed without merging"

r="$(row "$out" nopr)"
check "$r" "commits with no PR are yours" "you: no PR for 1 commit"

r="$(row "$out" ready-1003)"
check "$r" "a ready, green PR is yours to merge" "you: ready to merge"
check "$r" "shows the review" "approved"
check "$r" "takes the issue from the branch name" "#1003 Issue 1003 title"

r="$(row "$out" draft-1002)"
check "$r" "a draft with failing CI is an agent's" "an agent: CI failed on the last commit"
check "$r" "says it is a draft" "PR #2002 draft, CI failed"

r="$(row "$out" extended-1011)"
check "$r" "a commit after a merged PR is yours, not safe to remove" "you: 1 commit not on the PR or main"

r="$(row "$out" greendraft-1012)"
check "$r" "a green draft is an agent's, not ready to merge" "an agent: draft, waiting on review"

r="$(row "$out" changes-1013)"
check "$r" "changes requested is an agent's, even when green" "an agent: changes requested"

r="$(row "$out" behind-1014)"
check "$r" "a worktree behind its PR counts as on the PR" "you: ready to merge"

r="$(row "$out" titled)"
check "$r" "takes the issue from a (#N) at the end of the PR title" "#1015 Fix the thing"

check "$out" "counts what is safe to remove" "4 of 15 worktrees are safe to remove."
check "$out" "says it removed nothing" "This removed nothing"
check "$out" "groups your moves" "Your move (7)"
check "$out" "groups agents' moves" "An agent's move (4)"

# ── GitHub refusing the read ────────────────────────────────────────────────

out="$(FAIL_GH=1 "$script")"
check "$out" "names the outage" "GitHub did not answer"
check "$out" "carries GitHub's own words" "Could not resolve host"
check "$(row "$out" held-1005)" "a held worktree still reads from the lease" "an agent: session"
check "$(row "$out" dirty-1004)" "uncommitted work still reads from git" "you: 1 uncommitted file"
check "$(row "$out" draft-1002)" "a PR state it could not read is unknown" "unknown: GitHub did not answer"
check "$(row "$out" atmain)" "a tree at main is finished on git alone" "finished, safe to remove"
check "$out" "counts only what git alone proves" "4 of 15 worktrees are safe to remove, on git alone."
refute "$out" "never claims GitHub was read when it was not" "read from GitHub"

# ── A lease script that answers nothing ─────────────────────────────────────

out="$(WORKTREE_LEASE=/usr/bin/false "$script")"
check "$out" "says the session list did not answer" "The session list did not answer"
check "$out" "calls nothing safe when sessions are unknown" "0 of 15 worktrees are safe to remove."
check "$(row "$out" atmain)" "a tree at main is unknown, not finished" "unknown: the session list did not answer"
out="$(MAIN_NAME=elsewhere "$script")"
check "$out" "a list with no row for the main checkout is not trusted" "0 of 15 worktrees are safe to remove."

# ── No lease script on this machine ─────────────────────────────────────────

out="$(WORKTREE_LEASE="$tmp/missing" "$script")"
check "$out" "says sessions are unknown without the lease script" "no worktree-lease.sh"
check "$(row "$out" held-1005)" "falls back to the PR without the lease" "an agent: CI running"

# ── No worktrees at all ─────────────────────────────────────────────────────

git init -q -b main "$tmp/lonely"
git -C "$tmp/lonely" commit -q --allow-empty -m base
out="$(cd "$tmp/lonely" && "$script")"
check "$out" "says there are none, and makes no GitHub call" "No worktrees beside the main checkout."

printf '\n%s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
