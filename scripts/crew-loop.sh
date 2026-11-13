#!/usr/bin/env bash
# The crew loop (#2821, specs/crew-loop.md): takes the top qualifying ready
# issue, makes its worktree, starts one `claude --bg` crew session on it,
# watches that session against the caps, and ends with a draft pull request or
# a parked issue. One issue at a time; nothing here marks a PR ready or merges.
#
#   crew-loop.sh                     run until the queue is empty, a cap is hit
#                                    or the stop file appears (`just crew`)
#   crew-loop.sh stop-session <id>   stop one background session and kill its
#                                    whole process tree; CREW_PARK_STEP for
#                                    harness check 7 in crew-qualify.sh
#
# Every outside command has a seam (CREW_* below), so
# scripts/tests/crew-loop-test.sh runs the loop with stand-ins and no session,
# network or simulator.

set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO=${CREW_REPO:-jonyardley/intrada}

# ── The four figures, set by the owner on 2026-11-12 ──────────────────────────
WALL_CLOCK=${CREW_WALL_CLOCK:-10800} # seconds per session: 3 hours
SILENCE=${CREW_SILENCE:-2700}        # 45 minutes; the simulator queue waits up to 1800
MAX_ISSUES=${CREW_MAX_ISSUES:-3}     # issues a night
SPEND_CAP=${CREW_SPEND_CAP:-150}     # dollars, the last 24 hours of `just usage`
# ──────────────────────────────────────────────────────────────────────────────

POLL=${CREW_POLL:-60}
STOP_FILE=${CREW_STOP_FILE:-$HOME/.intrada-crew-stop}
SIM_LOCK=${IOS_SIM_LOCK_DIR:-/tmp/intrada-ios-test.lock}
LOG=${CREW_LOG:-$HOME/.claude/crew-loop.log}

log() {
  local line
  line="$(date '+%Y-%m-%d %H:%M:%S') $*"
  printf '%s\n' "$line"
  printf '%s\n' "$line" >>"$LOG" 2>/dev/null || :
}

now() { date +%s; }

main_checkout() {
  dirname "$(git -C "$root" rev-parse --path-format=absolute --git-common-dir)"
}

# Where `just worktree-new` puts worktrees; each crew one is crew-<issue>.
worktrees() {
  local main
  if [ -n "${CREW_WORKTREES:-}" ]; then
    printf '%s' "$CREW_WORKTREES"
    return
  fi
  main=$(main_checkout)
  printf '%s' "$(dirname "$main")/$(basename "$main")-worktrees"
}

# ── Sessions ─────────────────────────────────────────────────────────────────

agent_record() {
  claude agents --json --all 2>/dev/null | jq -c --arg id "$1" '.[] | select(.id == $id)'
}

descendants() {
  local p kid
  for p in "$@"; do
    for kid in $(pgrep -P "$p" 2>/dev/null); do
      printf '%s\n' "$kid"
      descendants "$kid"
    done
  done
}

# The session half of parking: stop the session and kill everything under it.
# The tree is read before the stop, since a child whose parent has died can no
# longer be found by its parent. The record is kept (no `claude rm`), so its
# state after the stop can be read and the conversation resumed.
stop_session() {
  local id=$1 pid tree p left=0 lock_pid
  pid=$(agent_record "$id" | jq -r '.pid // empty')
  tree=""
  [ -n "$pid" ] && tree=$(printf '%s\n%s\n' "$pid" "$(descendants "$pid")" | sed '/^$/d')
  claude stop "$id" >/dev/null 2>&1
  for p in $tree; do kill -TERM "$p" 2>/dev/null; done
  sleep "${CREW_KILL_GRACE:-3}"
  for p in $tree; do
    if kill -0 "$p" 2>/dev/null; then
      kill -KILL "$p" 2>/dev/null
      left=$((left + 1))
    fi
  done
  # The simulator lock is released only when a process of this session held it.
  lock_pid=$(cat "$SIM_LOCK/pid" 2>/dev/null)
  if [ -n "$lock_pid" ] && printf '%s\n' "$tree" | grep -qx "$lock_pid"; then
    rm -rf "$SIM_LOCK"
    echo "released the simulator lock held by $lock_pid"
  fi
  echo "stopped $id and $(printf '%s\n' "$tree" | sed '/^$/d' | wc -l | tr -d ' ') processes ($left needed kill -9)"
}

start_session() {
  local wt=$1 n=$2 out id
  out=$(cd "$wt" && INTRADA_CREW=1 claude --bg --model opus --effort high \
    --permission-mode auto -n "crew-$n" "/crew $n" 2>&1)
  id=$(printf '%s' "$out" | grep -oE 'backgrounded[^0-9a-f]+[0-9a-f]{8}' | grep -oE '[0-9a-f]{8}$' | head -1)
  if [ -z "$id" ]; then
    log "could not start a session for #$n: $(printf '%s' "$out" | tail -1)"
    return 1
  fi
  printf '%s' "$id"
}

# Newest change to the session's transcript or any of its subagents'. A lead
# waiting on a long task agent writes nothing to its own transcript.
last_activity() {
  local cwd=$1 sid=$2 dir newest=0 f m
  dir="$HOME/.claude/projects/$(printf '%s' "$cwd" | sed 's/[^A-Za-z0-9]/-/g')"
  for f in "$dir/$sid.jsonl" "$dir/$sid"/subagents/*.jsonl; do
    [ -f "$f" ] || continue
    m=$(stat -f %m "$f" 2>/dev/null || stat -c %Y "$f" 2>/dev/null || echo 0)
    [ "$m" -gt "$newest" ] && newest=$m
  done
  printf '%s' "$newest"
}

# The last thing the session said, for a park comment.
last_words() {
  local cwd=$1 sid=$2 f
  f="$HOME/.claude/projects/$(printf '%s' "$cwd" | sed 's/[^A-Za-z0-9]/-/g')/$sid.jsonl"
  [ -f "$f" ] || return 0
  jq -r 'select(.type == "assistant") | .message.content[]? | select(.type == "text") | .text' "$f" 2>/dev/null |
    tail -c 600
}

# ── GitHub ───────────────────────────────────────────────────────────────────

# The newest note the session left (brief, built, reviewed, QA, draft or
# parked), counting only notes from $2 on when given, so last night's parked
# note never reads as tonight's outcome.
last_note() {
  gh issue view "$1" -R "$REPO" --json comments --jq \
    '[.comments[] | select(.createdAt >= "'"${2:-}"'") | .body | select(startswith("**Crew: "))
      | capture("^\\*\\*Crew: (?<s>[A-Za-z]+)\\*\\*").s] | last // ""' 2>/dev/null
}

has_label() {
  gh issue view "$1" -R "$REPO" --json labels --jq '.labels[].name' 2>/dev/null | grep -qx "$2"
}

# The issue half of parking: label, one question, the owner's phone. The claim,
# branch and worktree stay, so the owner's answer resumes from there.
park_issue() {
  local n=$1 why=$2 question=$3 said=${4:-}
  gh issue edit "$n" -R "$REPO" --add-label needs-owner >/dev/null 2>&1 ||
    log "could not add needs-owner to #$n"
  local body="**Crew: parked**: the crew loop stopped this session: $why.

Done so far: the newest Crew note above, and the branch \`crew-$n\`.

$question"
  [ -n "$said" ] && body="$body

The session's last words:

$said"
  gh issue comment "$n" -R "$REPO" --body "$body" >/dev/null 2>&1 ||
    log "could not post the park note on #$n"
  notify "Crew parked #$n: $why"
}

# ── The owner's phone ────────────────────────────────────────────────────────

# A Mac notification always; then the phone, through a short session that
# sends Claude's own push, since nothing else here reaches the phone without an
# outside service. The push tool holds a push back while the owner is active
# at a terminal, so it lands when they are away, which is when the loop runs.
notify() {
  local msg=$1
  if [ -n "${CREW_NOTIFY:-}" ]; then
    $CREW_NOTIFY "$msg"
    return
  fi
  osascript -e "display notification \"${msg//\"/\'}\" with title \"Intrada crew\"" >/dev/null 2>&1 || :
  (cd "${TMPDIR:-/tmp}" && claude -p --model haiku --allowedTools PushNotification \
    "Send this exact text with the PushNotification tool, then stop: $msg" >/dev/null 2>&1) || :
  log "told the owner: $msg"
}

# ── Caps and the queue ───────────────────────────────────────────────────────

# Dollars in the last 24 hours across every project on this Mac. Fails when the
# report cannot be read, so the loop stops rather than spends blind.
spend() {
  local out
  if [ -n "${CREW_SPEND:-}" ]; then
    out=$($CREW_SPEND) || return 1
  else
    out=$(python3 "$HOME/.claude/hooks/usage-report.py" --days 1 --brief 2>/dev/null) || return 1
  fi
  # The report says "nothing recorded" for a day with no sessions at all.
  if printf '%s' "$out" | grep -q 'nothing recorded'; then
    echo 0
    return
  fi
  printf '%s' "$out" | grep -oE '\$[0-9,]+' | head -1 | tr -d '$,' | grep -E '^[0-9]+$'
}

# The ready queue in `just status` order, less any issue parked for the owner;
# exit 2 when GitHub cannot be read.
queue() {
  local out
  out=$(${CREW_QUEUE:-"$root/scripts/generate-status.sh" --ready-json}) || return 2
  printf '%s' "$out" | jq -r '.[] | select([.labels.nodes[].name] | index("needs-owner") | not) | .number' ||
    return 2
}

# Crew worktrees left by an earlier night, still open with no draft and not
# waiting on the owner: a restart takes these before the queue, which leaves
# out a claimed issue. A parked issue whose needs-owner label the owner has
# removed is among them: the answer is on the issue.
unfinished() {
  local n
  find "$(worktrees)" -maxdepth 1 -type d -name 'crew-[0-9]*' 2>/dev/null |
    sed -n 's#.*/crew-\([0-9][0-9]*\)$#\1#p' | sort -n |
    while IFS= read -r n; do
      [ "$(gh issue view "$n" -R "$REPO" --json state --jq .state 2>/dev/null)" = OPEN ] || continue
      [ "$(last_note "$n")" = draft ] && continue
      has_label "$n" needs-owner && continue
      echo "$n"
    done
}

# A crew worktree, made from the main checkout like any other.
worktree_for() {
  local n=$1 wt
  wt="$(worktrees)/crew-$n"
  if [ ! -d "$wt" ]; then
    if [ -n "${CREW_WORKTREE:-}" ]; then
      $CREW_WORKTREE "$n" "$wt" >/dev/null 2>&1 || return 1
    else
      (cd "$(main_checkout)" && just worktree-new "crew-$n") >/dev/null 2>&1 || return 1
    fi
  fi
  [ -d "$wt" ] || return 1
  printf '%s' "$wt"
}

# Harness check 1: a live crew session is refused `gh pr ready`. The loop
# starts no issue unless it passes tonight.
check_guards() {
  if [ -n "${CREW_CHECK1:-}" ]; then
    $CREW_CHECK1
    return
  fi
  local dir
  dir=$(mktemp -d "${TMPDIR:-/tmp}/crew-check1.XXXXXX")
  "$root/scripts/crew-qualify.sh" run "$dir" 1 >/dev/null 2>&1
  grep -q '^PASS 1:' "$dir/results.txt" 2>/dev/null
}

# ── One issue ────────────────────────────────────────────────────────────────

# Watches one session to its end; prints the outcome: draft, parked, or the
# reason the loop parked it.
watch() {
  local id=$1 wt=$2 n=$3 since=$4 started rec sid pid state status waiting idle=0 last
  started=$(now)
  while :; do
    sleep "$POLL"
    rec=$(agent_record "$id")
    sid=$(printf '%s' "$rec" | jq -r '.sessionId // empty')
    pid=$(printf '%s' "$rec" | jq -r '.pid // empty')
    state=$(printf '%s' "$rec" | jq -r '.state // empty')
    status=$(printf '%s' "$rec" | jq -r '.status // empty')
    waiting=$(printf '%s' "$rec" | jq -r '.waitingFor // empty')

    if [ "$waiting" = "permission prompt" ]; then
      echo "permission"
      return
    fi
    if [ -z "$rec" ] || [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
      case "$(last_note "$n" "$since")" in draft) echo draft ;; parked) echo parked ;; *) echo "no-process" ;; esac
      return
    fi
    # Two idle polls in a row: the turn has ended, so the notes decide.
    if [ "$status" = idle ] || [ "$state" = done ] || [ "$state" = blocked ]; then
      idle=$((idle + 1))
    else
      idle=0
    fi
    if [ "$idle" -ge 2 ]; then
      case "$(last_note "$n" "$since")" in draft) echo draft ;; parked) echo parked ;; *) echo "ended" ;; esac
      return
    fi
    if [ $(($(now) - started)) -ge "$WALL_CLOCK" ]; then
      echo "wall-clock"
      return
    fi
    last=$(last_activity "$wt" "$sid")
    [ "$last" -gt 0 ] || last=$started
    if [ $(($(now) - last)) -ge "$SILENCE" ]; then
      echo "silence"
      return
    fi
  done
}

run_issue() {
  local n=$1 wt id outcome sid said since
  since=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  wt=$(worktree_for "$n") || {
    park_issue "$n" "it could not make the worktree crew-$n" "Is a branch or folder of that name in the way?"
    return
  }
  id=$(start_session "$wt" "$n") || {
    park_issue "$n" "it could not start a session" "Does \`claude --bg\` start here by hand?"
    return
  }
  log "#$n: session $id in $wt"
  outcome=$(watch "$id" "$wt" "$n" "$since")
  sid=$(agent_record "$id" | jq -r '.sessionId // empty')
  said=$(last_words "$wt" "$sid")
  log "#$n: $(stop_session "$id")"
  case "$outcome" in
    draft) notify "Crew: draft ready for your review on #$n" ;;
    parked) notify "Crew parked #$n with a question" ;;
    permission) park_issue "$n" "it stopped at a permission prompt" \
      "Should the crew be allowed what it asked for, or should the plan avoid it? Answer here and remove the needs-owner label." "$said" ;;
    no-process) park_issue "$n" "the session showed as working with no process behind it" \
      "Remove the needs-owner label to run it again tonight's way, or say what to change." "$said" ;;
    wall-clock) park_issue "$n" "it hit the $((WALL_CLOCK / 60)) minute wall clock" \
      "Should this issue be split, or run again as it is?" "$said" ;;
    silence) park_issue "$n" "nothing was written for $((SILENCE / 60)) minutes" \
      "Run it again as it is, or is something on this issue likely to hang?" "$said" ;;
    *) park_issue "$n" "the session ended its turn without a draft or a park note" \
      "What should the crew do next on this issue?" "$said" ;;
  esac
}

# ── The night ────────────────────────────────────────────────────────────────

main() {
  local count=0 tried=" " n cand candidates dollars rc
  if [ -n "${CI:-}" ]; then
    echo "crew-loop.sh starts real sessions on this Mac, never in CI" >&2
    exit 1
  fi
  if [ -e "$STOP_FILE" ]; then
    log "the stop file $STOP_FILE is there; remove it to run the crew"
    exit 0
  fi
  if ! check_guards; then
    log "harness check 1 failed: a crew session was not refused gh pr ready, so no issue starts tonight"
    notify "Crew did not start: the guards check failed"
    exit 1
  fi
  log "harness check 1 passed; caps: $MAX_ISSUES issues, \$$SPEND_CAP, $((WALL_CLOCK / 60)) min a session, $((SILENCE / 60)) min silence"

  while :; do
    if [ -e "$STOP_FILE" ]; then
      log "stop file found; ending the night"
      break
    fi
    if [ "$count" -ge "$MAX_ISSUES" ]; then
      log "ran $count issues, the nightly cap"
      break
    fi
    if ! dollars=$(spend) || ! [[ $dollars =~ ^[0-9]+$ ]]; then
      log "could not read the spend report, so the loop stops rather than spend blind"
      break
    fi
    if [ "$dollars" -ge "$SPEND_CAP" ]; then
      log "spend in the last 24 hours is \$$dollars, at or past the \$$SPEND_CAP cap"
      break
    fi

    n=""
    candidates=$(unfinished)
    candidates="$candidates
$(queue)"
    rc=$?
    for cand in $candidates; do
      case "$tried" in *" $cand "*) continue ;; esac
      tried="$tried$cand "
      if "${CREW_PICK:-$root/scripts/crew-pick.sh}" "$cand" >/dev/null 2>&1; then
        n=$cand
        break
      fi
    done
    if [ -z "$n" ]; then
      [ "$rc" -eq 0 ] && log "no qualifying ready issue left" || log "could not read the ready queue"
      break
    fi

    run_issue "$n"
    count=$((count + 1))
  done
  log "night over: $count issues run"
  [ "$count" -gt 0 ] && notify "Crew finished: $count issues run, details on each issue"
  exit 0
}

case "${1:-}" in
  "") main ;;
  stop-session)
    [ $# -eq 2 ] || { echo "usage: $0 stop-session <id>" >&2; exit 2; }
    stop_session "$2"
    ;;
  *)
    echo "usage: $0 [stop-session <id>]" >&2
    exit 2
    ;;
esac
