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
#   crew-loop.sh issue <issue>       the issue as the briefs read it: its body
#                                    only when trusted, and trusted comments
#   crew-loop.sh lessons             trusted Lessons comments not yet folded in
#
# Every outside command has a seam (CREW_* below), so
# scripts/tests/crew-loop-test.sh runs the loop with stand-ins and no session,
# network or simulator.

set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO=${CREW_REPO:-jonyardley/intrada}
# shellcheck source=crew-trusted.sh
. "$root/scripts/crew-trusted.sh"
TRUSTED=$(trusted_jq)

# ── The four figures, set by the owner on 2026-11-12 ──────────────────────────
WALL_CLOCK=${CREW_WALL_CLOCK:-10800} # seconds per session: 3 hours
SILENCE=${CREW_SILENCE:-2700}        # 45 minutes; the simulator queue waits up to 1800
MAX_ISSUES=${CREW_MAX_ISSUES:-3}     # issues a run
SPEND_CAP=${CREW_SPEND_CAP:-150}     # dollars, the loop's own sessions in the last 24 hours
# ──────────────────────────────────────────────────────────────────────────────

# ── Ostinato (#2854): the run record and the agent's identity ───────────────
OSTI=${CREW_OSTI:-osti}
export OSTINATO_PROJECT=$REPO
export OSTINATO_AGENT_KEYCHAIN_SERVICE=${OSTINATO_AGENT_KEYCHAIN_SERVICE:-ostinato-intrada-loop}
export OSTINATO_AGENT_KEYCHAIN_ACCOUNT=${OSTINATO_AGENT_KEYCHAIN_ACCOUNT:-ostinato-agent}
export OSTINATO_AGENT_EMAIL=${OSTINATO_AGENT_EMAIL:-356601309+ostinato-agent@users.noreply.github.com}
RECORD="${OSTINATO_HOME:-$HOME/.local/state/ostinato/records}/$(printf '%s' "$REPO" | sed 's#/#__#' | tr '[:upper:]' '[:lower:]').jsonl"
RUN=""         # the run's name, once run_started is written
ATTEMPT=0      # the open attempt's number; 0 when none is open
ATTEMPTS=0     # attempts this run
SESSION_ID=""  # the open attempt's Claude Code session, once known
SPEND_SEEN=0   # the spend figure the cap last read, on each picked line
RECORD_FAILED=0
# ──────────────────────────────────────────────────────────────────────────────

POLL=${CREW_POLL:-60}
STOP_FILE=${CREW_STOP_FILE:-$HOME/.intrada-crew-stop}
SIM_LOCK=${IOS_SIM_LOCK_DIR:-/tmp/intrada-ios-test.lock}
LOCK=${CREW_LOCK:-/tmp/intrada-crew-loop.lock}
LOCK_GRACE=${CREW_LOCK_GRACE:-10} # seconds a lock may lack its pid while a run writes it
CAFFEINATE=${CREW_CAFFEINATE:-caffeinate}
START_TRIES=${CREW_START_TRIES:-5}
LOG=${CREW_LOG:-$HOME/.claude/crew-loop.log}
CURRENT_N=""  # the issue being started or watched; INT, TERM or HUP stops its session
STARTED_ID=""
OUTCOME=""

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

# The session half of parking (#2821): the tree is read before the stop, since
# an orphan cannot be found from its parent. The record is kept (no `claude rm`)
# so its state can be read and the conversation resumed.
stop_session() {
  local id=$1 pid tree p left=0 lock_pid
  pid=$(agent_record "$id" | jq -r '.pid // empty')
  # A record's pid may have been reused by another process since.
  if [ -n "$pid" ] && ! grep -q claude <<<"$(ps -o command= -p "$pid" 2>/dev/null)"; then
    echo "pid $pid of $id is not a claude process, so its tree is left alone"
    pid=""
  fi
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
  if [ -n "$lock_pid" ] && grep -qx "$lock_pid" <<<"$tree"; then
    rm -rf "$SIM_LOCK"
    echo "released the simulator lock held by $lock_pid"
  fi
  echo "stopped $id and $(printf '%s\n' "$tree" | sed '/^$/d' | wc -l | tr -d ' ') processes ($left needed kill -9)"
}

# A session left running in this worktree, by a loop that was killed, would
# build the same issue twice.
stop_live_in() {
  local id pid
  claude agents --json --all 2>/dev/null |
    jq -r --arg c "$1" '.[] | select(.kind == "background" and .id != null and .cwd == $c and .pid != null)
      | "\(.id) \(.pid)"' |
    while read -r id pid; do
      kill -0 "$pid" 2>/dev/null && log "a live session was already in $1: $(stop_session "$id")"
    done
}

# Every session named crew-$1 that is not stopped yet.
stop_named() {
  local id
  for id in $(claude agents --json --all 2>/dev/null |
    jq -r --arg m "crew-$1" '.[] | select(.name == $m and .state != "stopped") | .id'); do
    log "#$1: $(stop_session "$id")"
  done
}

# Sets STARTED_ID from the session list: the session named crew-N started
# since $since, in Unix milliseconds as the list counts them, since
# `claude --bg` prints its id only as text (#2821). One that shows up too late
# may still run, so any session by that name is stopped.
start_session() {
  local wt=$1 n=$2 out since try=0 found
  since=$(($(now) * 1000))
  # The run and attempt ride in the first prompt too, so every `osti write` the
  # session makes names them (#2854).
  out=$(cd "$wt" && INTRADA_CREW=1 OSTINATO_RUN=$RUN OSTINATO_PROJECT=$REPO claude --bg --model opus \
    --effort high --permission-mode auto -n "crew-$n" "/crew $n $RUN $ATTEMPT" 2>&1)
  STARTED_ID=""
  while [ -z "$STARTED_ID" ] && [ "$try" -lt "$START_TRIES" ]; do
    [ "$try" -gt 0 ] && sleep "${CREW_START_WAIT:-2}"
    try=$((try + 1))
    found=$(claude agents --json --all 2>/dev/null | jq -r --arg m "crew-$n" --argjson t "$since" \
      '[.[] | select(.name == $m and (.startedAt // 0) >= $t)] | first | select(.id != null)
        | "\(.id) \(.sessionId // "")"')
    STARTED_ID=${found%% *}
    # Known from the start, so a stop before the first poll still prices it.
    [ -n "${found#* }" ] && SESSION_ID=${found#* }
  done
  if [ -z "$STARTED_ID" ]; then
    log "could not start a session for #$n: $(printf '%s' "$out" | tail -1)"
    stop_named "$n"
    return 1
  fi
}

# Newest change to the session's transcript, its subagents' or its background
# tasks' output: a lead waiting on a task agent writes nothing itself (#2821).
last_activity() {
  local cwd=$1 sid=$2 key dir tasks newest=0 f m
  key=$(printf '%s' "$cwd" | sed 's/[^A-Za-z0-9]/-/g')
  dir="$HOME/.claude/projects/$key"
  tasks="${CREW_TASKS_ROOT:-/private/tmp/claude-$(id -u)}/$key/$sid/tasks"
  for f in "$dir/$sid.jsonl" "$dir/$sid"/subagents/*.jsonl "$tasks"/*.output; do
    [ -f "$f" ] || continue
    # GNU first: GNU `stat -f` succeeds with the wrong output.
    m=$(stat -c %Y "$f" 2>/dev/null || stat -f %m "$f" 2>/dev/null || echo 0)
    [ "$m" -gt "$newest" ] && newest=$m
  done
  printf '%s' "$newest"
}

# The last thing the session said, for a park comment, with no backtick that
# could close the fence it goes in.
transcript() {
  printf '%s/.claude/projects/%s/%s.jsonl' "$HOME" "$(printf '%s' "$1" | sed 's/[^A-Za-z0-9]/-/g')" "$2"
}

last_words() {
  local f
  f=$(transcript "$1" "$2")
  [ -f "$f" ] || return 0
  jq -r 'select(.type == "assistant") | .message.content[]? | select(.type == "text") | .text' "$f" 2>/dev/null |
    tail -c 300 | tr '`' "'"
}

# The session's last assistant record, when it is Claude Code's own error for
# the plan's usage limit (#2854). Only an API error record counts, never what
# the session wrote, and only the last, so a limit it recovered from does not.
# The wording is unverified, so the caller logs the record whole.
usage_limit() {
  local f
  f=$(transcript "$1" "$2")
  [ -n "$2" ] && [ -f "$f" ] || return 1
  jq -cR 'fromjson? // empty | select(.type == "assistant")' "$f" 2>/dev/null | tail -1 |
    jq -ce 'select(.isApiErrorMessage == true and ((.error // "") == "rate_limit"
      or ([.message.content[]? | select(.type == "text") | .text] | join(" ")
        | test("usage limit|hit your limit|limit reached|limit will reset"; "i"))))' 2>/dev/null
}

# ── GitHub ───────────────────────────────────────────────────────────────────

# The newest Crew note from $2 on, so an earlier run's park is not this
# run's outcome.
last_note() {
  gh issue view "$1" -R "$REPO" --json comments --jq "$TRUSTED"'
    [.comments[] | select(trusted and .createdAt >= "'"${2:-}"'") | .body | select(startswith("**Crew: "))
      | capture("^\\*\\*Crew: (?<s>[A-Za-z]+)\\*\\*").s] | last // ""' 2>/dev/null
}

# The issue half of parking. Fails when the label could not be added, since an
# unlabelled issue would be taken again (#2821).
#
# $2 is the reason code the run record counts (#2854); the note keeps the Crew
# heading, since the restart and the outcome read only that.
park_issue() {
  local n=$1 code=$2 why=$3 question=$4 said=${5:-} labelled=0 file
  if ! gh issue edit "$n" -R "$REPO" --add-label needs-owner >/dev/null 2>&1; then
    log "could not add needs-owner to #$n"
    labelled=1
  fi
  local body="**Crew: parked**: the crew loop stopped this session: $why.

Done so far: the newest Crew note above, and the branch \`crew-$n\`.

$question"
  [ -n "$said" ] && body="$body

The session's last words:

\`\`\`text
$said
\`\`\`"
  file=$(mktemp "${TMPDIR:-/tmp}/crew-park.XXXXXX")
  printf '%s\n' "$body" >"$file"
  record_noted "$n" "$body" parked --item "$(item "$n")" --url "$(issue_url "$n")" --attempt "$ATTEMPT" \
    --reason "$code" --question "$question" --body-file "$file" ${SESSION_ID:+--session "$SESSION_ID"}
  rm -f "$file"
  notify "Crew parked #$n: $why"
  return "$labelled"
}

# ── The run record (#2854, the Ostinato record format) ──────────────────────

item() { printf '%s#%s' "${REPO#*/}" "$1"; }
issue_url() { printf 'https://github.com/%s/issues/%s' "$REPO" "$1"; }

# One line in the run record. Exit 1 means nothing was written and 3 that the
# line was written but its note was not; either way it is never run again,
# since a repeat would hold the line twice.
record() {
  local rc=0 err
  err=$("$OSTI" write --project "$REPO" --run "$RUN" "$@" 2>&1 >/dev/null) || rc=$?
  case $rc in
    0) ;;
    3) log "the record has $1, but osti could not post its note: $(printf '%s' "$err" | tail -1)" ;;
    *) log "could not write $1 to the run record (osti exit $rc): $(printf '%s' "$err" | tail -1)" ;;
  esac
  return "$rc"
}

# A line with a note: when osti did not post the note, gh posts it, since the
# notes are what a restart reads.
record_noted() {
  local n=$1 body=$2 rc=0
  shift 2
  record "$@" || rc=$?
  if [ "$rc" -ne 0 ]; then
    gh issue comment "$n" -R "$REPO" --body "$body" >/dev/null 2>&1 ||
      log "could not post the $1 note on #$n"
  fi
  return 0
}

# The attempt's end, with no note: one after a park note would read as the
# owner's answer and run the issue again (#2854).
end_attempt() {
  local n=$1 outcome=$2 wt=${3:-}
  local attempt=$ATTEMPT sid=$SESSION_ID
  [ "$attempt" -gt 0 ] || return 0
  # Cleared before the write, so a signal while osti prices it never writes
  # the end twice.
  ATTEMPT=0
  SESSION_ID=""
  record attempt-ended --no-note --item "$(item "$n")" --url "$(issue_url "$n")" --attempt "$attempt" \
    --outcome "$outcome" ${sid:+--session "$sid"} ${wt:+--cwd "$wt"} || RECORD_FAILED=1
}

# The next attempt at issue $1: one past its newest picked line in the record.
next_attempt() {
  local last
  [ -f "$RECORD" ] || { echo 1; return; }
  last=$(jq -R --arg i "$(item "$1")" 'fromjson? // empty
    | select(.event == "picked" and ((.item // "") | ascii_downcase) == ($i | ascii_downcase)) | .attempt' \
    "$RECORD" 2>/dev/null | sort -n | tail -1)
  echo $((${last:-0} + 1))
}

# Ends the run's record once; $1 is an extra field such as stopped=usage_limit.
end_run() {
  local run=$RUN
  [ -n "$run" ] || return 0
  RUN=""
  RUN=$run record run-ended --attempts "$ATTEMPTS" ${1:+--field "$1"}
}

# ── The owner's phone ────────────────────────────────────────────────────────

# A Mac notification, and Claude's own push to the phone only when
# CREW_PHONE_PUSH=1: the owner turned the push off by default (#2854).
notify() {
  local msg=$1 job
  if [ -n "${CREW_NOTIFY:-}" ]; then
    $CREW_NOTIFY "$msg" || :
    return 0
  fi
  osascript -e 'on run argv' -e 'display notification (item 1 of argv) with title "Intrada crew"' \
    -e 'end run' "$msg" >/dev/null 2>&1 || :
  if [ "${CREW_PHONE_PUSH:-}" = 1 ]; then
    (cd "${TMPDIR:-/tmp}" && exec claude -p --model haiku --allowedTools PushNotification \
      "Send this exact text with the PushNotification tool, then stop: $msg") >/dev/null 2>&1 &
    job=$!
    bound "$job" "${CREW_PUSH_LIMIT:-120}"
  fi
  log "told the owner: $msg"
  return 0
}

# Waits for background job $1, killing it after $2 seconds; bash 3.2 has no
# `wait` with a timeout.
bound() {
  local i=0
  while kill -0 "$1" 2>/dev/null; do
    if [ "$i" -ge "$2" ]; then
      kill "$1" 2>/dev/null
      sleep 1
      kill -KILL "$1" 2>/dev/null
      break
    fi
    sleep 1
    i=$((i + 1))
  done
  wait "$1" 2>/dev/null
}

# ── Caps and the queue ───────────────────────────────────────────────────────

# Dollars the loop's own sessions cost in the last 24 hours, from the run
# record (#2854): each session once, at its latest running total on an
# attempt_ended line, since a session's figure is everything it has cost so
# far. A line osti could not price per session counts its cost_usd. No record
# yet is $0; one that cannot be read fails, so the loop stops rather than
# spends blind. A torn line, from a writer killed mid line, is skipped.
spend() {
  [ -f "$RECORD" ] || { echo 0; return; }
  [ -r "$RECORD" ] || return 1
  jq -nR '[inputs | fromjson? // empty
      | select(.event == "attempt_ended")
      | select((.at // "" | sub("\\.[0-9]+Z$"; "Z") | try fromdateiso8601 catch 0) >= (now - 86400))]
    | (reduce (.[] | .session_costs // {} | to_entries[]) as $e ({}; .[$e.key] = $e.value)
        | [.[]] | add // 0) as $sessions
    | ([.[] | select(.session_costs == null) | .cost_usd // 0] | add // 0) as $unsplit
    | ($sessions + $unsplit) * 100 | round / 100' "$RECORD" 2>/dev/null
}

# The ready queue in `just status` order, less any issue parked for the owner;
# exit 2 when GitHub cannot be read.
queue() {
  local out
  out=$(${CREW_QUEUE:-"$root/scripts/generate-status.sh" --ready-json}) || return 2
  printf '%s' "$out" | jq -r '.[] | select([.labels.nodes[].name] | index("needs-owner") | not) | .number' ||
    return 2
}

# Issues $@ in one GraphQL read, each under the alias i<number>, so the reads
# do not grow by a call per crew worktree (#2821). GitHub answers a missing
# issue with null and an error, so the caller reads each alias on its own.
read_issues() {
  local q n
  q='fragment F on Issue {
    number state title body author { login } editor { login }
    labels(first: 50) { nodes { name } }
    comments(last: 100) { nodes { author { login } body createdAt } }
    timelineItems(itemTypes: [UNLABELED_EVENT], last: 50) {
      nodes { ... on UnlabeledEvent { createdAt actor { login } label { name } } } }
  }
  query($owner: String!, $name: String!) { repository(owner: $owner, name: $name) {'
  for n in "$@"; do q="$q i$n: issue(number: $n) { ...F }"; done
  gh api graphql -f query="$q } }" -f owner="${REPO%%/*}" -f name="${REPO#*/}" 2>/dev/null
}

# Crew worktrees left by an earlier run, open, with no draft and not waiting on
# the owner: no needs-owner label, and no park note unless a trusted comment
# follows it or a trusted login removed needs-owner after it, which the park
# question offers as "run it again". A park whose label never landed has
# neither, so it stays (#2821). An unreadable issue is skipped.
unfinished() {
  local nums n raw ok
  nums=$(find "$(worktrees)" -maxdepth 1 -type d -name 'crew-[0-9]*' 2>/dev/null |
    sed -n 's#.*/crew-\([0-9][0-9]*\)$#\1#p' | sort -n)
  [ -n "$nums" ] || return 0
  # shellcheck disable=SC2086 # one argument per issue number
  raw=$(read_issues $nums)
  for n in $nums; do
    ok=$(printf '%s' "$raw" | jq -r --arg k "i$n" "$TRUSTED"'
      .data.repository[$k] // null
      | if . == null then "unread"
        elif .state != "OPEN" or ([.labels.nodes[].name] | index("needs-owner")) then "no"
        else
          [.comments.nodes[] | select(trusted)] as $c
          | ([$c[].body | startswith("**Crew: ")] | indices(true) | last) as $i
          | (if $i == null then "" else ($c[$i].body
              | capture("^\\*\\*Crew: (?<s>[A-Za-z]+)\\*\\*").s) // "" end) as $s
          | if $s == "draft" then "no"
            elif $s != "parked" or $i < ($c | length) - 1 then "yes"
            elif any(.timelineItems.nodes[]; .label.name == "needs-owner"
              and (.actor.login // "" | trusted_login) and .createdAt > $c[$i].createdAt) then "yes"
            else "no" end
        end' 2>/dev/null)
    case $ok in
      yes) echo "$n" ;;
      no) ;;
      *) log "could not read #$n from GitHub, so it is skipped" >&2 ;;
    esac
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

# Harness check 1: the agent's Keychain item is there, a live crew session acts
# as the agent account, and osti's own guard refuses it `gh pr ready` (#2854).
# The loop starts no issue unless it passes at the start of the run.
check_guards() {
  local dir passed=1
  dir=$(mktemp -d "${TMPDIR:-/tmp}/crew-check1.XXXXXX") || return 1
  ${CREW_QUALIFY:-"$root/scripts/crew-qualify.sh"} run "$dir" 1 >/dev/null 2>&1
  if grep -q '^PASS 1:' "$dir/results.txt" 2>/dev/null; then
    passed=0
  else
    log "$(grep -m1 '^FAIL 1:' "$dir/results.txt" 2>/dev/null || echo "check 1 left no result")"
  fi
  rm -rf "$dir"
  return "$passed"
}

# Each worktree gets osti's guard, with the agent's identity from the
# environment, before its session starts (#2854).
guard_worktree() {
  local err
  err=$("$OSTI" guard install --worktree "$1" 2>&1) && return 0
  log "osti guard install failed in $1: $(printf '%s' "$err" | tail -1)"
  return 1
}

# ── One issue ────────────────────────────────────────────────────────────────

# The newest note decides a turn that ended; $3 when there is none, and
# unread when GitHub fails twice, so a blip never parks a draft (#2821).
note_outcome() {
  local note
  note=$(last_note "$1" "$2") || note=$(last_note "$1" "$2") || { echo unread; return; }
  case "$note" in draft) echo draft ;; parked) echo parked ;; *) echo "$3" ;; esac
}

# Watches one session to its end and sets OUTCOME: draft, parked, or the reason
# the loop parks it; waiting:<what> for a session waiting on someone. Runs in
# the main shell, so a signal stops the session.
watch() {
  local id=$1 wt=$2 n=$3 since=$4 started rec sid="" s pid state status waiting ended=0 gone=0 last
  started=$(now)
  while :; do
    sleep "$POLL" &
    wait $!
    rec=$(agent_record "$id")
    s=$(printf '%s' "$rec" | jq -r '.sessionId // empty')
    [ -n "$s" ] && sid=$s && SESSION_ID=$s
    pid=$(printf '%s' "$rec" | jq -r '.pid // empty')
    state=$(printf '%s' "$rec" | jq -r '.state // empty')
    status=$(printf '%s' "$rec" | jq -r '.status // empty')
    waiting=$(printf '%s' "$rec" | jq -r '.waitingFor // empty')

    # Nobody answers a crew session, so any wait parks (#2821).
    if [ -n "$waiting" ] || [ "$status" = waiting ]; then
      OUTCOME="waiting:$waiting"
      return
    fi
    # State before pid: a finished session reads blocked with no pid (#2821).
    if [ "$state" = blocked ] || [ "$state" = done ] || [ "$state" = stopped ] ||
      [ "$state" = failed ] || [ "$status" = idle ]; then
      ended=$((ended + 1))
      gone=0
    elif [ "$state" = working ] || [ "$status" = busy ]; then
      ended=0
      if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then gone=$((gone + 1)); else gone=0; fi
    else
      ended=0
      gone=0
    fi
    # Two polls in a row, so one odd read never decides.
    if [ "$ended" -ge 2 ]; then
      if [ "$state" = failed ]; then
        OUTCOME=$(note_outcome "$n" "$since" failed)
      else
        OUTCOME=$(note_outcome "$n" "$since" ended)
      fi
      return
    fi
    if [ "$gone" -ge 2 ]; then
      OUTCOME=$(note_outcome "$n" "$since" no-process)
      return
    fi
    if [ $(($(now) - started)) -ge "$WALL_CLOCK" ]; then
      OUTCOME=wall-clock
      return
    fi
    last=$(last_activity "$wt" "$sid")
    [ "$last" -gt 0 ] || last=$started
    if [ $(($(now) - last)) -ge "$SILENCE" ]; then
      OUTCOME=silence
      return
    fi
  done
}

# A park for each thing Claude Code's agent view says a session can wait on,
# each with the reason code the run record counts (#2854).
park_waiting() {
  local n=$1 what=$2 said=$3 answer="Answer here and remove the needs-owner label."
  case "$what" in
    "permission prompt") park_issue "$n" permission_prompt "it stopped at a permission prompt" \
      "Should the crew be allowed what it asked for, or should the plan avoid it? $answer" "$said" ;;
    "input needed") park_issue "$n" input_needed "it stopped for an answer in its own chat" \
      "What should it be told? Its question is in the last words below. $answer" "$said" ;;
    "sandbox request") park_issue "$n" sandbox_request "it asked to go outside its sandbox" \
      "Should the crew be allowed past the sandbox for this, or should the plan avoid it? $answer" "$said" ;;
    "worker request") park_issue "$n" worker_request "one of its workers stopped for a decision" \
      "What should the worker be allowed, or should the plan avoid it? $answer" "$said" ;;
    "dialog open") park_issue "$n" dialog_open "a dialog opened that nobody could close" \
      "Which step opens a dialog, and can the plan avoid it? $answer" "$said" ;;
    "") park_issue "$n" waiting "it showed as waiting without saying on what" \
      "Run it again as it is, or say what to change. $answer" "$said" ;;
    *) park_issue "$n" waiting "it waited on something the loop does not know: $what" \
      "What was it waiting for, and should the plan avoid it? $answer" "$said" ;;
  esac
}

# Exit 0 to go on; 1 when a park could not label the issue; 2 when no session
# started; 3 when the notes could not be read; 4 when the session hit the
# plan's usage limit. All but 0 end the run.
run_issue() {
  local n=$1 wt id sid said since hit rc
  since=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  ATTEMPT=$(next_attempt "$n")
  ATTEMPTS=$((ATTEMPTS + 1))
  CURRENT_N=$n
  record picked --no-note --item "$(item "$n")" --url "$(issue_url "$n")" --attempt "$ATTEMPT" \
    --field "spend_24h_seen_usd=$SPEND_SEEN"
  wt=$(worktree_for "$n") || {
    park_issue "$n" no_worktree "it could not make the worktree crew-$n" "Is a branch or folder of that name in the way?"
    rc=$?
    end_attempt "$n" parked
    CURRENT_N=""
    return "$rc"
  }
  stop_live_in "$wt"
  if ! guard_worktree "$wt" || ! start_session "$wt" "$n"; then
    end_attempt "$n" failed "$wt"
    CURRENT_N=""
    return 2
  fi
  id=$STARTED_ID
  log "#$n: session $id in $wt"
  watch "$id" "$wt" "$n" "$since"
  sid=$(agent_record "$id" | jq -r '.sessionId // empty')
  [ -n "$sid" ] && SESSION_ID=$sid
  sid=$SESSION_ID
  said=$(last_words "$wt" "$sid")
  log "#$n: $(stop_session "$id")"
  # A session stopped by the plan's usage limit is not the issue's fault, so it
  # is neither parked nor announced: the run ends and the next one resumes it.
  case "$OUTCOME" in
    draft | parked | unread | waiting:*) ;;
    *)
      if hit=$(usage_limit "$wt" "$sid"); then
        log "#$n: the session hit the plan's usage limit: $hit"
        end_attempt "$n" capped "$wt"
        CURRENT_N=""
        return 4
      fi
      ;;
  esac
  rc=0
  case "$OUTCOME" in
    unread)
      log "could not read the notes on #$n, so it is neither announced nor parked"
      end_attempt "$n" stalled "$wt"
      CURRENT_N=""
      return 3
      ;;
    draft)
      end_attempt "$n" draft "$wt"
      notify "Crew: draft ready for your review on #$n"
      ;;
    parked)
      end_attempt "$n" parked "$wt"
      notify "Crew parked #$n with a question"
      ;;
    waiting:*) park_waiting "$n" "${OUTCOME#waiting:}" "$said" || rc=$? ;;
    failed) park_issue "$n" session_failed "the session failed with an error" \
      "Remove the needs-owner label to run it again as it is, or say what to change." "$said" || rc=$? ;;
    no-process) park_issue "$n" no_process "the session showed as working with no process behind it" \
      "Remove the needs-owner label to run it again as it is, or say what to change." "$said" || rc=$? ;;
    wall-clock) park_issue "$n" time_cap "it hit the $((WALL_CLOCK / 60)) minute wall clock" \
      "Should this issue be split, or run again as it is?" "$said" || rc=$? ;;
    silence) park_issue "$n" silence "nothing was written for $((SILENCE / 60)) minutes" \
      "Run it again as it is, or is something on this issue likely to hang?" "$said" || rc=$? ;;
    *) park_issue "$n" no_draft "the session ended its turn without a draft or a park note" \
      "What should the crew do next on this issue?" "$said" || rc=$? ;;
  esac
  [ "$ATTEMPT" -gt 0 ] && end_attempt "$n" parked "$wt"
  CURRENT_N=""
  return "$rc"
}

# ── The run ──────────────────────────────────────────────────────────────────

# One run at a time: a second would count the first's worktree as unfinished
# and stop its live session (#2821). A holder that died without its EXIT trap
# leaves a stale lock, as in ios-sim-lock.sh: a pid that is no crew run, or no
# pid past the grace a starting run takes to write it.
take_lock() {
  local holder stale m
  if ! mkdir "$LOCK" 2>/dev/null; then
    holder=$(cat "$LOCK/pid" 2>/dev/null)
    if [ -n "$holder" ]; then
      crew_run "$holder" && return 1
    else
      m=$(stat -c %Y "$LOCK" 2>/dev/null || stat -f %m "$LOCK" 2>/dev/null || echo 0)
      [ $(($(now) - m)) -ge "$LOCK_GRACE" ] || return 1
    fi
    # Moved aside, never removed in place: two starters that both judged it
    # stale would otherwise each clear the other's fresh lock.
    stale="$LOCK.stale.$$"
    mv "$LOCK" "$stale" 2>/dev/null || return 1
    if [ "$(cat "$stale/pid" 2>/dev/null)" != "$holder" ]; then
      [ -e "$LOCK" ] || mv "$stale" "$LOCK" 2>/dev/null
      rm -rf "$stale"
      return 1
    fi
    rm -rf "$stale"
    log "cleared the crew lock of process ${holder:-unknown}, which has gone"
    mkdir "$LOCK" 2>/dev/null || return 1
  fi
  echo $$ >"$LOCK/pid" 2>/dev/null && [ "$(cat "$LOCK/pid" 2>/dev/null)" = "$$" ]
}

# A live process running this script, not a pid reused by something else.
crew_run() {
  grep -q 'crew-loop' <<<"$(ps -o command= -p "$1" 2>/dev/null)"
}

release_lock() {
  [ "$(cat "$LOCK/pid" 2>/dev/null)" = "$$" ] && rm -rf "$LOCK"
}

# A sleep jumps the wall clock and parks the issue as a cap (#2821). Watching
# this pid, caffeinate ends with the run and leaves its signals alone.
stay_awake() {
  if command -v "$CAFFEINATE" >/dev/null 2>&1; then
    "$CAFFEINATE" -i -w $$ >/dev/null 2>&1 &
  else
    log "no caffeinate on this machine, so it may sleep during the run"
  fi
}

on_signal() {
  trap - INT TERM HUP
  log "interrupted"
  if [ -n "$CURRENT_N" ]; then
    stop_named "$CURRENT_N"
    # A stopped run still closes its record (#2854).
    end_attempt "$CURRENT_N" cancelled "$(worktrees)/crew-$CURRENT_N"
  fi
  end_run
  exit "$1"
}

issues() { if [ "$1" -eq 1 ]; then echo "1 issue"; else echo "$1 issues"; fi; }

main() {
  local count=0 tried=" " n cand candidates q qrc prc pickfail dollars rc stopped="" ver
  if [ -n "${CI:-}" ]; then
    echo "crew-loop.sh starts real sessions on this Mac, never in CI" >&2
    exit 1
  fi
  if ! take_lock; then
    log "another crew run holds $LOCK, so this one ends; remove it only if no run is going"
    exit 0
  fi
  trap release_lock EXIT
  if [ -e "$STOP_FILE" ]; then
    log "the stop file $STOP_FILE is there; remove it to run the crew"
    exit 0
  fi
  if ! ver=$("$OSTI" --version 2>/dev/null); then
    log "osti is not installed (just install in the Ostinato repo), so no run starts: the run record and the guard need it"
    exit 1
  fi
  stay_awake
  trap 'on_signal 130' INT
  trap 'on_signal 143' TERM
  trap 'on_signal 129' HUP
  if ! check_guards; then
    log "harness check 1 failed: a crew session was not refused gh pr ready, so no issue starts this run"
    notify "Crew did not start: the guards check failed"
    exit 1
  fi
  log "harness check 1 passed; caps: $MAX_ISSUES issues, \$$SPEND_CAP, $((WALL_CLOCK / 60)) min a session, $((SILENCE / 60)) min silence"
  RUN="r-$(date -u +%Y%m%d-%H%M%S)"
  if ! record run-started --cap "issues=$MAX_ISSUES" --cap "spend_24h_usd=$SPEND_CAP" \
    --cap "session_hours=$(awk -v s="$WALL_CLOCK" 'BEGIN { print s / 3600 }')" \
    --cap "silence_min=$(awk -v s="$SILENCE" 'BEGIN { print s / 60 }')" \
    --field "claude_version=$(claude --version 2>/dev/null | head -1)" --field "osti_version=$ver"; then
    RUN=""
    log "the run record could not take the run's start, so no issue starts: the spend cap reads that record"
    notify "Crew did not start: the run record could not be written"
    exit 1
  fi
  log "run $RUN started, $ver"

  while :; do
    if [ -e "$STOP_FILE" ]; then
      log "stop file found; ending the run"
      break
    fi
    if [ "$count" -ge "$MAX_ISSUES" ]; then
      log "ran $count issues, the cap for a run"
      break
    fi
    if ! dollars=$(spend) || ! [[ $dollars =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
      log "could not read the run record $RECORD, so the loop stops rather than spend blind"
      break
    fi
    if awk -v d="$dollars" -v c="$SPEND_CAP" 'BEGIN { exit !(d >= c) }'; then
      log "the loop's sessions cost \$$dollars in the last 24 hours, at or past the \$$SPEND_CAP cap"
      stopped=spend_cap
      break
    fi
    SPEND_SEEN=$dollars

    n=""
    pickfail=0
    q=$(queue)
    qrc=$?
    candidates="$(unfinished)
$q"
    for cand in $candidates; do
      case "$tried" in *" $cand "*) continue ;; esac
      tried="$tried$cand "
      "${CREW_PICK:-$root/scripts/crew-pick.sh}" "$cand" >/dev/null 2>&1
      prc=$?
      if [ "$prc" -eq 0 ]; then
        n=$cand
        break
      fi
      if [ "$prc" -eq 2 ]; then
        log "the pick could not read #$cand from GitHub"
        pickfail=1
      fi
    done
    if [ -z "$n" ]; then
      if [ "$qrc" -ne 0 ]; then
        log "could not read the ready queue"
      elif [ "$pickfail" -eq 1 ]; then
        log "no issue picked, and the pick failed on at least one"
      else
        log "no qualifying ready issue left"
      fi
      break
    fi

    run_issue "$n"
    rc=$?
    if [ "$rc" -eq 2 ]; then
      log "ending the run: no session started for #$n"
      break
    fi
    count=$((count + 1))
    if [ "$rc" -eq 1 ]; then
      log "ending the run: #$n is parked without its needs-owner label"
      break
    fi
    if [ "$rc" -eq 3 ]; then
      log "ending the run: GitHub could not be read for #$n's outcome"
      break
    fi
    if [ "$rc" -eq 4 ]; then
      log "ending the run: the plan's usage limit stopped #$n's session, so the next run resumes it"
      notify "Crew stopped at the plan's usage limit on #$n"
      stopped=usage_limit
      break
    fi
    if [ "$RECORD_FAILED" -eq 1 ]; then
      log "ending the run: the run record could not take #$n's end, so the spend cap cannot count its cost"
      break
    fi
  done
  end_run ${stopped:+"stopped=$stopped"}
  log "run over: $(issues "$count") run"
  [ "$count" -gt 0 ] && notify "Crew finished: $(issues "$count") run, details on each issue"
  exit 0
}

usage="usage: $0 [stop-session <id> | issue <issue> | lessons]"
case "${1:-}" in
  "") main ;;
  stop-session)
    [ $# -eq 2 ] || { echo "$usage" >&2; exit 2; }
    stop_session "$2"
    ;;
  issue)
    [ $# -eq 2 ] && [[ $2 =~ ^[0-9]+$ ]] || { echo "$usage" >&2; exit 2; }
    read_issues "$2" | jq -er --arg k "i$2" "$TRUSTED"'
      .data.repository[$k] // error("no issue")
      | "Issue #\(.number), \(.state), labels: \([.labels.nodes[].name] | join(", "))\n\n"
        + (if trusted_body then "Title: \(.title)\n\n\(.body)\n"
          else "Title and body left out: the issue was opened or last edited by someone other than the owner or the agent account.\n" end)
        + ([.comments.nodes[] | select(trusted) | "\nComment by \(.author.login) on \(.createdAt):\n\(.body)\n"] | join(""))' ||
      exit 2
    ;;
  lessons)
    gh issue list -R "$REPO" --label lessons-pending --state all --limit 100 --json number,comments \
      --jq "$TRUSTED"'.[] | .number as $n | .comments[]
        | select(trusted and (.body | startswith("**Lessons**"))) | "#\($n): \(.body)"' || exit 2
    ;;
  *)
    echo "$usage" >&2
    exit 2
    ;;
esac
