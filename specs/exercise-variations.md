# Practice variations, and the record of what you played

> Tier 3 spec, issue [#1739], kept until #1970 closes. It holds the settled
> shape of a play, a variation, a key, a section and an exercise link. Code
> cites "#1739 decision N" and "#1761 rule N" by the numbers below. Earlier
> decisions that still hold are in [`decisions.md`](../docs/decisions.md).
>
> **Language.** On screen a variation is a "variation", never a step (#1733),
> and a repetition is a "repetition", never a pass (#1735).

[#1739]: https://github.com/jonyardley/intrada/issues/1739

## Shape

An entry holds an ordered list of **plays**. Each play says how the item was
played (section, key, variations) and carries the time, repetitions, tempo and
mark. Switching mid item closes the open play and opens another.

```text
Variation { id, label, updated_at, deleted_at }    // shared across the library (#2246)
Item { …, key, keys: Vec<Key>, variation_ids, sections: Vec<ItemSection>,
       exercise_links: Vec<ExerciseLink> }
SetlistEntry { …, planned_key, planned_variation_ids, planned_rep_target,
               plays: Vec<Play> }
Play { id, section_id, key, variation_ids, started_at, seconds, rep_target,
       rep_count, rep_history, tempo_changes, achieved_tempo, click_pattern, score,
       away: Vec<Away> }
```

## Key decisions (#1739)

1. **A variation is unordered.** "What should I practise next" is a
   recommendation, and it is [#1501]'s job.
2. **Plays live inside the entry, one entry per item.** One entry with a list
   of plays keeps "what I set out to do" and "what I did" separable.
3. **Every practised entry has at least one play, including pieces.** An item
   with nothing to choose gets one plain play. An entry never practised has
   zero plays.
4. **The per-play fields leave the entry.** Score, repetitions, tempo and click
   pattern live on the play only. The entry keeps the plan.
5. **The plan stays on the entry.** The Building phase's choice is a plan;
   `StartSession` seeds the first play from it.
6. **Switching is explicit, never inferred.** `SwitchPlay` (Active only) closes
   the open play, stamping its seconds, and opens a new one. Switching to the
   way already open is a no-op.
7. **A tempo is recorded when there is evidence, per play** (T16): when the
   musician moved the stepper or the click was sounding, never from a default.
8. **Old history is read, not migrated.** A saved entry with no `plays` folds
   its legacy per-entry fields into one play when the core reads the stored
   row; a skipped row folds into none. A shape change to the crash-recovery blob
   bumps its key in the same PR.
9. **One lossy projection, named.** `score_summary` is the mean of the plays
   that carry a score, rounded to nearest, `None` when none do. History per
   variation reads the plays, never the summary. It is never stored.
10. **The item-complete sheet is a row per play, and the mark is the only
    tap.** Time and repetitions are prefilled from what was measured.

[#1501]: https://github.com/jonyardley/intrada/issues/1501

## Validation

- A play's variations and section must be live on the entry's item.
- At most `MAX_PLAYS_PER_ENTRY` (24) plays per entry, bounding the blob.
- `play_id` must belong to the named entry.
- Every terminal transition drops a play with no score, no repetitions and
  under five seconds. A practised entry always keeps one play, so the shell can
  never hold a `play_id` the core has just deleted.
- A skipped entry keeps only the plays that banked a mark or a repetition.

## A tempo is stamped as its play closes (#1761)

`SwitchPlay`, `NextItem`, `EndSessionEarly` and `PrepareReflection` each carry
a `TempoReading`: what the click was doing at that instant. The core stamps the
closing play from it.

1. **The payload** is not an `Option`: a silent click is
   `click_sounding: false`.
2. **The stamp** writes `achieved_tempo` and `click_pattern` only when the
   click is sounding; a silent reading clears nothing. An invalid reading
   stamps nothing and still closes.
3. **A stamp is not a record.** A stray tap while the click sounds is still
   dropped.
4. **`PrepareReflection` carries its own reading**, and the shell sends it
   again on the `NextItem` that follows, since the click stops before the
   sheet appears.
5. **`UpdateEntryTempo` survives for the manual path**, with `user_set` and the
   metre the number was counted in. An untouched prefill writes nothing;
   `None` clears both fields.
6. **The sheet's rows** count in the unit of the click that stamped them,
   from `tempo_display`, so the shell never converts a tempo.
7. **A skip stamps nothing and keeps no tempo**; the repetitions stay.
8. **`FinishSession` is gone**; `NextItem` on the last item finishes.

## Keys

A key is one typed value everywhere and keeps its spelling: `Key` in
`domain/key.rs`, a letter, an accidental and an optional mode, with nothing
guessing major (#2106, #2246). An item has its written `key` and the `keys`
chosen for practice; a play's `None` is the written key. Church modes are #830.

## Sections (#2245)

A section is a part of a piece or exercise, or a trouble spot: `ItemSection
{ id, name, bars: Option<BarRange>, kind: Form | TroubleSpot, target_bpm,
position, updated_at, deleted_at }`. How a section is going is worked out from
plays when read, never stored on it.

- **A section needs a name or bars, or both.** A nameless spot is labelled
  "Bars 12 to 14"; the label is derived, so editing the bars keeps it true.
- **Duplicate names are allowed.** Rows reconcile by id only.
- **The target tempo is stored as given.** `None` means the piece's tempo.
- **Removal always tombstones**, so old plays stay resolvable.

`ItemEvent::ChangeSection` changes one section at a time with
`SectionChange::Save`, `Remove` or `Arrange` (#2447); a change that leaves the
list as it was writes nothing. `parse_bar_range` reads what a musician types
("1-16", "bars 5 to 12", "bb. 5-12", any dash) and refuses a reversed range,
bar 0, a half range, two ranges or words around it.

## Exercise links (#2248)

`ExerciseLink { id, exercise_id, section_id, position, updated_at, deleted_at }`
lives on the piece; `section_id: None` is the whole piece.

- **A link is one row per exercise and target**, reconciled on (exercise,
  section), so relinking revives the old row and keeps its id.
- **Whole piece and sections together are allowed**, with no rule between them.
- **The host is a piece.** Unlinking tombstones.

The piece's card changes one exercise at a time with `ChangePieceLink` and
`LinkChange::Set`, `Move` or `Unlink` (#2447); the pickers send
`ChoosePieceExercises` and `ChooseExercisePieces` (#2379), and
`SetExerciseLinks` writes an exercise's whole set in one batch. An identical
set writes nothing, and removing a section tombstones every live link naming it
in the same save.

## Open questions

- Whether a mark should fade with time (#58).
- Whether a play should carry its own note. Not in v1.

Mockups: [`exercise-variations/design/`](exercise-variations/design/), with
sections and keys in [`design/sections/`](exercise-variations/design/sections/).
