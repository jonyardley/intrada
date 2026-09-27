# The item-complete sheet's answers survive a crash

> Tier 3 (what the crash-recovery copy of a running practice holds). Changes
> the `ActiveSession` blob (version 4 to 5), adds one event and one view field,
> so it ships as two PRs: core, then screens. Issue #2137, split out of #2061.

## Problem

When a piece finishes, the sheet asking how it went opens. The musician marks
each play, may set a tempo by hand, and types a note. None of that reaches the
core until Next: the shell sends the note, then `NextItem`, then the marks
and tempos. If the app dies while the sheet is open, resume brings the
practice back on the running player with the time and the click's tempo for
that piece (#2061), but the marks, tempos and note are gone, with no message.

## Rejected: the core accepts marks before Next

The first route in #2137 sends each answer to the core as it is written. It
fails on a fact about plays: a play has no closed state. The last play is
always the open one, and `PrepareReflection` only stamps its seconds. So the
core cannot tell "at the sheet" from "still playing":

- accepting a mark early means dropping the Completed guard for the piece
  being played, not just the one on the sheet;
- Skip on the sheet is `NextItem` with nothing else, so marks written early
  would survive a Skip that should discard them;
- resume still lands on a running timer with the sheet gone, and the time
  between resume and the second tap on Done reads as practice.

## Approach: the sheet's draft lives in the saved copy

`ActiveSession` gains `reflection: Option<ReflectionDraft>`. `Some` means the
sheet is open on the current entry; `None` means it is not.

```rust
pub struct ReflectionDraft {
    pub now: DateTime<Utc>,       // the stamp's instant
    pub reading: TempoReading,    // the click at the stamp
    pub answers: ReflectionAnswers,
}
pub struct ReflectionAnswers {
    pub marks: Vec<DraftMark>,    // { play_id, score }
    pub note: String,
    pub tempos: Vec<DraftTempo>,  // { play_id, tempo, click }, set by hand only
}
```

The rules:

1. **`PrepareReflection`** stamps the open play as today and opens an empty
   draft holding its `now` and `reading`, then saves. With a draft already
   open it does nothing: the stamp is final, and a second one would count
   the sheet's dwell as practice.
2. **`UpdateReflectionDraft { answers }`** replaces the answers wholesale and
   saves. It is refused without a draft, and refused whole when a mark is
   out of range, a play is not on the current entry, the note is too long or
   a tempo does not validate. Like `PrepareReflection` it neither raises nor
   clears `last_error` (#944): the sheet shows the refusal on Next.
3. **`NextItem` with a draft open** closes the entry with the draft's `now`,
   not the event's. The two are equal before a crash; after one the shell
   no longer has it. The reading stays the event's, so a later sounding
   close still wins whole (#1761); after a resume the shell's click is
   stopped, and a silent reading writes nothing, so the stamp stands.
4. **Leaving the entry** (`NextItem`, `SkipItem`) drops the draft. Terminal
   transitions drop the whole `ActiveSession`. The draft never writes a mark,
   tempo or note itself: the shell still sends them around `NextItem`
   exactly as today, so Skip discards the draft by sending nothing.
5. **`RecoverSession` with a draft** moves the draft's `now` to the resume
   instant. Recovery already backdates the item and play clocks by what the
   plays recorded, so closing at the resume instant records exactly the
   stamped seconds, and the downtime is never practice.

## View

`ActiveSessionView` gains `reflection: Option<ReflectionView>` carrying the
answers and the reading. The screens PR presents the sheet whenever it is
`Some`, rather than from the player's own state, seeds the sheet's answers from
it, and seeds unstamped tempo rows from the reading's click. The sheet closes
when the core drops the draft, which is when `NextItem` lands; a refused note
or move leaves it open with its answers, as today.

## Saving

Each `UpdateReflectionDraft` saves the whole blob to UserDefaults. The screens
PR sends one at once for a mark or tempo, and one for the note after a short
pause in typing and when the app goes to the background, so a paragraph is a
handful of writes, not one a keystroke.

## Blob version

The blob is positional bincode, so a new field is a new shape: `BLOB_VERSION`
goes from 4 to 5, the retired maximum from 3 to 4, and the wire pin in
`session/tests.rs` is re-pinned. The shell's key follows the version (#1116).
A practice in progress when the update installs is not resumed; that is the
cost every blob change pays, and there is no migration.

## Tests (core PR, written first)

- The draft round-trips the bincode wire and the pinned blob hex.
- `PrepareReflection` opens a draft; a second one leaves the draft and the
  stamped seconds alone.
- `UpdateReflectionDraft` replaces the answers and saves; it is refused
  without a draft, for an unknown play, an out-of-range mark and an
  over-long note.
- `NextItem` and `SkipItem` drop the draft; `NextItem` uses the draft's
  instant over the event's, and still the event's reading.
- Recovery keeps the draft and moves its instant, and a `NextItem` after it
  records the stamped seconds, not the downtime. Deleting the draft from
  recovery breaks this test.
- The view carries the draft's answers and reading, and `None` without one.

## Out of scope

- The summary screen's notes, which are already in the core.
- Migrating version 4 blobs.
- A mark written before the sheet opens.
