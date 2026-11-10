#!/usr/bin/env bash
# Print every worktree beside the main checkout with its issue, its PR, CI on
# the PR's last commit, review state, local state and whose move it is (#2783).
#
# Reports only: it never removes, fetches or writes anything. GitHub is read in
# one GraphQL call with an alias per branch, which answered 19 branches in
# about a second where listing every PR with its checks took 17.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

repo="jonyardley/intrada"
main_ref="origin/main"
lease="${WORKTREE_LEASE:-$HOME/.claude/hooks/worktree-lease.sh}"
sep=$'\x1f'

for tool in gh jq; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "❌ $tool is required" >&2
    exit 1
  }
done

# The first block of the porcelain list is the main checkout, which is never
# removable, so it is dropped. A folder that has gone is git's to prune.
main_name="$(basename "$(git worktree list --porcelain | sed -n '1s/^worktree //p')")"
paths=()
branches=()
while IFS="$sep" read -r path branch; do
  [ -d "$path" ] || continue
  paths+=("$path")
  branches+=("$branch")
done < <(git worktree list --porcelain | awk -v s="$sep" '
  /^worktree / { n++; path[n] = substr($0, 10) }
  /^branch refs\/heads\// { branch[n] = substr($0, 19) }
  END { for (i = 2; i <= n; i++) print path[i] s branch[i] }')

if [ "${#paths[@]}" -eq 0 ]; then
  echo "No worktrees beside the main checkout."
  exit 0
fi

# A branch or folder ending in an issue number names its issue, as
# `just worktree-new` names them.
issue_in() {
  printf '%s' "$1" | sed -nE 's/.*-([0-9]{3,})$/\1/p'
}

nums=()
decl=""
fields=""
args=()
pr_fields='number title state isDraft reviewDecision headRefOid
  closingIssuesReferences(first: 1) { nodes { number title } }
  commits(last: 1) { nodes { commit { statusCheckRollup { state } } } }'
for i in "${!paths[@]}"; do
  b="${branches[$i]}"
  n="$(issue_in "$b")"
  [ -n "$n" ] || n="$(issue_in "$(basename "${paths[$i]}")")"
  nums+=("$n")
  if [ -n "$b" ]; then
    decl="$decl, \$b$i: String!"
    fields="$fields w$i: pullRequests(headRefName: \$b$i, first: 1,
      orderBy: {field: CREATED_AT, direction: DESC}) { nodes { $pr_fields } }"
    args+=(-f "b$i=$b")
  fi
  if [ -n "$n" ]; then
    decl="$decl, \$n$i: Int!"
    fields="$fields i$i: issue(number: \$n$i) { title }"
    args+=(-F "n$i=$n")
  fi
done

# An issue number that does not exist comes back as null beside an error, and
# gh exits non-zero with the rest of the data intact, so success is judged on
# the data, not the exit code.
resp='{"data":{"repository":{}}}'
gh_error=""
if [ -n "$fields" ]; then
  err_file="$(mktemp)"
  trap 'rm -f "$err_file"' EXIT
  resp="$(gh api graphql -f query="query(${decl#, }) {
    repository(owner: \"${repo%%/*}\", name: \"${repo#*/}\") { $fields } }" \
    "${args[@]}" 2>"$err_file" || true)"
  if ! printf '%s' "$resp" | jq -e '.data.repository' >/dev/null 2>&1; then
    gh_error="$(head -1 "$err_file")"
    [ -n "$gh_error" ] || gh_error="it answered something other than data"
    resp='{"data":{"repository":{}}}'
  fi
fi

# A missing lease script means a machine with no session tracking. One that is
# there but lists no row for the main checkout failed or changed its wording,
# so a held worktree could read as free: nothing is then called safe.
lease_out=""
lease_note=""
lease_broken=""
if [ -x "$lease" ]; then
  lease_out="$("$lease" list "$PWD" 2>/dev/null || true)"
  if ! printf '%s\n' "$lease_out" | awk -v n="$main_name" '$1 == n { f = 1 } END { exit !f }'; then
    lease_broken=1
    lease_note="    The session list did not answer, so nothing is called safe to remove."
  fi
else
  lease_note="    (no worktree-lease.sh on this machine, so sessions are not shown)"
fi

held_by() {
  printf '%s\n' "$lease_out" | awk -v n="$1" '
    $1 == n && / LEASED by / {
      for (i = 1; i < NF; i++) if ($i == "by") { print $(i + 1); exit }
    }'
}

plural() {
  if [ "$1" -eq 1 ]; then echo "$1 $2"; else echo "$1 $2s"; fi
}

has_main=""
git rev-parse -q --verify "$main_ref^{commit}" >/dev/null && has_main=1

order=()
blocks=()
groups=()
for i in "${!paths[@]}"; do
  p="${paths[$i]}"
  b="${branches[$i]}"
  name="$(basename "$p")"
  head="$(git -C "$p" rev-parse HEAD)"
  dirty="$(git -C "$p" status --porcelain | wc -l | tr -d ' ')"
  age="$(git -C "$p" log -1 --format=%cr)"
  order+=("$(git -C "$p" log -1 --format=%ct) $i")

  on_main=""
  ahead=0
  if [ -n "$has_main" ]; then
    git -C "$p" merge-base --is-ancestor HEAD "$main_ref" && on_main=1
    ahead="$(git -C "$p" rev-list --count "$main_ref..HEAD")"
  fi

  IFS="$sep" read -r number title state draft review oid rollup closing_n closing_t issue_t \
    < <(printf '%s' "$resp" | jq -r --arg w "w$i" --arg is "i$i" --arg s "$sep" '
      .data.repository as $r | ($r[$w].nodes[0] // {}) as $pr |
      [$pr.number // "", $pr.title // "", $pr.state // "", ($pr.isDraft // false | tostring),
       $pr.reviewDecision // "", $pr.headRefOid // "",
       ($pr.commits.nodes[0].commit.statusCheckRollup.state // ""),
       ($pr.closingIssuesReferences.nodes[0].number // ""),
       ($pr.closingIssuesReferences.nodes[0].title // ""),
       ($r[$is].title // "")] | map(tostring) | join($s)') || true

  if [ -n "$closing_n" ]; then
    issue="#$closing_n $closing_t"
  elif [ -n "$issue_t" ]; then
    issue="#${nums[$i]} $issue_t"
  elif [[ "$title" =~ ^(.*[^ ])\ *\(#([0-9]+)\)\ *$ ]]; then
    issue="#${BASH_REMATCH[2]} ${BASH_REMATCH[1]}"
  elif [ -n "$b" ]; then
    issue="no issue, branch $b"
  else
    issue="no issue, detached HEAD"
  fi

  case "$rollup" in
    SUCCESS) ci="CI passed" ;;
    FAILURE | ERROR) ci="CI failed" ;;
    PENDING | EXPECTED) ci="CI running" ;;
    *) ci="no CI result" ;;
  esac
  case "$review" in
    APPROVED) rev="approved" ;;
    CHANGES_REQUESTED) rev="changes requested" ;;
    REVIEW_REQUIRED) rev="review required" ;;
    *) rev="no review" ;;
  esac
  case "$state" in
    MERGED) pr_state="merged" ;;
    CLOSED) pr_state="closed" ;;
    OPEN) if [ "$draft" = true ]; then pr_state="draft"; else pr_state="ready"; fi ;;
  esac

  if [ "$dirty" -gt 0 ]; then
    local_state="$(plural "$dirty" "uncommitted file")"
  else
    local_state="clean"
  fi
  if [ -n "$gh_error" ]; then
    remote="GitHub did not answer, so PR, CI and review are unknown"
  elif [ -n "$number" ]; then
    remote="PR #$number $pr_state, $ci, $rev"
  elif [ -n "$on_main" ]; then
    remote="no PR, nothing beyond main"
  else
    remote="no PR, $(plural "$ahead" "commit") not on main"
  fi

  # Whose move: the first rule that matches wins.
  on_pr=""
  if [ -n "$oid" ]; then
    if [ "$head" = "$oid" ] || { git -C "$p" cat-file -e "$oid^{commit}" 2>/dev/null &&
      git -C "$p" merge-base --is-ancestor HEAD "$oid"; }; then
      on_pr=1
    fi
  fi
  held="$(held_by "$name")"
  if [ -n "$held" ]; then
    group=agent move="an agent: session $held is working in it"
  elif [ "$dirty" -gt 0 ]; then
    group=you move="you: $local_state and no session holds it"
  elif [ -n "$number" ] && [ -z "$on_pr" ] && [ -z "$on_main" ]; then
    stray="$ahead"
    git -C "$p" cat-file -e "$oid^{commit}" 2>/dev/null &&
      stray="$(git -C "$p" rev-list --count "$oid..HEAD")"
    group=you move="you: $(plural "$stray" "commit") not on the PR or main; push or drop"
  elif [ "$state" = OPEN ]; then
    if [ "$draft" = false ] && [ "$rollup" = SUCCESS ] && [ "$review" != CHANGES_REQUESTED ]; then
      group=you move="you: ready to merge"
    else
      group=agent
      case "$review/$ci" in
        CHANGES_REQUESTED/*) move="an agent: changes requested" ;;
        */"CI failed") move="an agent: CI failed on the last commit" ;;
        */"CI running") move="an agent: CI running" ;;
        */"no CI result") move="an agent: no CI result on the last commit yet" ;;
        *) move="an agent: draft, waiting on review before it is marked ready" ;;
      esac
    fi
  elif [ -n "$on_main" ]; then
    group=done move="finished, safe to remove"
  elif [ -z "$has_main" ] && [ -z "$number" ]; then
    group=unknown move="unknown: no $main_ref here to compare against"
  elif [ -n "$gh_error" ]; then
    group=unknown move="unknown: GitHub did not answer, so whose move is not known"
  elif [ "$state" = MERGED ]; then
    group=done move="finished, safe to remove"
  elif [ "$state" = CLOSED ]; then
    group=you move="you: PR closed without merging; reopen it, or remove it and its branch"
  else
    group=you move="you: no PR for $(plural "$ahead" "commit"); open one, or remove it and its branch"
  fi

  if [ "$group" = done ] && [ -n "$lease_broken" ]; then
    group=unknown move="unknown: the session list did not answer, so a session may be in it"
  fi
  groups+=("$group")
  blocks+=("- $name
    $issue
    $remote; last commit $age; $local_state
    $move")
done

section() {
  local want="$1" heading="$2" count=0 out="" ts i
  while read -r ts i; do
    [ "${groups[$i]}" = "$want" ] || continue
    count=$((count + 1))
    out="$out${blocks[$i]}
"
  done < <(printf '%s\n' "${order[@]}" | sort -rn)
  if [ "$want" = unknown ] && [ "$count" -eq 0 ]; then return; fi
  echo
  echo "$heading ($count)"
  echo
  if [ -n "$out" ]; then printf '%s' "$out"; else echo "    nothing"; fi
}

if [ -n "$gh_error" ]; then
  echo "Worktrees of $repo, read from git just now."
  echo "    GitHub did not answer, so PR, CI and review are unknown:"
  echo "    $gh_error"
fi
[ -n "$gh_error" ] || echo "Worktrees of $repo, read from GitHub and git just now."
[ -z "$lease_note" ] || echo "$lease_note"
[ -n "$has_main" ] || echo "    No $main_ref here, so nothing is judged to be on main."

section you "Your move"
section agent "An agent's move"
section done "Finished, safe to remove"
section unknown "Unknown"

safe=0
for g in "${groups[@]}"; do [ "$g" = done ] && safe=$((safe + 1)); done
echo
line="$safe of ${#paths[@]} worktrees are safe to remove"
[ -z "$gh_error" ] || line="$line, on git alone"
echo "$line. This removed nothing: just worktree-rm <name> does."
