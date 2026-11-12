#!/usr/bin/env bash
# Self-test for scripts/crew-pick.sh (#2819). A stand-in gh on PATH serves one
# canned issue and adds any posted comment to it, so a second pass reads the
# first pass's Crew skipped comment exactly as GitHub would return it. The base
# issue qualifies; each case breaks one rule. The plan lines are #2812's and
# the evidence row is #2819's own, so the parser meets text the owner writes.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/crew-pick.sh"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
fails=0
n=0

mkdir -p "$work/bin"
cat >"$work/bin/gh" <<'GH'
#!/usr/bin/env bash
# gh exits 1 on a GraphQL error but still prints the partial data it got.
[ -f "$FAKE/down" ] && { cat "$FAKE/issue.json"; echo "gh: Field 'blockedBy' doesn't exist on type 'Issue'" >&2; exit 1; }
case "$1 $2" in
  "api graphql")
    printf '%s\n' "$@" >"$FAKE/query"
    cat "$FAKE/issue.json"
    ;;
  "api repos/"*/comments)
    [ -f "$FAKE/no-post" ] && exit 1
    body=""
    while [ $# -gt 0 ]; do
      [ "$1" = -f ] && [[ $2 == body=* ]] && body=${2#body=}
      shift
    done
    printf '%s\n' "$body" >>"$FAKE/posted"
    jq --arg b "$body" --arg t "$(cat "$FAKE/clock" 2>/dev/null || echo 2026-11-10T00:00:00Z)" \
      '.data.repository.issue.comments.nodes += [{body: $b, createdAt: $t, lastEditedAt: null}]' \
      "$FAKE/issue.json" >"$FAKE/next" && mv "$FAKE/next" "$FAKE/issue.json"
    echo '{}'
    ;;
  *) echo "stand-in gh: unexpected call: $*" >&2; exit 9 ;;
esac
GH
chmod +x "$work/bin/gh"

DONE_LINE='an agent adding `ready` to an issue is asked for the owner'"'"'s yes, and refused outright in a crew session; a crew session is refused marking a PR ready or merging it by any route; an interactive `just pr-ready` still works.'
plan_text() {
  local files=${1:-"\`docs/harness-log.md\` and the Troubleshooting table in \`docs/working-with-agents.md\`."}
  local mockup=${2:-"none, no screen."}
  printf '**Done looks like**: %s\r\n**Time budget**: one session.\r\n**Decisions taken**: one list, the rules file'"'"'s.\r\n**Files**: %s\r\n**Tests**: a battery case for each command form, each rule mutation-tested by removing its line or pattern.\r\n**Mockup**: %s\r\n**Rabbit holes**: none.\r\n**Out of scope**: merges.\r\n**Routing**: epic #2799, not on the sensitive surfaces, harness only.' \
    "${3:-$DONE_LINE}" "$files" "$mockup"
}
BODY='## What you would notice

Nothing on screen.

## Done looks like

- A plan naming `crates/intrada-ffi/` exits 3 from the park step; a plan naming none exits 0 (self-test).
- Each qualifying rule, broken on its own, skips the issue with one comment, and a second pass adds none (self-test).

## Where it lives

- Epic #2799; parent work #2803.
- Tier 2, harness only.'

# Writes the base issue to $work/<name>/issue.json, then applies a jq edit.
fixture() {
  local d="$work/$1"
  mkdir -p "$d"
  jq -n --arg body "$BODY" --arg plan "$(plan_text)" '{data: {repository: {issue: {
    state: "OPEN", body: $body,
    labels: {nodes: [{name: "documentation"}, {name: "ready"}]},
    comments: {nodes: [
      {body: "Picking this up.", createdAt: "2026-11-01T09:00:00Z", lastEditedAt: null},
      {body: $plan, createdAt: "2026-11-01T10:00:00Z", lastEditedAt: "2026-11-02T10:00:00Z"}]},
    timelineItems: {nodes: [
      {createdAt: "2026-11-01T08:00:00Z", label: {name: "horizon:now"}},
      {createdAt: "2026-11-03T10:00:00Z", label: {name: "ready"}}]},
    blockedBy: {nodes: [{number: 2700, state: "CLOSED"}]}}}}}' |
    jq "${2:-.}" "${@:3}" >"$d/issue.json"
}
I=.data.repository.issue
PLAN="$I.comments.nodes[1]"

# expect <exit> <name> <needle in stdout> <comments wanted after the run>
expect() {
  local want=$1 name=$2 needle=$3 posts=$4 out rc=0 got
  n=$((n + 1))
  out=$(FAKE="$work/$name" PATH="$work/bin:$PATH" bash "$script" 4242 2>&1) || rc=$?
  got=$(grep -c '^\*\*Crew skipped\*\*' "$work/$name/posted" 2>/dev/null || true)
  if [ "$rc" -ne "$want" ] || [[ "$out" != *"$needle"* ]] || [ "${got:-0}" -ne "$posts" ]; then
    echo "✗ $name: wanted exit $want, '$needle' and $posts comments; got exit $rc, ${got:-0} comments: $out" >&2
    fails=$((fails + 1))
  fi
}

# ── qualifies ───────────────────────────────────────────────────────────────
fixture pass
expect 0 pass "#4242 qualifies" 0
# A Tier 1 issue is parked on its whole body, so this one names no sensitive path.
fixture tier1-no-plan "$I.body = (\$b | sub(\"Tier 2\"; \"Tier 1\") | sub(\"crates/intrada-ffi/\"; \"a sensitive path\")) | $I.comments.nodes = [$I.comments.nodes[0]]" --arg b "$BODY"
expect 0 tier1-no-plan "qualifies" 0
fixture plan-done-split "$I.body = \"Tier 2.\" | $PLAN.body = \$p" --arg p "$(plan_text "" "" "the park step exits 3 (self-test); a second pass adds none (snapshot)")"
expect 0 plan-done-split "qualifies" 0
fixture screen-approved "$PLAN.body = \$p" --arg p "$(plan_text "ios/Intrada/Views/LibraryView.swift" "https://claude.ai/design/p/abc, the owner approved the mockup on 2026-11-02.")"
expect 0 screen-approved "qualifies" 0
fixture newest-plan-wins "$I.comments.nodes = [$PLAN | .body = \$old | .createdAt = \"2026-10-01T00:00:00Z\" | .lastEditedAt = null] + $I.comments.nodes" --arg old "$(plan_text "crates/intrada-ffi/src/lib.rs")"
expect 0 newest-plan-wins "qualifies" 0

# ── passed over in silence: not in the queue ────────────────────────────────
fixture not-ready "$I.labels.nodes = [{name: \"documentation\"}]"
expect 1 not-ready "not in the crew's queue" 0
fixture closed "$I.state = \"CLOSED\""
expect 1 closed "not in the crew's queue" 0
fixture down
touch "$work/down/down"
expect 2 down "could not read #4242: gh: Field 'blockedBy'" 0
fixture empty-reply '.data.repository.issue = null'
expect 2 empty-reply "no issue #4242" 0

# ── each rule broken on its own: skipped with one comment ───────────────────
fixture no-plan "$I.comments.nodes = [$I.comments.nodes[0]]"
expect 1 no-plan "No plan comment" 1
fixture no-tier "$I.body = \$b" --arg b "${BODY%Tier 2*}"
expect 0 no-tier "qualifies" 0
fixture no-tier-no-plan "$I.body = \$b | $I.comments.nodes = [$I.comments.nodes[0]]" --arg b "${BODY%Tier 2*}"
expect 1 no-tier-no-plan "a Tier 2 issue needs one" 1
fixture no-tests "$PLAN.body |= sub(\"\\\\*\\\\*Tests\\\\*\\\\*\"; \"Tests\")"
expect 1 no-tests "no Tests line" 1
fixture no-routing "$PLAN.body |= sub(\"\\\\*\\\\*Routing\\\\*\\\\*\"; \"Routing\")"
expect 1 no-routing "no Routing line" 1
fixture ready-before-edit "$PLAN.lastEditedAt = \"2026-11-04T10:00:00Z\""
expect 1 ready-before-edit "older than the plan's last edit (2026-11-04T10:00:00Z)" 1
fixture ready-before-create "$PLAN.lastEditedAt = null | $PLAN.createdAt = \"2026-11-05T10:00:00Z\""
expect 1 ready-before-create "older than the plan's last edit (2026-11-05T10:00:00Z)" 1
fixture no-ready-event "$I.timelineItems.nodes = [$I.timelineItems.nodes[0]]"
expect 1 no-ready-event "(added never)" 1
fixture own-evidence-row "$I.body = \$b" --arg b "${BODY/
## Where/- Harness check 5 passes.

## Where}"
expect 1 own-evidence-row "names no test, snapshot or screenshot as its evidence: \"Harness check 5 passes.\"" 1
fixture star-bullet "$I.body = \$b" --arg b "${BODY/- Each/* Each qualifying rule holds.
- Each}"
expect 1 star-bullet "\"Each qualifying rule holds.\"" 1
fixture plan-done-no-evidence "$I.body = \"Tier 2.\" | $PLAN.body = \$p" --arg p "$(plan_text "" "" "the park step exits 3 (self-test); a second pass adds none")"
expect 1 plan-done-no-evidence "evidence: \"a second pass adds none\"" 1
fixture no-done-lines "$I.body = \"Tier 1.\" | $I.comments.nodes = [$I.comments.nodes[0]]"
expect 1 no-done-lines "No Done looks like lines" 1
fixture call-in-body "$I.body = \$b" --arg b "${BODY/(self-test)./(self-test), but the wording needs your call.}"
expect 1 call-in-body "still needs your call" 1
fixture call-in-plan "$PLAN.body = \$p" --arg p "$(plan_text "" "" "the park step exits 3 (self-test); Needs your call on the wording")"
expect 1 call-in-plan "still needs your call" 1
fixture screen-no-mockup "$PLAN.body = \$p" --arg p "$(plan_text "ios/Intrada/Views/LibraryView.swift")"
expect 1 screen-no-mockup "needs a Mockup line" 1
fixture android-screen "$PLAN.body = \$p" --arg p "$(plan_text "android/app/src/main/kotlin/com/intrada/android/ui/library/LibraryScreen.kt")"
expect 1 android-screen "needs a Mockup line" 1
fixture screen-waiting "$PLAN.body = \$p" --arg p "$(plan_text "ios/Intrada/Views/LibraryView.swift" "https://claude.ai/design/p/abc, waiting for the owner's word on the mockup.")"
expect 1 screen-waiting "needs a Mockup line" 1
fixture screen-no-link "$PLAN.body = \$p" --arg p "$(plan_text "ios/Intrada/Views/LibraryView.swift" "approved on 2026-11-02, canvas to follow.")"
expect 1 screen-no-link "needs a Mockup line" 1
fixture blocked "$I.blockedBy.nodes += [{number: 2810, state: \"OPEN\"}, {number: 2811, state: \"OPEN\"}]"
expect 1 blocked "Blocked by #2810, #2811, still open." 1
fixture sensitive "$PLAN.body = \$p" --arg p "$(plan_text "crates/intrada-ffi/src/lib.rs and its tests.")"
expect 1 sensitive "sensitive surface, which the lead builds: crates/intrada-ffi/src/lib.rs matches crates/intrada-ffi/**" 1
fixture tier1-sensitive "$I.body = (\$b | sub(\"Tier 2\"; \"Tier 1, in crates/intrada-store/src/lib.rs\")) | $I.comments.nodes = [$I.comments.nodes[0]]" --arg b "$BODY"
expect 1 tier1-sensitive "crates/intrada-store/src/lib.rs matches" 1
fixture sensitive-only-outside-files "$PLAN.body |= sub(\"one list\"; \"crates/intrada-ffi stays alone, one list\")"
expect 0 sensitive-only-outside-files "qualifies" 0
fixture park-broken
n=$((n + 1))
out=$(CREW_PARK_RULES="$work/nowhere.md" FAKE="$work/park-broken" PATH="$work/bin:$PATH" bash "$script" 4242 2>&1) && rc=0 || rc=$?
if [ "$rc" -ne 1 ] || [[ "$out" != *"park step could not run: crew-park: no paths: list"* ]]; then
  echo "✗ park-broken: wanted exit 1 and the park step's error, got (exit $rc): $out" >&2
  fails=$((fails + 1))
fi

# ── one comment until the plan or the label changes ─────────────────────────
fixture two-reasons "$I.comments.nodes = [$I.comments.nodes[0]] | $I.blockedBy.nodes += [{number: 2810, state: \"OPEN\"}]"
expect 1 two-reasons "Blocked by #2810" 1
n=$((n + 1))
if ! grep -q 'No plan comment' "$work/two-reasons/posted"; then
  echo "✗ two-reasons: the one comment should list both reasons" >&2
  fails=$((fails + 1))
fi
edit_again() { jq "$1" "$work/again/issue.json" >"$work/again/next" && mv "$work/again/next" "$work/again/issue.json"; }
fixture again "$I.blockedBy.nodes += [{number: 2810, state: \"OPEN\"}]"
expect 1 again "Commented on #4242." 1
expect 1 again "Already said on #4242" 1
echo 2026-11-12T00:00:00Z >"$work/again/clock"
edit_again "$I.timelineItems.nodes += [{createdAt: \"2026-11-11T00:00:00Z\", label: {name: \"ready\"}}]"
expect 1 again "Commented on #4242." 2
expect 1 again "Already said on #4242" 2
echo 2026-11-14T00:00:00Z >"$work/again/clock"
edit_again "$PLAN.lastEditedAt = \"2026-11-13T00:00:00Z\""
expect 1 again "Commented on #4242." 3
fixture old-skip "$I.comments.nodes += [{body: \"**Crew skipped**: old reasons\", createdAt: \"2026-11-02T12:00:00Z\", lastEditedAt: null}] | $PLAN.lastEditedAt = \"2026-11-04T10:00:00Z\""
expect 1 old-skip "Commented on #4242." 1
fixture plain-skip-heading "$I.comments.nodes += [{body: \"Crew skipped: said by hand\", createdAt: \"2026-11-05T12:00:00Z\", lastEditedAt: null}] | $PLAN.lastEditedAt = \"2026-11-04T10:00:00Z\""
expect 1 plain-skip-heading "Already said on #4242" 0
fixture edited-after-skip "$I.comments.nodes += [{body: \"**Crew skipped**: before the edit\", createdAt: \"2026-11-05T12:00:00Z\", lastEditedAt: null}] | $PLAN.lastEditedAt = \"2026-11-06T10:00:00Z\""
expect 1 edited-after-skip "Commented on #4242." 1
fixture post-fails "$I.blockedBy.nodes += [{number: 2810, state: \"OPEN\"}]"
touch "$work/post-fails/no-post"
expect 1 post-fails "could not post the Crew skipped comment" 0

for bad in abc "4242 4243"; do
  n=$((n + 1))
  rc=0
  # shellcheck disable=SC2086 # the second case is two arguments on purpose
  out=$(FAKE="$work/pass" PATH="$work/bin:$PATH" bash "$script" $bad 2>&1) || rc=$?
  if [ "$rc" -ne 2 ] || [[ "$out" != *usage* ]]; then
    echo "✗ usage '$bad': wanted exit 2 and the usage line, got (exit $rc): $out" >&2
    fails=$((fails + 1))
  fi
done

# The stand-in serves every field whatever was asked, so the query itself is
# checked: each field the rules read, braces balanced, the issue number passed.
n=$((n + 1))
q=$(cat "$work/pass/query")
for want in 'state body' 'labels(first: 50) { nodes { name } }' 'comments(last: 100) { nodes { body createdAt lastEditedAt } }' \
  'timelineItems(itemTypes: [LABELED_EVENT]' 'createdAt label { name }' 'blockedBy(first: 50) { nodes { number state } }' \
  'repository(owner: $owner, name: $name)' 'issue(number: $n)' 'n=4242' 'owner=jonyardley' 'name=intrada'; do
  if [[ "$q" != *"$want"* ]]; then
    echo "✗ query: no '$want' in what was sent" >&2
    fails=$((fails + 1))
  fi
done
if [ "$(printf '%s' "$q" | tr -cd '{' | wc -c)" -ne "$(printf '%s' "$q" | tr -cd '}' | wc -c)" ]; then
  echo "✗ query: braces do not balance" >&2
  fails=$((fails + 1))
fi

if [ "$fails" -ne 0 ]; then
  echo "✗ crew-pick: $fails of $n cases wrong" >&2
  exit 1
fi
echo "✓ crew-pick: $n cases, each qualifying rule skipping once and only once"
