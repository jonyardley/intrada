# Built session, play-through altitudes, and qualitative capture

**Status**: Phase D core landed (Journey C's rules); D-screens next ·
**Issue**: #1256 · **Tier**: 3
**Design**: `specs/built-session/design/` (Built Session A/B/C mockups) ·
briefs in `design/briefs/2026-08-built-session-journeys.md` (journeys) and
`design/briefs/2026-08-copy-language.md` (voice, graduated as T13)

## Problem

The old `SessionBuilderScreen` (hand-assembled setlists, 1–5 self-ratings) was
deleted in #1255 (close-out of #1190). Its replacement is decision 19's
**built session**: the user composes today's blocks from their own items, each
block keeps a gate, and evidence lands in the mastery model exactly as a
prescribed block's does. Alongside it sit decision 16's two lower altitudes
(off-piste, unmonitored play) and decision 17's qualitative capture. None of
this exists yet; the journeys are designed and judged right (Built Session
A/B/C, 2026-08-07).

## The three journeys (what ships)

**A — compose from a lesson.** A steer line under the untouched Practice hero
("I know what I want to practise today") opens a sheet, not a mode (decision
11). Each new item resolves one of three ways (decision 19):

- (a) **matches an authored node** — proposal + one-tap confirm; "No, it's
  different" falls through to (b) with the name pre-filled.
- (b) **countable, no node → user drill** — one dictated/typed sentence is the
  criterion; tempo, key and passes parse into read-back chips, not empty
  fields. Low-band prior, cold-testable, own mastery, optional *serves* tag.
- (c) **genuinely unmeasurable → journal item** — opaque target on the
  judgement track; time and notes, kept with the piece.

Resolution is paid once per item, ever: repeat visits are add-add-add-build.
The composed session reuses the press-start scaffold; template shape (warm-up
first, music at the ends) is a one-line declinable suggestion. Start enters
the existing drill loop; a user drill's boundary card shows GateDots filling
from tap-verdicts and the serves line.

**B — play-through, three altitudes.** From a piece, the B0 sheet asks "Play
it through — how should it count?" (headline settled, Jon 2026-08-07 on
#1256): **run-through** (section-by-section gated run, "Held / Broke down",
one tap each), **off-piste** (time logged + a 96pt "Found something? Say it"
mic; exit offers "Keep this as a drill?" routing into A's form with the
transcript as draft criterion), **just play** (minutes only, silent middle, no
exit prompt ever). The AltitudeChip stays visible in the orientation strip for
the whole run; absence of instrumentation is the consent signal.

**C — qualitative capture.** At most one feel moment per block ("Fought it /
Getting there / It sang"), only where feel is the point, skipped entirely
after two misses in a block (the budget shrinks on a bad day). Voice-first
reflection at session close (audio kept, transcription opportunistic). A kept
reflection can resurface next morning as a proposed steer: quote back, one
concrete offer, accept inserts one block marked "you added this" (decision 12:
propose, confirm, never plan).

## Approach

Core owns everything (Crux, local-first, offline-first invariants apply in
full). New domain surface, sketched for Phase A:

- **Entities** (GRDB, client ulids, `updated_at`/`deleted_at`): `UserDrill`
  (criterion text, parsed gate params, serves tag, low-band mastery state),
  `JournalItem`, `BuiltSession` (ordered blocks + provenance), `PlayThrough`
  (per-section verdicts for run-throughs; Phase C split the other two
  altitudes into `WanderRecord` and `UnmonitoredRecord`, which carry elapsed),
  `Reflection` / feel entries (judgement track: never feed mastery, may retire
  a target, decision 17).
- **Events/Effects**: compose/resolve/reorder events; existing drill-loop
  events reused unchanged for gated blocks; a new audio-capture `AppEffect`
  for voice notes (record + opportunistic transcription in the shell as a dumb
  pipe; the core stores the marker, path and transcript).
- **Evidence**: a user drill's tap-verdicts land at full weight on its own
  node; a run-through's section verdicts land on the pipeline stage; per the
  #1244 ruling, the tap bounds the attempt at l0 (`TapVerdictUntimed`, lower
  weight, already covers the looser bound). Off-piste and unmonitored produce
  time entries only, with zero inference (decision 16).
- **Resolution matching (a)**: v1 matches typed/dictated names against library
  and node names in core (normalised text match); LLM-proposed matching and
  the C3 morning proposal's LLM narration are Phase 3 of the coach roadmap;
  C3 ships rule-based (most recent kept reflection with a named target) or not
  at all in v1. The criterion-sentence parsing (tempo/keys/passes) is a small
  core parser over dictated text, not an LLM call.
- **Voice/copy**: every screen is written against T13. Engine vocabulary never
  appears on screen; in-session prose ≤ 8 words beyond labels.

### New design-system components (promoted with the screen that ships them)

`AltitudeChip` (always visible while playing), `FeelChips` (one-per-block rule
in its DS entry), `ReflectionCard` + proposed-steer card, journal kind badge
(third kind beside piece/exercise), read-back chips (A4), the inline
shape-advice card (CoachNote-weight, two inline choices), GateDots at section
granularity (one-line DS note). Each lands in `Theme.swift` + the design
system together; no hand-rolled clones.

### Phasing

- **A (landed)**: spec + design assets + core scaffold: entity types,
  events, model fields, migrations, bridge round-trip tests for every new
  payload (build precondition, #846), no UI.
- **B (landed)**: Journey A end-to-end (steer line, sheet, resolution,
  composed session, drill-loop integration, evidence landing). A composed
  session becomes an ordinary `Plan` and runs the canonical drill loop; what
  differs per block is only what its taps may mean, which is `BlockOrigin`'s
  job. A user drill's rung comes from its own sentence — a tempo gives a
  clicked rung, no tempo gives l0 — and its gate is the parsed passes (or key
  coverage, where the sentence named keys).
- **C**: Journey B (B0 sheet, run-through with section gates, off-piste,
  unmonitored; AltitudeChip). **Ships as two PRs**: C-core (the altitudes as
  engine states, the section gates, evidence and replay, the wander tag and its
  migration) is reviewed first, and C-screens follows against a settled
  contract. The core/screens split is the rule #1283 adds to CLAUDE.md, off the
  back of Phase B landing 4,300 insertions in one PR with both self-review
  blockers in core code written in the first third.
- **D**: Journey C (feel moments, reflection at close, morning proposal).
  **Ships as two PRs** like C: D-core (the feel budget, the close offer, the
  rule-based steer and its migration) is reviewed first, D-screens follows
  against a settled contract.

Each phase is its own PR; every screen ships with snapshots, VoiceOver labels,
Dynamic Type and iPad SplitView per the per-screen quality rule.

## Key decisions (settled; do not reopen)

1. Never a mode (11): composition is a steer reached from the hero; altitudes
   are reached from the piece.
2. Three-way resolution (19), template shape is advice, resolution paid once.
3. Qualitative data never feeds mastery (17); measurement budget: one tap per
   rep, ≤ 1 feel per block, budget shrinks on a bad day.
4. Tap bounds the l0 attempt (#1244 ruling); B0 headline stays "how should it
   count?" (#1256 comment; record in the design file when next open).
5. Voice is T13; serif is reserved for the user's own words.
6. **An unreadable stored value quarantines its row** (#1269, settled in
   Phase B): reported, left out of the load, and left untouched on disk. The
   two options the issue offered were preserve-verbatim and fail-the-decode;
   this is the second, scoped to the row rather than the whole load. A session
   decoded *partially* — a dropped block, a defaulted enum — would be saved
   back over the only copy of the user's data with less than it had, and a row
   the model never holds is a row no write can reach. One bad row costs that
   row, not the library, and a newer binary still reads it whole.
7. **Off-piste carries the piece; unmonitored carries nothing** (Phase C). B0
   is reached from a piece for all three altitudes, but only off-piste keeps
   the tag: it already writes a record, because it captures. Unmonitored's
   record is an id and two instants and nothing else, so there is nothing to
   tag and nothing that could later say what was played. The asymmetry is the
   consent gradient, not an oversight — "minutes only" means minutes.

   **Amended 2026-08-08 (#1285).** This decision originally read "unmonitored
   writes no record at all", and the engine took it literally: the minutes were
   computed on close into a field nothing persisted, so decision 16's one
   promised output died with the process. The consent rule was never about
   writing nothing — it is about writing nothing *attributable*. So the minutes
   land in their own `unmonitored_play` table (v14), whose columns are `id`,
   `started_at`, `ended_at` and the two sync columns. Deliberately **not** a
   discriminated `wander_record`: that table has an `item_id`, an `attempts`
   blob and the keep-prompt answer, so reusing it would leave "minutes only"
   resting on every future writer remembering the discriminator. A table with
   no column for a piece cannot be made to name one. The rows are write-only,
   as wanders already are — nothing reads them back, because reading them into
   anything is the inference decision 16 forbids.
8. **The feel question follows `BlockOrigin`, and the reflection follows the
   altitude** (Phase D). C1 is asked only where a block closed on the
   judgement track, never alongside GateDots, and never after two misses in
   that block — one question per block, and the budget shrinks on a bad day.
   C2 is offered once as a session closes and never after unmonitored play,
   whose promptless exit is the whole of what "minutes only" agreed to, nor
   after off-piste, which has already asked twice on the way out.

   Both prompts live on the `Model`, not on `EngineSession`. A prompt lost to
   a crash costs nothing — the record it follows is already written — and a
   field anywhere in the crash-recovery graph costs every device its blob.

9. **An accepted steer is one column, not a banked block** (Phase D). The
   reflection carries `steer` and `steer_at`; the block is re-derived from
   them each time a plan is made, and rides for twenty-four hours from the
   accept. A block held only in memory would be lost by the relaunch that
   remakes the plan, while the answered column would stop it ever being
   proposed again — losing the steer silently, which is the #846 class.

   **Accepting also places the block there and then**, in the same handler,
   rather than waiting for the next planning run. The Practice screen asks for
   a plan only when it has none, so a steer that waited would leave the card up
   over an unchanged plan for the rest of the app run — and neither of the two
   halves the C3 frame shows at once would be true. The re-derivation on each
   plan is what survives a relaunch; the immediate placement is what makes the
   accept mean something. Placing the same steer twice is refused by node, and
   in a running session it goes after the block in flight, never before it.

10. **Judgement-track blocks are enforced by `BlockOrigin`, not by convention.**
   It rides the spec *and* the record, because the mastery track is rebuilt
   from records at launch: a decision-17 rule the live path enforces and the
   replay does not is not a rule (the #1214 class).

## Open questions

1. ~~How much pipeline does v1 need?~~ **Resolved in Phase C: none.** What a
   run-through gates on is the piece's own named chart sections — the `[A]`,
   `[Bridge]` labels already in `Item.chord_chart`, in reading order. No stage
   graph, no authored pipeline entity, nothing between the piece and its
   sections; the data is the user's own and reads in their words. This is B1's
   note taken literally, and the degradation it asked for falls out: a piece
   with no chart, or a chart with no labelled section, cannot be run through,
   and B0 offers off-piste and just-play instead (saying why, rather than
   hiding the option).

   Evidence lands per section, on `piece:<item_id>#<label>`, at l0 — the tap is
   what bounds the attempt there (#1244). **Nothing lands on the piece as a
   whole**: Phase B's interim stands, because "the piece held" is still not a
   claim anything can make. A whole-piece verdict is what a single unnamed
   section would be, which is why an unlabelled chart is refused rather than
   run on one gate.

   **The known cost**: evidence is label-addressed, so renaming `[Bridge]` to
   `[B]` in the chart orphans that section's mastery silently. Accepted for v1
   because the alternative is stable per-section ids on a chart the user edits
   as text, which is a bigger change than the altitude needs — tracked in #1287
   to fix before there is much user data to strand.
2. ~~Audio storage/retention for voice notes and reflections.~~ **Settled in
   the Phase A schema review**: `reflection` carries `audio_path` (a
   shell-relative path) and `duration_s`; the bytes are the shell's, the core
   never reads them, and a tombstoned row leaves its file for the shell to
   reap. A size cap and the reaping pass are deferred to Phase D, when there
   is recorded audio to cap (#1267).
3. ~~C3's v1 trigger (rule-based vs deferred to coach Phase 3)?~~ **Resolved in
   Phase D: rule-based, and deliberately timid.** The most recent unanswered
   session-close reflection between six and twenty hours old is scanned
   sentence by sentence; the first sentence naming exactly one thing the
   library can resolve — a user drill, a journal target, a piece, or a chart
   section label — is quoted back verbatim with one eight-minute offer.

   The thresholds each buy something. Six hours stops the app quoting someone
   back to themselves the same evening, which reads as strange rather than
   attentive; twenty is the outer edge of a card that says "last night", and
   the reason the scan never falls through to the reflection behind an
   unresolvable one. Names below three characters never match, because charts
   are routinely labelled `[A]` and "a" is the commonest word in English — a
   floor is what stops "It was a good session" proposing the piece with an A
   section. A quote longer than thirty words is declined rather than shown,
   because dictation often returns a whole reflection with no full stop in it
   and the card is designed around one short thought.

   **Exactly one** is the important one: an ambiguous name proposes nothing,
   because a wrong quote-back is worse than no card at all — the whole
   affordance rests on the user recognising their own words and the app
   having understood them.

   **The known cost**: a reflection whose target the library cannot name never
   returns, however useful it was. That is the LLM's job in coach Phase 3, and
   the rules here are the floor it replaces, not a design it has to keep.
