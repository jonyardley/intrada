# Simpler practice: removing focus, repetitions and the Add detail choices

> Tier 3 (store migration, crash recovery blob, practice defaults blob,
> bridge reshape). Rides as the first commit of the PR for #2791. Both apps.
> Decisions taken by the owner on 10 November 2026, in the plan on #2791.

## Problem

Build session asks for a focus and a repetition target on each item, the
practice screen counts plays, and Session complete asks whether the aim was
met, offers chips read from the note and a folded "Add detail" with set
choices for how it felt and what got in the way. The owner ruled that all of
these add weight without paying for it. The aim before playing and the note after
stay, as plain free text.

## What goes

From the core, both apps and saved data:

- **Focus**: `IntentionFocus`, `FocusKind`, `SetFocus`, the focus choices and
  the focus read from a typed aim ("at 84").
- **Repetitions**: the target on a plan and on a play, the count, the tap
  history, `SetRepTarget`, `RepGotIt`, `RepMissed`, `RepUndo`, the counter's
  slots, the past-target count and undo, and the default target.
- **"Aim met?"** (Yes, Partly, Not yet): `IntentionMet`, `AnswerIntention` and
  the answer worked out from the plays (decision 2).
- **Note chips** (bars, tempo, repetitions): `NotePoint`, `NoteSpan`,
  `ConfirmNotePoint` and the note reader in `note_patterns.rs`, whose only
  reader was the chips (decision 3).
- **Add detail**: `Felt`, `Obstacle`, `SetFelt`, `ToggleObstacle`.

## What stays

- The aim (`PlannedItem.intention`) and the note (`SetlistEntry.notes`), free text.
- Marks, the tempo stamped as a play closes, where the tempo rested
  (`tempo_changes`, still capped at 500, under its own name), time away, keys
  and variations.
- A play nobody practised is still dropped: it now means no mark and under
  five seconds, since there is no count to keep it.

## Saved data

### The crash recovery blob

`ActiveSession::BLOB_VERSION` goes from 10 to 11 before any shape changes
(#1345), and the pin is re-taken. A practice left open across the update is
retired with the notice the app already shows. The pinned fixtures fill every
play optional and list that remains (tempo changes, time away, click
pattern, key) so the pin still tells shapes apart.

### The practice defaults blob (decision 4)

`PracticeDefaults` loses `rep_target` and keeps the click start and session
length. `BLOB_VERSION` goes from 2 to 3.

The musician's choices carry across once:

- The core keeps the version 2 shape privately and exports its number
  (`practice_defaults_previous_blob_version`), so neither shell names a version.
- At launch, when the version 3 slot is empty and the version 2 slot holds
  bytes, the shell sends those bytes in a new event,
  `PracticeDefaultsEvent::Carried(Vec<u8>)`, then clears the version 2 slot.
- The core decodes the version 2 shape, clamps the session length as `Loaded`
  does, sets the model and emits `SavePracticeDefaults`, so the version 3 blob
  is written through the one write path. Bytes that will not decode write
  nothing and raise nothing the musician could act on: the defaults stay as
  they start.
- `Loaded` is unchanged and still never re-saves.

The test reads the two version 2 blobs pinned before #2791
(`0501000000011900`, `0a0000000000`) as real device bytes, and refuses both
version 3 pins sent by mistake.

### Saved sessions (store step v23)

One step, `v23_remove_focus_and_repetitions`, JSON surgery in SQL on the
session table's `entries`, never through the codec (#1947), on the pattern of
v22. It removes, by path rather than bare key name:

- on each entry: `focus`, `plannedRepTarget`, `intentionMet`, `felt`,
  `gotInTheWay`, `notePoints`, and the legacy entry level `repTarget`,
  `repCount`, `repTargetReached` and `repHistory` (rows from before #1739);
- on each play: `repTarget`, `repCount`, `repTargetReached`, `repHistory`.

Every other value stays byte for byte, `updated_at` included. A row whose
entries are not a JSON array is named and skipped, as v22 does. The test
asserts on the raw `entries` text, since the new reader ignores unknown keys
and a test through the codec would pass with the step deleted. Decoys of the
same names inside values that stay must survive it.

**Old rows with banked repetitions and no mark.** The reader folds an entry
from before #1739 into one play when it was completed, or when it carried a
mark or banked repetitions. With repetitions gone, a skipped or unreached
entry with repetitions and no mark now reads with no play. Such an entry was
stored with no time (skipped and unreached entries keep none), so the record
loses nothing the app still shows.

A device on an older build syncing a row back would bring the keys with it;
the reader ignores them and the next save drops them.

## Bridge

Removing event variants and fields reorders the positional wire, which is
safe because both apps regenerate from the same core in this PR. The
reshaped types get `assert_round_trips` in Rust, `SessionBridgeTests` on iOS
and `BridgeRoundTripTest` on Android.

## Screens

- **Build session and customise item**: no focus row or choices, no
  repetitions control; the aim stays as a text field.
- **Practice screen**: no repetition counter, past-target count or undo.
- **Session complete**: no "Aim met?", no note chips, no "Add detail".
- **Settings**: Practice defaults keeps the click start and session length,
  without the repetition target.

No mockup: the screens only lose controls. Snapshots are re-recorded on both
apps and orphaned references deleted.

## Tests

- Store: a database at v22 holding every removed key, at entry and play
  level, plus a row from before #1739 and an unreadable row, upgrades with
  the raw remaining text unchanged. Deleting the v23 step fails it.
- Practice defaults: both version 2 blobs carry into version 3 and emit the
  save; garbage bytes write nothing; the version 3 wire is pinned.
- Crash recovery: the version 11 pin, with the fuller fixture.
- Bridge round trips for the reshaped events and views on all three sides.
- Tests of removed behaviour are deleted, not repaired.

## Out of scope

The time split (#2790), the confidence control (#2793, #2794), the countdown
clock (#2789), and renaming code identifiers such as the "reflection" sheet.
