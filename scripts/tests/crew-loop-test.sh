#!/usr/bin/env bash
# Self-test for scripts/crew-loop.sh (#2821). Stand-in `claude` and `gh` on
# PATH play one scripted session per issue; the loop's other seams (queue,
# pick, check 1, spend, worktree, notify) point at small stand-ins. Each case
# sets one condition and checks what the loop did on the fake GitHub. Every
# case ends with the guard that the loop never called a ready or merge route.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/crew-loop.sh"

work="$(mktemp -d)"
bgpids=""
cleanup() {
  for p in $bgpids $(cat "$work"/fake/pid-* 2>/dev/null); do kill "$p" 2>/dev/null || :; done
  rm -rf "$work"
}
trap cleanup EXIT
fails=0
n=0

mkdir -p "$work/bin"

# Issue N's session plays $FAKE/scenario-N (#2821): draft, parked, ended, permission,
# noproc, busy, broken (unreadable start), done-draft or done-quiet (no pid).
cat >"$work/bin/claude" <<'CLAUDE'
#!/usr/bin/env bash
printf 'claude %s\n' "$*" >>"$FAKE/calls"
case "$1" in
  --bg)
    printf 'INTRADA_CREW=%s\n' "${INTRADA_CREW:-unset}" >>"$FAKE/calls"
    issue=${*: -1}; issue=${issue#/crew }
    id=$(printf 'c%07d' "$issue")
    printf '%s %s %s\n' "$id" "$issue" "$PWD" >>"$FAKE/sessions"
    [ -f "$FAKE/stop-on-start" ] && touch "$(cat "$FAKE/stop-on-start")"
    [ -f "$FAKE/down-on-start" ] && cp "$FAKE/down-on-start" "$FAKE/gh-down-$issue"
    [ -f "$FAKE/slow-start" ] && sleep 2
    scenario=$(cat "$FAKE/scenario-$issue" 2>/dev/null || echo busy)
    case $scenario in
      noproc | done-* | stopped) ;;
      *)
        nohup bash -c 'exec -a claude-session sleep 300' >/dev/null 2>&1 &
        echo $! >"$FAKE/pid-$issue"
        ;;
    esac
    case $scenario in
      draft | parked | done-draft)
        word=${scenario#done-}
        jq --arg b "**Crew: $word**: done" --arg t "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
          '.comments += [{body: $b, createdAt: $t}]' "$FAKE/issue-$issue.json" >"$FAKE/next" &&
          mv "$FAKE/next" "$FAKE/issue-$issue.json"
        ;;
    esac
    if [ "$scenario" = broken ]; then
      echo "error: something went wrong"
    else
      echo "backgrounded · $id · crew-$issue"
    fi
    ;;
  agents)
    # $FAKE/empty-reads holds how many reads come back empty first.
    left=$(cat "$FAKE/empty-reads" 2>/dev/null || echo 0)
    if [ "$left" -gt 0 ]; then
      echo $((left - 1)) >"$FAKE/empty-reads"
      echo '[]'
      exit 0
    fi
    # $FAKE/cycle holds states the watched session takes in turn, one a read:
    # alive, dead (working, its pid gone), blocked (no pid) or empty.
    if [ -s "$FAKE/cycle" ]; then
      set -- $(cat "$FAKE/cycle")
      i=$(cat "$FAKE/cycle-at" 2>/dev/null || echo 0)
      echo $((i + 1)) >"$FAKE/cycle-at"
      shift $((i % $#))
      cyc=$1
      [ "$cyc" = empty ] && { echo '[]'; exit 0; }
    fi
    while read -r id issue cwd; do
      scenario=$(cat "$FAKE/scenario-$issue" 2>/dev/null || echo busy)
      pid=$(cat "$FAKE/pid-$issue" 2>/dev/null || echo 999999)
      case ${cyc:-} in
        dead) scenario=busy; pid=999999 ;;
        blocked) scenario=done-quiet ;;
      esac
      case $scenario in
        done-*) st='"state":"blocked","status":null,"waitingFor":null'; pid=null ;;
        stopped) st='"state":"stopped","status":null,"waitingFor":null'; pid=null ;;
        draft | parked | ended) st='"state":"done","status":"idle","waitingFor":null' ;;
        permission) st='"state":"blocked","status":"idle","waitingFor":"permission prompt"' ;;
        *) st='"state":"working","status":"busy","waitingFor":null' ;;
      esac
      [ -f "$FAKE/stopped-$id" ] && st='"state":"stopped","status":"idle","waitingFor":null'
      kind=$(cat "$FAKE/kind-$issue" 2>/dev/null || echo background)
      printf '{"id":"%s","kind":"%s","pid":%s,"sessionId":"sess-%s","name":"crew-%s","cwd":"%s",%s}\n' \
        "$id" "$kind" "$pid" "$issue" "$issue" "$cwd" "$st"
    done <"$FAKE/sessions" | jq -s .
    ;;
  stop) : >"$FAKE/stopped-$2" ;;
  -p)
    printf '%s\n' "${*: -1}" >>"$FAKE/pushed"
    echo $$ >"$FAKE/push-pid"
    trap "" TERM
    exec sleep 300
    ;;
  *) echo "stand-in claude: unexpected call: $*" >&2; exit 9 ;;
esac
CLAUDE

cat >"$work/bin/gh" <<'GH'
#!/usr/bin/env bash
printf 'gh %s\n' "$*" >>"$FAKE/gh.log"
n=$3
f="$FAKE/issue-$n.json"
jqx=""; body=""; label=""
args=("$@")
for i in "${!args[@]}"; do
  case "${args[$i]}" in
    --jq) jqx=${args[$((i + 1))]} ;;
    --body) body=${args[$((i + 1))]} ;;
    --add-label) label=${args[$((i + 1))]} ;;
  esac
done
case "$1 $2" in
  "issue view")
    # $FAKE/gh-down-N: empty fails every read, a number fails that many.
    if [ -f "$FAKE/gh-down-$n" ]; then
      left=$(cat "$FAKE/gh-down-$n")
      [ -z "$left" ] && exit 1
      [ "$left" -gt 0 ] && { echo $((left - 1)) >"$FAKE/gh-down-$n"; exit 1; }
    fi
    jq -r "${jqx:-.}" "$f"
    ;;
  "issue edit")
    [ -f "$FAKE/label-fails-$n" ] && exit 1
    jq --arg l "$label" '.labels += [{name: $l}]' "$f" >"$FAKE/next" && mv "$FAKE/next" "$f" ;;
  "issue comment")
    jq --arg b "$body" --arg t "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
      '.comments += [{body: $b, createdAt: $t}]' "$f" >"$FAKE/next" && mv "$FAKE/next" "$f" ;;
  *) echo "stand-in gh: unexpected call: $*" >&2; exit 9 ;;
esac
GH

cat >"$work/bin/queue" <<'Q'
#!/usr/bin/env bash
[ -f "$FAKE/queue-down" ] && exit 2
cat "$FAKE/queue.json"
Q
cat >"$work/bin/pick" <<'P'
#!/usr/bin/env bash
printf '%s\n' "$1" >>"$FAKE/picked"
[ -f "$FAKE/pick-down-$1" ] && exit 2
[ ! -f "$FAKE/skip-$1" ]
P
cat >"$work/bin/spend" <<'S'
#!/usr/bin/env bash
cat "$FAKE/spend"
S
cat >"$work/bin/mkwt" <<'W'
#!/usr/bin/env bash
mkdir -p "$2"
W
cat >"$work/bin/notify" <<'N'
#!/usr/bin/env bash
printf '%s\n' "$1" >>"$FAKE/notified"
N
cat >"$work/bin/osascript" <<'O'
#!/usr/bin/env bash
printf '%s\n' "${*: -1}" >>"$FAKE/osascript"
O
# Harness check 1: writes a result, as crew-qualify.sh does, and keeps its folder.
cat >"$work/bin/qualify" <<'C'
#!/usr/bin/env bash
echo "$2" >"$FAKE/check1-dir"
echo "${CHECK1:-PASS} 1: refused" >"$2/results.txt"
C
chmod +x "$work/bin/"*

# A fresh fake world: queue holds the given issues, each open and ready.
setup() {
  export FAKE="$work/fake"
  for p in $(cat "$FAKE"/pid-* 2>/dev/null); do kill "$p" 2>/dev/null || :; done
  cat "$FAKE/gh.log" >>"$work/gh-all.log" 2>/dev/null || :
  rm -rf "$FAKE" "$work/home" "$work/wt" "$work/tasks" "$work/stop"
  mkdir -p "$FAKE" "$work/home" "$work/wt" "$work/tmp"
  : >"$FAKE/sessions"
  : >"$FAKE/gh.log"
  echo 'Claude usage, last 1d: $40 at API prices; opus 90%' >"$FAKE/spend"
  local q='[]' i
  for i in "$@"; do
    q=$(printf '%s' "$q" | jq --argjson n "$i" '. + [{number: $n, labels: {nodes: [{name: "ready"}]}}]')
    echo '{"state":"OPEN","labels":[{"name":"ready"}],"comments":[]}' >"$FAKE/issue-$i.json"
  done
  printf '%s' "$q" >"$FAKE/queue.json"
}

# "$BASH", so a run of this file under /bin/bash runs the loop under it too.
# BG=1 leaves the loop running, its pid in LOOP_PID.
run_loop() {
  set -- env -u CI PATH="$work/bin:$PATH" HOME="$work/home" TMPDIR="$work/tmp" \
    CREW_QUEUE="$work/bin/queue" CREW_PICK="$work/bin/pick" CREW_SPEND="$work/bin/spend" \
    CREW_WORKTREE="$work/bin/mkwt" CREW_WORKTREES="$work/wt" CREW_NOTIFY="${NOTIFY-$work/bin/notify}" \
    CREW_QUALIFY="$work/bin/qualify" CHECK1="${CHECK1:-PASS}" CREW_STOP_FILE="$work/stop" \
    CREW_LOG="$work/log" CREW_TASKS_ROOT="$work/tasks" CREW_PUSH_LIMIT=1 \
    CREW_POLL="${POLL:-0}" CREW_KILL_GRACE=0 IOS_SIM_LOCK_DIR="$work/simlock" \
    CREW_WALL_CLOCK="${WALL:-5}" CREW_SILENCE="${SILENCE:-600}" CREW_MAX_ISSUES="${MAX:-3}" \
    "$BASH" "$script" "$@"
  if [ -n "${BG:-}" ]; then
    "$@" >"$work/out" 2>&1 &
    LOOP_PID=$!
  else
    "$@" >"$work/out" 2>&1 || :
  fi
}

ok() {
  n=$((n + 1))
  if eval "$2"; then
    echo "ok $n - $1"
  else
    echo "not ok $n - $1"
    sed 's/^/    /' "$work/out" | tail -8
    fails=$((fails + 1))
  fi
}

labels() { jq -r '.labels[].name' "$FAKE/issue-$1.json"; }
notes() { jq -r '.comments[].body' "$FAKE/issue-$1.json"; }
started() { awk '{print $2}' "$FAKE/sessions" | paste -sd ' ' -; }
never_ready_or_merge() {
  ! grep -qE 'pr ready|pr merge|(--add-label|labels\[\]=) ?ready( |$)|markPullRequestReadyForReview|/merge' "$1"
}
alive() { kill -0 "$(cat "$FAKE/pid-$1")" 2>/dev/null; }

# ── before any issue ─────────────────────────────────────────────────────────

setup 101
CHECK1=FAIL run_loop
ok "a failed harness check 1 starts no session" '[ -z "$(started)" ] && grep -q "check 1 failed" "$work/out"'
ok "harness check 1's folder is removed after it is read" '[ -s "$FAKE/check1-dir" ] && [ ! -e "$(cat "$FAKE/check1-dir")" ]'

setup 101
echo draft >"$FAKE/scenario-101"
run_loop
ok "a passing harness check 1 lets the run start, and its folder is removed" \
  '[ "$(started)" = 101 ] && [ ! -e "$(cat "$FAKE/check1-dir")" ]'

setup 101
touch "$work/stop"
run_loop
rm -f "$work/stop"
ok "the stop file starts no session, nor harness check 1" \
  '[ -z "$(started)" ] && [ ! -e "$FAKE/check1-dir" ] && grep -q "remove it to run the crew" "$work/out"'

setup 101 102
echo draft >"$FAKE/scenario-101"
echo draft >"$FAKE/scenario-102"
echo "$work/stop" >"$FAKE/stop-on-start"
run_loop
rm -f "$work/stop"
ok "a stop file that appears during an issue starts no further issue" \
  '[ "$(started)" = 101 ] && grep -q "stop file found; ending the run" "$work/out"'

setup 101
echo 'Claude usage, last 1d: $151 at API prices' >"$FAKE/spend"
run_loop
ok "spend past the cap starts no session" '[ -z "$(started)" ] && grep -q "past the \$150 cap" "$work/out"'

setup 101
echo 'no report' >"$FAKE/spend"
run_loop
ok "an unreadable spend report starts no session" '[ -z "$(started)" ] && grep -q "spend blind" "$work/out"'

setup 101
echo 'Claude usage, last 1d: nothing recorded' >"$FAKE/spend"
echo draft >"$FAKE/scenario-101"
run_loop
ok "a day with nothing recorded counts as no spend" '[ "$(started)" = 101 ]'

setup 101
touch "$FAKE/queue-down"
run_loop
ok "a failed queue read starts no session and says so" '[ -z "$(started)" ] && grep -q "could not read the ready queue" "$work/out"'

setup 101
touch "$FAKE/pick-down-101"
run_loop
ok "a pick that cannot read GitHub is logged as a pick failure" \
  '[ -z "$(started)" ] && grep -q "the pick could not read #101" "$work/out" && grep -q "the pick failed on at least one" "$work/out" && ! grep -q "no qualifying ready issue left" "$work/out"'

setup 101
touch "$FAKE/skip-101"
run_loop
ok "a queue with nothing that qualifies says so" '[ -z "$(started)" ] && grep -q "no qualifying ready issue left" "$work/out"'

# ── the queue ────────────────────────────────────────────────────────────────

setup 101 102 103
echo draft >"$FAKE/scenario-102"
echo draft >"$FAKE/scenario-103"
jq '.[0].labels.nodes += [{name: "needs-owner"}]' "$FAKE/queue.json" >"$FAKE/q" && mv "$FAKE/q" "$FAKE/queue.json"
MAX=1 run_loop
ok "an issue labelled needs-owner is never picked" '! grep -qx 101 "$FAKE/picked"'
ok "the first remaining ready issue runs, and the cap for a run stops the next" '[ "$(started)" = 102 ]'

setup 101 102
touch "$FAKE/skip-101"
echo draft >"$FAKE/scenario-102"
run_loop
ok "an issue that does not qualify is passed over for the next" '[ "$(started)" = 102 ]'

setup 101 102 103
for i in 101 102 103; do echo draft >"$FAKE/scenario-$i"; done
MAX=2 run_loop
ok "the cap of issues a run ends the loop" '[ "$(started)" = "101 102" ] && grep -q "the cap for a run" "$work/out"'

setup 101
echo '{"state":"OPEN","labels":[{"name":"ready"},{"name":"in-flight"}],"comments":[{"body":"**Crew: built**: two commits","createdAt":"2026-11-01T00:00:00Z"}]}' >"$FAKE/issue-205.json"
echo '{"state":"OPEN","labels":[{"name":"ready"},{"name":"needs-owner"}],"comments":[{"body":"**Crew: parked**: a question","createdAt":"2026-11-01T00:00:00Z"}]}' >"$FAKE/issue-204.json"
echo '{"state":"OPEN","labels":[{"name":"ready"},{"name":"in-flight"}],"comments":[{"body":"**Crew: draft**: #3000","createdAt":"2026-11-01T00:00:00Z"}]}' >"$FAKE/issue-203.json"
echo '{"state":"CLOSED","labels":[{"name":"ready"}],"comments":[]}' >"$FAKE/issue-202.json"
mkdir -p "$work/wt/crew-202" "$work/wt/crew-203" "$work/wt/crew-204" "$work/wt/crew-205"
echo draft >"$FAKE/scenario-205"
echo draft >"$FAKE/scenario-101"
MAX=1 run_loop
ok "a restart takes an unfinished crew worktree before the queue" '[ "$(started)" = 205 ]'
ok "a restart leaves a worktree still waiting on the owner" '! grep -qx 204 "$FAKE/picked"'
ok "a restart leaves a worktree whose draft is open" '! grep -qx 203 "$FAKE/picked"'
ok "a restart leaves a worktree whose issue is closed" '! grep -qx 202 "$FAKE/picked"'

setup
echo '{"state":"OPEN","labels":[{"name":"ready"}],"comments":[{"body":"**Crew: parked**: a question","createdAt":"2026-11-01T00:00:00Z"}]}' >"$FAKE/issue-208.json"
mkdir -p "$work/wt/crew-208"
run_loop
ok "a restart leaves a parked issue with no later comment, even without its label" \
  '[ -z "$(started)" ] && ! grep -qsx 208 "$FAKE/picked"'

setup
echo '{"state":"OPEN","labels":[{"name":"ready"}],"comments":[]}' >"$FAKE/issue-206.json"
touch "$FAKE/gh-down-206"
mkdir -p "$work/wt/crew-206"
run_loop
ok "a restart skips a worktree whose issue cannot be read" \
  '[ -z "$(started)" ] && ! grep -qsx 206 "$FAKE/picked" && grep -q "could not read #206" "$work/out"'

setup 101
echo '{"state":"OPEN","labels":[{"name":"ready"}],"comments":[{"body":"**Crew: parked**: a question","createdAt":"2026-11-01T00:00:00Z"},{"body":"Do it the second way.","createdAt":"2026-11-01T01:00:00Z"}]}' >"$FAKE/issue-207.json"
mkdir -p "$work/wt/crew-207"
echo ended >"$FAKE/scenario-207"
MAX=1 run_loop
ok "an issue the owner answered resumes, and an earlier run's park note is not this run's outcome" \
  '[ "$(started)" = 207 ] && [ "$(notes 207 | grep -c "^\*\*Crew: parked")" = 2 ] && notes 207 | grep -q "without a draft or a park note"'

# ── one issue's outcomes ─────────────────────────────────────────────────────

setup 101
echo draft >"$FAKE/scenario-101"
run_loop
ok "a draft is announced, with no park" \
  'grep -q "draft ready for your review on #101" "$FAKE/notified" && ! labels 101 | grep -qx needs-owner && grep -q "run over: 1 issue run" "$work/out" && grep -qx "Crew finished: 1 issue run, details on each issue" "$FAKE/notified"'
ok "the session is started as the crew, on Opus at high effort, with the brief" \
  'grep -qE -- "--bg --model opus --effort high --permission-mode auto -n crew-101 /crew 101" "$FAKE/calls" && grep -qx INTRADA_CREW=1 "$FAKE/calls"'
ok "the session is started in its own crew worktree" 'grep -q " 101 $work/wt/crew-101$" "$FAKE/sessions"'

setup 101
echo parked >"$FAKE/scenario-101"
run_loop
ok "a session that parked itself gets no second note" '[ "$(notes 101 | grep -c "Crew: parked")" = 1 ] && grep -q "parked #101 with a question" "$FAKE/notified"'

setup 101 102
echo permission >"$FAKE/scenario-101"
echo draft >"$FAKE/scenario-102"
run_loop
ok "a permission prompt parks: label, note and the owner's phone" \
  'labels 101 | grep -qx needs-owner && notes 101 | grep -q "permission prompt" && grep -q "Crew parked #101" "$FAKE/notified"'
ok "a parked issue's session is stopped and its process killed" '[ -f "$FAKE/stopped-c0000101" ] && ! alive 101'
ok "after a park the loop takes the next issue" '[ "$(started)" = "101 102" ]'

setup 101
echo noproc >"$FAKE/scenario-101"
run_loop
ok "a session working with no process behind it parks" 'notes 101 | grep -q "no process behind it" && labels 101 | grep -qx needs-owner'

setup 101
echo done-draft >"$FAKE/scenario-101"
run_loop
ok "a finished turn with no pid and a draft note is a draft" \
  'grep -q "draft ready for your review on #101" "$FAKE/notified" && ! labels 101 | grep -qx needs-owner'

setup 101
echo done-quiet >"$FAKE/scenario-101"
run_loop
ok "a finished turn with no pid and no note is an ended turn, not a missing process" \
  'notes 101 | grep -q "without a draft or a park note" && ! notes 101 | grep -q "no process"'

setup 101
echo 1 >"$FAKE/empty-reads"
WALL=1 run_loop
ok "a single empty read of the sessions never parks" 'notes 101 | grep -q "wall clock" && ! notes 101 | grep -q "no process"'

# Reads that alternate never make two in a row, so only the wall clock ends these.
for cycle in "blocked alive" "dead blocked" "dead empty" "blocked empty"; do
  setup 101
  echo "$cycle" >"$FAKE/cycle"
  WALL=1 run_loop
  ok "reads alternating $cycle never decide the turn" 'notes 101 | grep -q "wall clock"'
done

setup 101 102
echo permission >"$FAKE/scenario-101"
echo draft >"$FAKE/scenario-102"
touch "$FAKE/label-fails-101"
run_loop
ok "a park that cannot add needs-owner ends the run" \
  '[ "$(started)" = 101 ] && grep -q "could not add needs-owner to #101" "$work/out" && grep -q "parked without its needs-owner label" "$work/out"'

setup 101 102
echo broken >"$FAKE/scenario-101"
echo draft >"$FAKE/scenario-102"
run_loop
ok "a session that cannot be started ends the run without parking" \
  '[ "$(started)" = 101 ] && ! labels 101 | grep -qx needs-owner && [ -z "$(notes 101)" ] && grep -q "could not start a session for #101" "$work/out" && grep -q "no session started for #101" "$work/out"'
ok "a start whose output cannot be read stops any session of that name" '[ -f "$FAKE/stopped-c0000101" ] && ! alive 101'
ok "a failed start does not count as an issue run" '! grep -qs "Crew finished" "$FAKE/notified"'

setup 101 102
echo draft >"$FAKE/scenario-101"
echo draft >"$FAKE/scenario-102"
: >"$FAKE/down-on-start"
run_loop
ok "a draft whose notes cannot be read ends the run, neither announced nor parked" \
  '[ "$(started)" = 101 ] && ! grep -qs "draft ready" "$FAKE/notified" && ! labels 101 | grep -qx needs-owner && [ "$(notes 101 | grep -c "Crew: parked")" = 0 ] && grep -q "could not read the notes on #101" "$work/out" && grep -q "GitHub could not be read for #101" "$work/out"'

setup 101
echo draft >"$FAKE/scenario-101"
echo 1 >"$FAKE/down-on-start"
run_loop
ok "one failed read of the notes is tried again" 'grep -q "draft ready for your review on #101" "$FAKE/notified"'

setup 101
echo stopped >"$FAKE/scenario-101"
run_loop
ok "a session that reads stopped lets the notes decide" 'notes 101 | grep -q "without a draft or a park note"'

setup 101
echo draft >"$FAKE/scenario-101"
touch "$FAKE/slow-start"
BG=1 run_loop
for _ in 1 2 3 4 5 6 7 8 9 10; do [ -s "$FAKE/sessions" ] && break; sleep 0.2; done
kill -TERM "$LOOP_PID"
wait "$LOOP_PID" 2>/dev/null || :
ok "TERM during a start stops the session it started" '[ -f "$FAKE/stopped-c0000101" ] && ! alive 101'

setup 101
echo draft >"$FAKE/scenario-101"
nohup bash -c 'exec -a claude-old sleep 304' >/dev/null 2>&1 &
echo $! >"$FAKE/pid-777"
disown "$!"
nohup bash -c 'exec -a claude-term sleep 306' >/dev/null 2>&1 &
echo $! >"$FAKE/pid-779"
disown "$!"
echo interactive >"$FAKE/kind-779"
printf 'c0000777 777 %s\nc0000778 778 %s\nc0000779 779 %s\n' "$work/wt/crew-101" "$work/wt/crew-101" "$work/wt/crew-101" >"$FAKE/sessions"
run_loop
ok "a live session already in the worktree is stopped before a new one starts" \
  '[ -f "$FAKE/stopped-c0000777" ] && ! kill -0 "$(cat "$FAKE/pid-777")" 2>/dev/null && [ ! -f "$FAKE/stopped-c0000778" ]'
ok "a session in the worktree that is not a background one is left alone" \
  '[ ! -f "$FAKE/stopped-c0000779" ] && kill -0 "$(cat "$FAKE/pid-779")"'

setup 101
POLL=5 BG=1 run_loop
for _ in 1 2 3 4 5 6 7 8 9 10; do [ -s "$FAKE/sessions" ] && break; sleep 0.5; done
sleep 1
t0=$(date +%s)
kill -TERM "$LOOP_PID"
wait "$LOOP_PID" 2>/dev/null || :
t1=$(date +%s)
ok "TERM stops the session being watched, at once" \
  '[ -f "$FAKE/stopped-c0000101" ] && ! alive 101 && [ $((t1 - t0)) -le 2 ] && grep -q interrupted "$work/out"'
ok "the watch waits between polls" '[ "$(grep -c "^claude agents" "$FAKE/calls")" -le 4 ]'

setup 101
proj="$work/home/.claude/projects/$(printf '%s' "$work/wt/crew-101" | sed 's/[^A-Za-z0-9]/-/g')"
mkdir -p "$proj"
long="START$(printf 'x%.0s' $(seq 1 400)) ask @someone about \`\`\`this\`\`\`"
jq -cn --arg t "$long" '{type: "assistant", message: {content: [{type: "text", text: $t}]}}' >"$proj/sess-101.jsonl"
echo permission >"$FAKE/scenario-101"
run_loop
ok "the park note keeps the last 300 characters inside one closed fence" \
  'body=$(notes 101) && printf "%s" "$body" | grep -q "@someone" && ! printf "%s" "$body" | grep -q START && [ "$(printf "%s\n" "$body" | grep -c "^\`\`\`")" = 2 ]'

setup 101
echo draft >"$FAKE/scenario-101"
t0=$(date +%s)
NOTIFY= run_loop
t1=$(date +%s)
ok "a notification passes the text whole and a push that ignores TERM is killed at its limit" \
  'grep -qx "Crew: draft ready for your review on #101" "$FAKE/osascript" && grep -q "then stop: Crew: draft ready" "$FAKE/pushed" && ! kill -0 "$(cat "$FAKE/push-pid")" 2>/dev/null && [ $((t1 - t0)) -lt 15 ]'

setup 101
echo ended >"$FAKE/scenario-101"
run_loop
ok "a turn that ends with no draft or park note parks with a question" 'notes 101 | grep -q "without a draft or a park note"'

setup 101
WALL=0 run_loop
ok "the wall clock parks a busy session" 'notes 101 | grep -q "wall clock" && ! alive 101'

setup 101
SILENCE=0 run_loop
ok "the silence limit parks a quiet session" 'notes 101 | grep -q "nothing was written for 0 minutes"'

setup 101
proj="$work/home/.claude/projects/$(printf '%s' "$work/wt/crew-101" | sed 's/[^A-Za-z0-9]/-/g')"
mkdir -p "$proj/sess-101/subagents"
touch -t 202601010000 "$proj/sess-101.jsonl"
touch "$proj/sess-101/subagents/agent-a.jsonl"
(while sleep 1; do touch "$proj/sess-101/subagents/agent-a.jsonl"; done) &
bgpids="$bgpids $!"
disown "$!"
POLL=1 SILENCE=3 WALL=5 run_loop
kill "${bgpids##* }" 2>/dev/null || :
ok "a subagent's writes keep a quiet lead from the silence limit" 'notes 101 | grep -q "wall clock" && ! notes 101 | grep -q "nothing was written"'

setup 101
proj="$work/home/.claude/projects/$(printf '%s' "$work/wt/crew-101" | sed 's/[^A-Za-z0-9]/-/g')"
tasks="$work/tasks/$(printf '%s' "$work/wt/crew-101" | sed 's/[^A-Za-z0-9]/-/g')/sess-101/tasks"
mkdir -p "$proj" "$tasks"
touch -t 202601010000 "$proj/sess-101.jsonl"
touch "$tasks/b1.output"
(while sleep 1; do touch "$tasks/b1.output"; done) &
bgpids="$bgpids $!"
disown "$!"
POLL=1 SILENCE=3 WALL=5 run_loop
kill "${bgpids##* }" 2>/dev/null || :
ok "a background task's output keeps a quiet lead from the silence limit" 'notes 101 | grep -q "wall clock" && ! notes 101 | grep -q "nothing was written"'

# ── stop-session, the park step check 7 calls ────────────────────────────────

setup
nohup bash -c 'sleep 301 & sleep 302 & wait' claude-tree >/dev/null 2>&1 &
parent=$!
disown "$parent"
sleep 1
kids=$(pgrep -P "$parent" | paste -sd ' ' -)
printf 'c0000999 999 %s\n' "$work" >"$FAKE/sessions"
echo "$parent" >"$FAKE/pid-999"
mkdir -p "$work/simlock"
echo "${kids%% *}" >"$work/simlock/pid"
run_loop stop-session c0000999
gone=yes
for p in $parent $kids; do kill -0 "$p" 2>/dev/null && gone=no; done
ok "stop-session kills the session and every process under it" '[ "$gone" = yes ] && grep -q "stopped c0000999 and 3 processes (0 needed kill -9)" "$work/out"'
ok "stop-session releases a simulator lock the session held" '[ ! -d "$work/simlock" ]'

setup
nohup sleep 303 >/dev/null 2>&1 &
other=$!
disown "$other"
bgpids="$bgpids $other"
printf 'c0000998 998 %s\n' "$work" >"$FAKE/sessions"
echo 999999 >"$FAKE/pid-998"
mkdir -p "$work/simlock"
echo "$other" >"$work/simlock/pid"
run_loop stop-session c0000998
ok "stop-session leaves a simulator lock another process holds" '[ -d "$work/simlock" ]'

setup
nohup bash -c 'trap "" TERM; exec -a claude-stubborn sleep 305' >/dev/null 2>&1 &
stubborn=$!
disown "$stubborn"
sleep 0.5
printf 'c0000997 997 %s\n' "$work" >"$FAKE/sessions"
echo "$stubborn" >"$FAKE/pid-997"
run_loop stop-session c0000997
ok "stop-session kills with -9 a process that ignores TERM" \
  '! kill -0 "$stubborn" 2>/dev/null && grep -q "stopped c0000997 and 1 processes (1 needed kill -9)" "$work/out"'

setup
nohup sleep 307 >/dev/null 2>&1 &
stranger=$!
disown "$stranger"
bgpids="$bgpids $stranger"
printf 'c0000996 996 %s\n' "$work" >"$FAKE/sessions"
echo "$stranger" >"$FAKE/pid-996"
run_loop stop-session c0000996
ok "stop-session leaves a pid that is not a claude process" \
  'kill -0 "$stranger" && grep -q "pid $stranger of c0000996 is not a claude process" "$work/out"'

# ── never a ready or merge ───────────────────────────────────────────────────

printf 'gh pr merge 1\ngh issue edit 1 --add-label ready\n' >"$work/planted.log"
ok "the guard below catches a planted merge and ready label" \
  '! never_ready_or_merge "$work/planted.log" && ! never_ready_or_merge <(sed 1d "$work/planted.log")'
setup
ok "across every case above the loop called GitHub, and never a ready or merge route" \
  'grep -q "add-label needs-owner" "$work/gh-all.log" && never_ready_or_merge "$work/gh-all.log"'

echo
if [ "$fails" -eq 0 ]; then
  echo "$n passed, 0 failed"
else
  echo "$((n - fails)) passed, $fails failed"
  exit 1
fi
