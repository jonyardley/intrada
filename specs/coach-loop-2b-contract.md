# Coach loop 2b — bridge contract

Temporary. Folds into `specs/intrada-coach-engine.md` §4 and §6 once the slice
lands (#1223, #1224); this file is deleted in the same PR.

The one sentence the whole slice turns on: **the click runs unbroken for a whole
block, and the shell can tell "keep clicking" from "restart the click" without
knowing a single domain rule.**

## 1. Continuous pulse

`BlockState::rep_seq` is gone. It meant "a rep began", which is now the wrong
question: a rep beginning is exactly the moment the click must *not* be touched.
Its replacement is `pulse_seq`, which means only one thing:

```rust
/// Bumped when the shell must stop the click and start it again, count-in
/// first. Unchanged means keep clicking, whatever else moved in this view.
pub pulse_seq: u32,
```

It is bumped in exactly three places: a block opening (`BlockState::open` sets
it to 1), an escalation rung that changed a click parameter
(`TempoDown` / `ShrinkScope`), and `RecoverSession`. It is **not** bumped by a
tap, a discard, a beat or a gate opening.

`pulse_seq` is block-scoped, so the shell keys the running click on
`(block_index, pulse_seq)` — the pair `DrillLoopHost` already keys on.

The pulse is unbounded in reps: the shell schedules `count_in_beats` count-in
clicks and then body beats **forever**, until `pulse_running` goes false, the
key changes, or the drill view disappears. It tops up a rolling window; nothing
in the view bounds the schedule.

The shell's whole rule, with no domain reasoning in it:

| view says | shell does |
|---|---|
| `pulse_running == false` | stop the click, forget the key |
| `pulse_running == true`, key differs from the sounding one | stop, start again with a count-in |
| `pulse_running == true`, key unchanged | nothing at all |

Beat reporting is unchanged in shape and changed in range: `Beat { beat_index }`
stays 0-based from the first body beat, and now counts **monotonically across
the whole pulse** rather than restarting each rep. The core derives the rep from
it (`beat_index % phrase_beats`), so a rep boundary is a beat like any other.

`DrillView::click_beats` (body + 1 landing beat) is replaced by:

```rust
/// Beats in one pass of the phrase. A pass completes every `phrase_beats`
/// beats; the landing beat the player aims at is the next pass's downbeat,
/// which is why there is no longer a "+ 1".
pub phrase_beats: u32,
```

### What a rep boundary does now

`Phase::AwaitingVerdict` no longer waits with the click stopped; it rests for
exactly one phrase cycle while the hands play the next pass.

- boundary beat (`beat_index > 0 && beat_index % phrase_beats == 0`) →
  `AwaitingVerdict` opens on the pass that just finished
- any other beat → `Listening`, and the verdict glance clears (T10, unchanged)
- a beat arriving while `AwaitingVerdict` is open does **not** close the window
- the next boundary closes an untapped window with no attempt recorded, and
  opens a fresh one on the pass that just finished

So a tap always judges exactly one bounded pass, and there is never more than
one candidate pass to judge. Decision 17 is untouched: `Tap { clean }` is still
a verdict on a bounded attempt, and `CleanPasses { count, consecutive }` still
sees attempt boundaries. The only thing that changed is that not tapping costs
an unrecorded pass instead of freezing the loop.

`Tap` no longer resets `beat_index` or restarts the pulse. It records the
attempt, resolves the gate/ladder as it does today, and returns to `Listening`
with `last_verdict` set — so the glance still draws until the next beat clears
it, about half a second at 120bpm, exactly as T10 specifies.

## 2. Discard

```rust
/// "Don't count that." A false start, or a pass the user does not want on the
/// record. Records nothing at all.
DiscardAttempt { now: DateTime<Utc> },
```

Legal in `Listening` and `AwaitingVerdict`; ignored everywhere else (there is
nothing to discard during a count-in, an escalation, an open gate or a
block-entry card).

- from `AwaitingVerdict`: closes the open window and lands in `Listening`. The
  pass in flight is untouched and will open its own window.
- from `Listening`: sets a `discarded` flag on the block, and lands in
  `Listening`. The next boundary consumes the flag and opens no window — the
  false start goes round again with nothing asked of it.

It appends no `AttemptSummary`, emits no `ScoredAttempt`, does not touch
`consecutive_fails`, `gate_progress`, `gate_opened_at_attempt` or
`reps_after_gate`, and never reaches the mastery track or the Beta estimate.
Its only writes are `block.now = now` and the flag.

## 3. Skip

```rust
/// Close this block now and move on. The card's secondary action.
SkipBlock { now: DateTime<Utc> },
```

Legal in any phase of a `Running` session, including `BlockEntry`. Closes the
block with the existing `Exit::Skipped` and advances
(`close_block(Exit::Skipped, true)`): the next block's entry card, or `Closing`
if it was the last. A block skipped from its card writes a real `BlockRecord`
with no attempts — a skip is a fact worth keeping, not an absence.

## 4. Block entry

A new phase, not a new `SessionState` and not a new `CoachView` field:

```rust
pub enum Phase   { BlockEntry, CountIn { .. }, Listening, AwaitingVerdict, Escalating { .. }, GateOpen }
pub enum DrillPhase { BlockEntry, Playing, CountIn { .. }, AwaitingVerdict, Acknowledged { .. }, GateOpen }
```

Why a phase: everything the card draws is already `DrillView` — title, section,
kind, block position, and now minutes and the why line. A new `SessionState`
variant would need the plan cursor rebuilt in it and would break `spec()`,
`mark_cold()` and recovery; a new `CoachView` field would make two surfaces
compete for "is a block running". A phase keeps one drill surface with two
faces, and `SessionState::Running` keeps meaning what it means.

`BlockState::open` now opens in `Phase::BlockEntry` with `pulse_running` false,
so a session starts on block 1's card rather than straight into a count-in, and
every block close lands on the next card rather than auto-advancing.

```rust
/// Leave the block-entry card and start the block.
StartBlock { now: DateTime<Utc> },
```

Legal only in `Phase::BlockEntry`. Re-stamps `started_at = now` (the card is not
practice time, and the ceiling starts when the hands do) and moves to
`Phase::CountIn`. `Tick` never closes a block on its ceiling while the card is
up.

Recovery does **not** land on a card: `RecoverSession` returns a running block
to its count-in as it does today, because a recovered block is mid-flight rather
than newly entered, and re-entering it would refund minutes already spent.

Two new `DrillView` fields, both written by the core:

```rust
/// The block's length, for the entry card. `ceiling_seconds` stays the
/// during-play ceiling.
pub minutes: u16,
/// One sentence, from the planner. The shell renders it and never composes
/// one — same rule as `PlannedBlockView::why`.
pub why: String,
```

## 5. Click placement (#1224)

`DrillView::click_level` stays a display `String` (the pill). Placement crosses
as a mask, not as the `ClickLevel` enum: re-exporting the enum would put a
`switch` over domain semantics in the shell, which is the dumb-pipe rule's
whole objection.

```rust
/// One cycle of the click placement, from the pulse's first body beat.
/// `click_pattern[beat_index % click_pattern.len()]` sounds; the rest are
/// silent beats the shell still reports. The count-in clicks every beat
/// regardless of level.
pub click_pattern: Vec<bool>,
```

`ClickLevel::pattern(beats_per_bar)` builds it, and the cycle is a bar or two
bars — never the phrase, because placement is a bar-level fact and a phrase of
an odd number of bars would otherwise flip `EveryOtherBar`'s alternation on
every pass:

| level | cycle | sounding beats (0-based) |
|---|---|---|
| `EveryBeat` | `beats_per_bar` | all |
| `TwoAndFour` | `beats_per_bar` | 1 and 3, dropped if the bar is shorter |
| `BarDownbeat` | `beats_per_bar` | 0 |
| `EveryOtherBar` | `2 * beats_per_bar` | 0 |

The mask is aligned to `beat_index` 0, which is bar 1 beat 1 of the pulse, so
`ShrinkScope` (which restarts the pulse) re-aligns it and nothing drifts.
Accents stay the shell's business: it already accents a bar downbeat, and an
accent on a beat the mask silences is simply not played.

## 6. Bridge-crossing types changed

`CoachEvent` (+3 variants), `Phase` (+1 variant), `DrillPhase` (+1 variant),
`DrillView` (−`click_beats`, −`rep_seq`, +`phrase_beats`, +`pulse_seq`,
+`pulse_running`, +`click_pattern`, +`minutes`, +`why`), `BlockState`
(−`rep_seq`, +`pulse_seq`, +`discarded`) and therefore `EngineSession`, which
`RecoverSession` carries.

All of them are positional bincode on the wire. No `deserialize_with`, no
`skip_serializing_if`, no `Option<Option<_>>` anywhere in the additions.
`assert_round_trips` covers every one of them, and the `LiveBridge` round-trip
test in `StoreEffectLoopTests` covers the events the shell sends.
