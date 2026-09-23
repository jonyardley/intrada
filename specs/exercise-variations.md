# Practice variations, and the record of what you played

> Tier 3 spec. Issue [#1739], milestone v0.13.0. **Supersedes decisions 4 and 5
> of the #1083 spec**: the ladder and its
> current rung go, and a session's record of one item stops being a single
> slot. The decisions that still hold from the mechanism that shipped in
> #1112 and #1118 are in [`exercise-linking-origins.md`](exercise-linking-origins.md).
>
> **Language.** On screen a variation is a "variation", never a step (#1733),
> and a repetition is a "repetition", never a pass (#1735). The core type keeps
> the name `Variant` it already has; renaming a Rust type buys nothing and
> costs a diff across every call site.
>
> **Scope:** `intrada-core` and native iOS. Plays live on the device only, as
> `variant_id` and `group_id` do. The API and its server were deleted on
> 2026-09-12 (#1746), so there is no server side to keep compiling.

[#1739]: https://github.com/jonyardley/intrada/issues/1739

## Problem

Practise a scale exercise in C for three minutes, switch to D and give it
seven, and the app cannot record it. `SetlistEntry` holds one `variant_id`,
one `score`, one `rep_count`, one `achieved_tempo` and one `click_pattern`, so
the item-complete sheet asks you to pick a single variation and whichever you
pick throws the rest away. Practising one exercise across several keys in a
sitting is the normal case, not an edge case, and it is the thing a practice
notebook exists to record.

## Approach

A variation stays a definition owned by the exercise. What changes is the
record: an entry holds an ordered list of **plays**, and each play carries the
variation, the time on it, the repetitions, the tempo reached and the mark.
Switching variation mid item closes the open play and opens a new one.

```text
Item (exercise)
  └─ variants: Vec<Variant { id, label, position, updated_at, deleted_at }>  // unchanged
PracticeSession
  └─ entries[]: SetlistEntry {
       …, planned_variation_id: Option<String>,   // the plan (Building phase)
       plays: Vec<VariationPlay>                  // the record (Active, Summary)
     }
       └─ VariationPlay { id, variation_id: Option<String>, started_at,
                          seconds, rep_target, rep_count, rep_history,
                          achieved_tempo, click_pattern, score }
```

## Key decisions

1. **A variation is unordered.** `position` survives as display order only.
   The current-rung concept goes: `VariantView.is_current` is deleted, and
   with it the rule that the current step is the first that is not solid.
   `is_solid` stays, as a per-variation state rather than a position on a
   ladder. "What should I practise next" is a recommendation, not a rung, and
   it is [#1501]'s job. This is the product change inside #1733's rename, and
   it is deliberate: the word variation was chosen to stop the screen
   implying an order that the material does not have.

2. **Plays live inside the entry, one entry per item.** The setlist carries
   `position`, `group_id` blocks and the builder's plan, so entries appearing
   and disappearing during practice would make the plan and the record the
   same object. One entry, a list of plays inside it, keeps "what I set out to
   do" and "what I did" separable, which is what every later analytics
   question needs.

3. **Every practised entry has at least one play, including pieces.** An item
   with no live variations, or a piece, gets exactly one play with
   `variation_id: None`. The player and the item-complete sheet then have one
   code path rather than two, and an unattributed play is an honest record
   rather than a missing one. An entry never practised (skipped, not
   attempted) has zero plays.

4. **The per-play fields leave the entry.** `score`, `rep_target`,
   `rep_count`, `rep_target_reached`, `rep_history`, `achieved_tempo` and
   `click_pattern` move onto `VariationPlay` and are removed from
   `SetlistEntry`. Two places holding one number is how this bug arrived; it
   does not get to stay for convenience. The entry keeps `id`, `item_id`,
   `item_title`, `item_type`, `position`, `duration_secs`, `status`, `notes`,
   `intention`, `planned_duration_secs` and `group_id`, and gains
   `planned_rep_target`: `SetRepTarget` is a Building-phase event, so the
   builder's target is a plan on the same footing as decision 5's
   `planned_variation_id`, and every play the entry opens starts from it.

5. **`variant_id` becomes `planned_variation_id` and stays on the entry.** A
   refinement of decision 4, not an exception to it: the Building phase's
   "which variation am I about to practise" is a plan, and plans belong on the
   entry. `SessionEvent::SetEntryVariant` keeps its Building-phase meaning and
   loses its Active and Summary meaning. `StartSession` seeds the first play
   from it.

6. **Switching is explicit, never inferred.** A variation picker in the
   player, and `SessionEvent::SwitchVariation { entry_id, variation_id }`
   (Active only) closes the open play, stamping its seconds, and opens a new
   one. Inference would have to guess when the switch happened, and the
   repetition counter resets on a switch, so a wrong guess corrupts two
   numbers rather than one. Switching to the variation already open is a
   no-op, so a stray tap cannot clear the dots.

7. **A tempo is recorded when there is evidence, per play.** Design principle
   T16, applied one level down: a play records `achieved_tempo` when the
   musician moved the stepper or the click was sounding, and never from a
   default. `click_pattern` rides with it, per play, for the same reason it
   rides with the entry today (#1499).

8. **Old history is read, not migrated.** Saved sessions store entries as a
   JSON column written and read by the shell, so the fold lives in Swift, in
   `LibraryStore.decodeEntries`: a row with no `plays` folds its legacy
   per-entry fields into one synthesised play, and a skipped row folds into
   none. Rust never decodes that JSON, so a serde shim there would be dead
   code that passed its own tests and never ran. Nothing on device is
   rewritten and no GRDB migration lands. The crash-recovery blob
   is positional bincode, where a removed field cannot be defaulted, so
   `Store.sessionInProgressKey` bumps to `v3` **in the same PR as the shape
   change**: one resume prompt is lost across the upgrade, as with `group_id`
   and `variant_id` before it (#1116).

9. **One lossy projection, named.** Where a single number per item per session
   is genuinely needed (the item's score history, the session's overall
   score), the ViewModel projects `SetlistEntryView.score_summary` as the
   arithmetic mean of the plays that carry a score, rounded to nearest, and
   `None` when none do. Per-variation history reads the plays directly and
   never the summary. This is the only place several marks collapse into one,
   and it is a projection, never stored.

10. **The item-complete sheet is a row per variation, and the mark is the only
    tap.** Time and repetitions are prefilled from what was measured; the
    musician marks each row. With one play it is the sheet that ships today.
    #1738 settles its title against the tone of voice document.

[#1501]: https://github.com/jonyardley/intrada/issues/1501

## Events

| Event | Phase | Change |
|---|---|---|
| `SetEntryVariant { entry_id, variant_id }` | Building | Sets `planned_variation_id`; no longer valid in Active or Summary |
| `SwitchVariation { entry_id, variation_id }` | Active | New. Closes the open play, opens another |
| `RepGotIt` / `RepMissed` | Active | Bank on the open play; no new argument |
| `SetRepTarget { entry_id, target }` | Building | Sets `planned_rep_target`, which seeds every play the entry opens |
| `UpdateEntryScore { entry_id, play_id, score }` | Active, Summary | Gains `play_id` so the sheet can score one row |
| `UpdateEntryTempo { entry_id, play_id, tempo }` | Active, Summary | Gains `play_id`; Phase C narrows it to the manual path (#1761) |
| `SwitchVariation`, `NextItem`, `EndSessionEarly`, `PrepareReflection` | Active | Phase C: each gains `reading: TempoReading`, stamped on the play it closes (#1761) |
| `FinishSession` | Active | Phase C: deleted, since no shell sends it (#1761) |
| `UpdateEntryNotes` | unchanged | Notes stay on the entry: a note is about the item, not one play |

## Validation

- `variation_id` must be a **live** variant of the entry's item; `None` is
  always allowed and means unattributed.
- At most `MAX_PLAYS_PER_ENTRY` (24) plays per entry, bounding the blob on the
  tier where the device is the only copy.
- `play_id` must belong to the named entry.
- Every terminal transition (finishing, ending early, and moving off an item)
  drops any play with no score, no repetitions and under five seconds, so a
  stray tap on the picker does not litter the record. A practised entry always
  keeps at least one play, so decision 3's invariant holds and the shell can
  never hold a `play_id` the core has just deleted.
- A skipped entry keeps only the plays that banked a mark or a repetition, which
  is what freezing rep state on a skip meant before plays existed.
- `SwitchVariation` to the currently open variation writes nothing.

## ViewModel

- `SetlistEntryView` gains `plays: Vec<VariationPlayView>`,
  `score_summary: Option<u8>`, `planned_variation_id` and
  `planned_rep_target`; loses the per-play fields it mirrors today.
- `VariationPlayView` resolves `variation_label` from the library, tombstones
  included, so a session practised on a variation that has since been deleted
  still says what it was.
- `ActiveSessionView` gains `current_variation_id` and
  `current_variation_label`, which is what the player's picker reads.
- `VariantView` loses `is_current`; `is_solid` and `score_history` stay, with
  history derived from plays rather than entries. The Up next card keeps the
  same recommendation by applying the rule itself (`suggestion.rs`) until
  #1501 replaces it.
- `ladder_is_all_keys` is untouched, so the Library row still says Keys when
  every live variation names one (#1464 then makes the exercise detail screen
  agree with it).

## Testing

- **Core, test first:** open and close a play; switch mid item accumulates two
  plays with the right seconds; switch to the same variation is a no-op;
  repetitions and tempo land on the open play; scoring by `play_id`; the
  five-second drop at `FinishSession`; validation rejections (dead variation,
  foreign `play_id`, cap exceeded, `SetEntryVariant` outside Building); a
  piece gets exactly one unattributed play; `score_summary` means correctly
  and is `None` when no play carries a mark; per-variation history reads plays;
  a skip drops the play that recorded nothing and keeps one that banked
  repetitions, which is what freezing rep state on skip meant before plays.
- **The decode shim, in Swift:** a legacy entry JSON with `score`, `variantId`,
  `repCount`, `achievedTempo` and no `plays` decodes to one play carrying all
  four; a skipped legacy row decodes to none; and an entry with `plays`
  ignores the legacy keys. Table test from strings a shipped build actually
  wrote, not ones invented here (#1256).
- **Bridge (#846):** `assert_round_trips` for `VariationPlay` in
  `SetlistEntry` in `SaveSession`, `SwitchVariation`, the two `play_id`
  events and `VariationPlayView`; plus a `LiveBridge` Swift test driving
  start, switch, score, finish through real bincode.
- **iOS:** a populated pre-change session row loads with its play intact; the
  `v3` key means a `v2` blob is ignored rather than misread; the player's
  picker and the sheet's rows under Dynamic Type.

## Phases

- **Phase A, core.** This spec as the first commit, then the types, events,
  validation, derivation and ViewModel projections, plus the
  `sessionInProgressKey` bump. Because the fold lives in the shell and the
  removed fields are read by screens, Phase A also carries the Swift
  persistence codec and the mechanical read-site edits that keep the app
  compiling. No new UI. One PR,
  reviewed before Phase B starts.
- **Phase B, screens.** The player's variation picker, the item-complete
  sheet's rows (the picker it replaces left the sheet in Phase A, since
  decision 5 takes `SetEntryVariant` out of Active), the Progress screen's
  per-variation surface, and the #1733 and #1735 renames folded in. Same
  working session as Phase A (#1374).
- **Phase C, the unblocked, and what Phase B left.** #1478 (a variation typed
  "E flat major" reads as a key) and #1464 (the exercise detail screen says
  Steps), plus the two Phase B found on the same surface: #1761 (a tempo
  reached is credited to the last variation rather than the one that earned
  it, which wants the core stamping a tempo as it closes a play, so this part
  goes core first) and #1758 (an exercise with variations opens on none of
  them, so the seconds before the first pick become a row nobody played).
  #1501 and #1107 follow outside this milestone.

### Phase C: a tempo is stamped as its play closes (#1761)

Decided 14 September. Decision 7 made a tempo a fact about one play, but the
evidence reached the core only once, from the item-complete sheet, which
credited the whole item's click to the last play it kept. Four events close a
play that can carry a tempo: `SwitchVariation`, `NextItem` (both branches; its
last item closes through `transition_to_summary`), `EndSessionEarly` and
`PrepareReflection`'s provisional close. Each now carries what the click was
doing at that instant, and the core stamps the closing play from it.
`SkipItem` closes a play too; rule 7 says why it carries nothing.

```rust
/// What the click was doing at the instant a play closed. Facts only: the
/// core rules on what they evidence (T16).
pub struct TempoReading {
    pub bpm: u16,                  // as displayed, in click.metre.unit; crotchets when click is None
    pub click_sounding: bool,
    pub click: Option<ClickState>,
}
```

1. **The payload.** Each of the four events gains `reading: TempoReading`,
   appended after its existing fields. Not an `Option`: the player always has
   a click, and a silent one is `click_sounding: false`, so no sender can
   leave out what it saw. `RepGotIt`, `RepMissed`, `SkipItem` and
   `RecoverSession` gain nothing. `SwitchVariation` to the open variation
   stays a no-op and ignores its reading.

2. **The stamp.** `close_open_play(entry, now, reading)` writes the open play's
   `achieved_tempo` (normalised through `Metre::crotchet_bpm`, as today) and
   `click_pattern` only when `click_sounding` is true. A silent reading writes
   nothing and clears nothing, so a close never erases a stamp. A sounding
   reading on a play already stamped overwrites it: the later instant wins. A
   reading that fails `validate_click_state` or `validate_achieved_tempo`
   stamps nothing and raises no error, since a close is not a user action
   (#944), and the close itself still happens.

3. **A stamp is not a record.** `recorded_something` and `is_incidental` do
   not read the tempo. A stray tap on the picker while the click sounds is
   still dropped at the terminal transition.

4. **`PrepareReflection` carries its own reading, and the terminal event
   repeats it.** The sheet reads the stamped plays straight after
   `PrepareReflection` to prefill its rows, and the shell stops the click
   before the sheet appears, so a reading taken at the terminal event would
   always be silent. The shell holds the reading on `ReflectionTarget` beside
   `now` and sends both again on the `NextItem` that follows. Under rule 2 the
   second stamp is idempotent, and the terminal close stays correct on its own
   if a later path reaches it without a sheet.

5. **`UpdateEntryTempo` survives, narrowed to the manual path.** T16 accepts
   two kinds of evidence and only the click's moves to the close: a musician
   who played without the click can give a tempo only by setting it on the
   sheet. The event becomes `UpdateEntryTempo { entry_id, play_id,
   tempo: Option<u16>, user_set: bool, click: Option<ClickState> }`.
   `observed` shrinks to `user_set`, and `TempoObservation` goes, having no
   other reader: a `click_sounding` on the sheet's write is exactly the path
   that credited the whole item's click to the last row, and the click's
   evidence has already landed by then. `click` stays, now meaning only the
   metre the number was counted in, so the core still normalises a quaver
   stepper (#1499); it is never stored, because a close is the only writer of
   `click_pattern`. `Some` with `user_set: false` is a prefill nobody touched
   and writes nothing; `Some` with `user_set: true` writes `achieved_tempo`
   and keeps any `click_pattern` the close stamped; `None` clears both, as
   today.

6. **The sheet's rows.** A stamped row's stepper counts in the unit of the
   click that stamped it and prefills from the stamp; an unstamped row counts
   in the click's current unit and prefills as the sheet does today, recording
   nothing unless moved. Either way the row sends the metre it counted in as
   `click`. `VariationPlayView` gains `tempo_display: Option<u16>`, the stamp
   expressed in its own click's unit, so the shell never converts a tempo.
   The round trip through crotchets can move a quaver prefill by one, which
   records nothing until the musician moves it anyway. The shell sends
   `UpdateEntryTempo` for every row with that row's own `user_set`, and the
   core ignores the untouched ones.

7. **A skip stamps nothing and keeps no tempo.** Only a completed entry has
   ever carried a tempo (`UpdateEntryTempo` refuses any other), and the tempo
   history reads plays without looking at the entry's status. `SkipItem`
   therefore carries no reading and clears `achieved_tempo` and
   `click_pattern` on the plays it keeps, including any an earlier switch
   stamped. The repetitions they banked stay, as they do today.

8. **`FinishSession` goes.** No shell sends it: `NextItem` on the last item is
   how a session finishes, and only core tests reach `FinishSession`. Giving
   it a reading would widen an event nobody sends, so the core PR deletes it
   (#1176) and moves its tests onto `NextItem`.

**No blob bump.** `VariationPlay` keeps the fields it has and events are never
persisted, so `Store.sessionInProgressKey` stays at `v4` and
`active_session_blob_wire_is_pinned` does not move. A stamp made at a switch
survives a crash in the blob with its play.

**Known limits, accepted.** Each is how the item's single reading behaves
today, moved to the play:

- A click stopped a moment before a switch or the hand-off leaves that play
  unstamped; the row shows it unmeasured and the musician sets it.
- A click started a moment before a switch stamps a play that ran silent until
  then.
- `PrepareReflection` does not save the session in progress, so a crash with
  the sheet up loses its stamp along with its seconds.

**Rejected: a live click event.** Sending the click's state to the core on
every start, stop and step would stamp a play whose click stopped before it
closed. It needs the core to hold the click's current state and hand it to the
play a switch opens, and one missed stop (the engine dying on a route change,
the background timer) then credits a tempo to every play after it, silently. A
reading at the closing instant misjudges only that instant, and the sheet
shows the musician the result.

**Tests, core first.** A switch with the click sounding stamps the play it
closes and not the one it opens. The issue's case, six minutes of C at 108 and
then thirty seconds of G, ends with C at 108, and G at 108 only because the
click was still sounding at the hand-off. A silent close after a stamp leaves
the stamp. `PrepareReflection` then `NextItem` with the same reading stamps
once. A quaver reading normalises, and `tempo_display` gives it back in
quavers. An invalid reading stamps nothing and still closes. A stamped stray
tap is still dropped. A skip clears the stamp on a play it keeps, including one
a switch stamped, and keeps its repetitions. `UpdateEntryTempo` with
`user_set: false` leaves a stamp alone, with `true` overwrites it, keeps the
pattern and normalises against `click`, and with `None` clears both. The
existing `TempoObservation` fixtures in `session.rs` and its bridge round trip
move to the new shape, not deleted. Bridge: `assert_round_trips` for each of
the four events carrying a `TempoReading` with `click: Some`, and for the
narrowed `UpdateEntryTempo`.

**Two PRs, one working session.**

- **Core.** `TempoReading`, the four payloads, the stamp, the skip's clearing,
  the narrowed `UpdateEntryTempo`, `tempo_display`, the removal of
  `FinishSession`, the tests above and regenerated bindings. It also carries
  the Swift call sites, so the app records the click's tempo on the right play
  through this PR alone: `FocusPlayerScreen` sends a real reading on switch,
  ending early and the hand-off (read before `click.stop()`) and holds it on
  `ReflectionTarget`; the sheet's single stepper still counts in the click's
  unit and writes to the last kept row with `user_set` and `click`. A
  `LiveBridge` test drives start, a switch with the click sounding and the
  hand-off through real bincode, and reads both rows' tempos.
- **Screens.** The item-complete sheet's tempo per row (rule 6), its
  snapshots under Dynamic Type, and the `FocusPlayerScreen` comment that still
  names #1761 as open.

## Open questions

- Whether `is_solid` should decay with time, and whether `SOLID_SCORE_MIN`
  should ever be tunable. Out of scope until the scheduler epic, as before.
- Whether a play should carry its own note. Not in v1: the entry's note covers
  the item, and per-play notes turn the sheet into the form decision 10 exists
  to avoid.
- ~~What the Progress screen shows for an exercise with twenty variations.~~
  Settled in Phase B: coverage, not rings. A row per practised exercise saying
  how many of its variations are solid, over a segment bar, which reads the
  same at three variations or twenty; the per-variation detail stays on the
  exercise's own screen. Five rows, most recently practised first. Mockups:
  [`exercise-variations/design/`](exercise-variations/design/).
