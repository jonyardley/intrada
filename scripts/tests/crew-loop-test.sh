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

# The session's scenario for issue N is in $FAKE/scenario-N: draft, parked,
# ended, permission, noproc or busy.
cat >"$work/bin/claude" <<'CLAUDE'
#!/usr/bin/env bash
printf 'claude %s\n' "$*" >>"$FAKE/calls"
case "$1" in
  --bg)
    issue=${*: -1}; issue=${issue#/crew }
    id=$(printf 'c%07d' "$issue")
    printf '%s %s %s\n' "$id" "$issue" "$PWD" >>"$FAKE/sessions"
    scenario=$(cat "$FAKE/scenario-$issue" 2>/dev/null || echo busy)
    if [ "$scenario" != noproc ]; then
      nohup sleep 300 >/dev/null 2>&1 &
      echo $! >"$FAKE/pid-$issue"
    fi
    case $scenario in
      draft | parked)
        word=$scenario
        jq --arg b "**Crew: $word**: done" --arg t "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
          '.comments += [{body: $b, createdAt: $t}]' "$FAKE/issue-$issue.json" >"$FAKE/next" &&
          mv "$FAKE/next" "$FAKE/issue-$issue.json"
        ;;
    esac
    echo "backgrounded · $id · crew-$issue"
    ;;
  agents)
    while read -r id issue cwd; do
      scenario=$(cat "$FAKE/scenario-$issue" 2>/dev/null || echo busy)
      pid=$(cat "$FAKE/pid-$issue" 2>/dev/null || echo 999999)
      case $scenario in
        draft | parked | ended) st='"state":"done","status":"idle","waitingFor":null' ;;
        permission) st='"state":"blocked","status":"idle","waitingFor":"permission prompt"' ;;
        *) st='"state":"working","status":"busy","waitingFor":null' ;;
      esac
      [ -f "$FAKE/stopped-$id" ] && st='"state":"stopped","status":"idle","waitingFor":null'
      printf '{"id":"%s","pid":%s,"sessionId":"sess-%s","cwd":"%s",%s}\n' "$id" "$pid" "$issue" "$cwd" "$st"
    done <"$FAKE/sessions" | jq -s .
    ;;
  stop) : >"$FAKE/stopped-$2" ;;
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
  "issue view") jq -r "${jqx:-.}" "$f" ;;
  "issue edit")
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
chmod +x "$work/bin/"*

# A fresh fake world: queue holds the given issues, each open and ready.
setup() {
  export FAKE="$work/fake"
  for p in $(cat "$FAKE"/pid-* 2>/dev/null); do kill "$p" 2>/dev/null || :; done
  cat "$FAKE/gh.log" >>"$work/gh-all.log" 2>/dev/null || :
  rm -rf "$FAKE" "$work/home" "$work/wt"
  mkdir -p "$FAKE" "$work/home" "$work/wt"
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

run_loop() {
  env -u CI PATH="$work/bin:$PATH" HOME="$work/home" \
    CREW_QUEUE="$work/bin/queue" CREW_PICK="$work/bin/pick" CREW_SPEND="$work/bin/spend" \
    CREW_WORKTREE="$work/bin/mkwt" CREW_WORKTREES="$work/wt" CREW_NOTIFY="$work/bin/notify" \
    CREW_CHECK1="${CHECK1:-true}" CREW_STOP_FILE="$work/stop" CREW_LOG="$work/log" \
    CREW_POLL="${POLL:-0}" CREW_KILL_GRACE=0 IOS_SIM_LOCK_DIR="$work/simlock" \
    CREW_WALL_CLOCK="${WALL:-5}" CREW_SILENCE="${SILENCE:-600}" CREW_MAX_ISSUES="${MAX:-3}" \
    bash "$script" "$@" >"$work/out" 2>&1 || :
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
CHECK1=false run_loop
ok "a failed harness check 1 starts no session" '[ -z "$(started)" ] && grep -q "check 1 failed" "$work/out"'

setup 101
touch "$work/stop"
run_loop
rm -f "$work/stop"
ok "the stop file starts no session" '[ -z "$(started)" ]'

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

# ── the queue ────────────────────────────────────────────────────────────────

setup 101 102 103
echo draft >"$FAKE/scenario-102"
echo draft >"$FAKE/scenario-103"
jq '.[0].labels.nodes += [{name: "needs-owner"}]' "$FAKE/queue.json" >"$FAKE/q" && mv "$FAKE/q" "$FAKE/queue.json"
MAX=1 run_loop
ok "an issue labelled needs-owner is never picked" '! grep -qx 101 "$FAKE/picked"'
ok "the first remaining ready issue runs, and the nightly cap stops the next" '[ "$(started)" = 102 ]'

setup 101 102
touch "$FAKE/skip-101"
echo draft >"$FAKE/scenario-102"
run_loop
ok "an issue that does not qualify is passed over for the next" '[ "$(started)" = 102 ]'

setup 101 102 103
for i in 101 102 103; do echo draft >"$FAKE/scenario-$i"; done
MAX=2 run_loop
ok "the nightly cap of issues ends the loop" '[ "$(started)" = "101 102" ] && grep -q "nightly cap" "$work/out"'

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

setup 101
echo '{"state":"OPEN","labels":[{"name":"ready"}],"comments":[{"body":"**Crew: parked**: a question","createdAt":"2026-11-01T00:00:00Z"},{"body":"Do it the second way.","createdAt":"2026-11-01T01:00:00Z"}]}' >"$FAKE/issue-207.json"
mkdir -p "$work/wt/crew-207"
echo ended >"$FAKE/scenario-207"
MAX=1 run_loop
ok "an issue the owner answered resumes, and last night's park note is not tonight's outcome" \
  '[ "$(started)" = 207 ] && [ "$(notes 207 | grep -c "^\*\*Crew: parked")" = 2 ] && notes 207 | grep -q "without a draft or a park note"'

# ── one issue's outcomes ─────────────────────────────────────────────────────

setup 101
echo draft >"$FAKE/scenario-101"
run_loop
ok "a draft is announced, with no park" 'grep -q "draft ready for your review on #101" "$FAKE/notified" && ! labels 101 | grep -qx needs-owner'
ok "the session is started as the crew, on Opus at high effort, with the brief" \
  'grep -qE -- "--bg --model opus --effort high --permission-mode auto -n crew-101 /crew 101" "$FAKE/calls"'
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
(while sleep 1; do touch "$proj/sess-101/subagents/agent-a.jsonl"; done) &
bgpids="$bgpids $!"
disown "$!"
POLL=1 SILENCE=3 WALL=5 run_loop
kill "${bgpids##* }" 2>/dev/null || :
ok "a subagent's writes keep a quiet lead from the silence limit" 'notes 101 | grep -q "wall clock" && ! notes 101 | grep -q "nothing was written"'

# ── stop-session, the park step check 7 calls ────────────────────────────────

setup
nohup bash -c 'sleep 301 & sleep 302 & wait' >/dev/null 2>&1 &
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
