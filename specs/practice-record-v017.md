# Practice record v0.17: everything that changes the record

> Tier 3 (the bridge shape and the practice in progress blob). Rides as the
> first commit of the core PR for v0.17.0, step 5 of epic #50: #2249, #2303,
> #2306, #2307, #2308 and #2315. One `ActiveSession::BLOB_VERSION` bump, 6 to
> 7. No GRDB migration: entries are stored as JSON since #2234.

## Problem

v0.16 gave the record its shape: sections, keys and variations on each play.
What the musician meant, how it went beyond the mark, and what got in the way
still live only in free text, or nowhere. v0.17 adds every field that changes
the record in one blob bump, so the practice in progress is lost across an
upgrade once, not six times. The target is `docs/practice-record-target.md`
(sections 1, 6 and 7); the decisions are on #50, 2026-10-03.

The rule under all of it: store events and the musician's own answers, never a
verdict the app works out. "Intention met" is worked out when read where the
play record allows; it is stored only when the musician answers it.

## Order

```text
Wave 1  spec + plans (this)  |  note reader (#2307)  |  spoken note input (#2307, iOS)
Wave 2  one core PR: every field below, the events, the view surface, LiveBridge test
Wave 3  builder stream (#2249 build, #2303, #2315 split)
        live + finish stream (#2249 spots and scoring, #2306, #2307, #2308, #2315 move on)
```

The core PR carries the ViewModel projections both screen streams read, so
wave 3 needs no core change. Wave 3 starts after the Claude Design pass for the
builder, the practice screen and the finish sheet.

## Data shape

On `SetlistEntry`, appended last:

```rust
pub segments: Vec<Segment>,              // replaces planned_section_ids (#2315)
pub focus: Option<IntentionFocus>,       // beside the free-text intention (#2303)
pub intention_met: Option<IntentionMet>, // the answer, only when asked and given
pub felt: Option<Felt>,                  // #2308
pub got_in_the_way: Vec<Obstacle>,       // #2307, from the folded Add detail row
pub note_points: Vec<NotePoint>,         // #2307, confirmed points only

pub struct Segment { pub section_id: String, pub planned_secs: u32 }
pub struct IntentionFocus {
    pub kind: FocusKind,                 // Tempo, CleanReps, FromMemory, Evenness
    pub section_id: Option<String>,      // None: the whole piece
    pub target: Option<u16>,             // bpm for Tempo, count for CleanReps
}
pub enum IntentionMet { Yes, Partly, NotYet }
pub enum Obstacle { Notes, Rhythm, Fingering, Memory, Tone, Tension }
pub struct NotePoint {
    pub kind: NotePointKind,             // as read_note returns it, less Section
    pub section_id: Option<String>,      // the section named before it in the note
    pub span: NoteSpan,                  // { start, end }: bytes of the note as written
}
pub enum Felt { Easy, Effortful, Tense } // from #2308; appended to, never reordered
```

On `Play`: `pub away: Vec<Away>` with
`Away { left_at, back_at: Option, left_out: bool }` (#2306). On
`ActiveSession`: `segment: Option<SegmentClock>`, the running segment's
start, allowance and what Stay has borrowed from the next.

- **Segments replace the planned section.** One planned section is a list of
  one, with `planned_secs` equal to the entry's planned time. A saved session
  from v0.16 reads `planned_section_ids` into segments; the old field is read,
  never written, as `StoredEntry` already does for retired fields.
- **The note is never touched.** Points carry a span into it so a later reader
  (#2316) can re-read old notes and replace derived points without touching
  confirmed ones.
- **Felt** is one optional value. Its choices are decided in the design pass
  (see Open). Tension as felt and tension as what got in the way are both
  kept: one is a state, the other a cause.
- **Away** records leaving and returning on the open play. `left_out` true
  takes the gap off the play's seconds when read; the raw times stay.

## Events

- Builder: `SetSegments { entry_id, segments }` (minutes always sum to the
  entry's planned time: a segment sent at zero shares what the others leave,
  and a new planned time splits evenly again), `SetFocus { entry_id, focus }`,
  `ApplyLastTime { entry_id }`. The suggested focus needs no event: the view
  reads "A1 at 84" from the saved intention. `SetEntryPlan` still plans one
  section, as one segment.
- Practice screen: `AddTroubleSpot { item_id, bars }` (no name, score order,
  moved from #2245), `MoveToNextSegment`, `StayOnSegment`, `WentAway { at }`,
  `CameBack { at }`, `LeaveAwayOut`.
- Finish sheet: `ConfirmNotePoint { entry_id, span }`, `SetFelt`,
  `ToggleObstacle`, `AnswerIntention`, each with the entry id. Their draft
  copies sit in `ReflectionAnswers` for crash recovery; the hand-off sends
  them after `NextItem`, like the marks, so a skipped sheet writes none.
  Order: the note before the confirms (a span is read from the stored note),
  the answer before any tempo edit. A note edit drops the points it no
  longer reads, in the draft and on the entry.

## Rules the core owns

- **Intention met is read, not stored,** when the focus allows: a Tempo focus
  on a section (or the whole piece) is met when a play of that part reached
  the target with the click sounding. Otherwise the finish sheet asks once;
  skipping stores nothing. An answer given is kept and shown over the read.
- **Away under a minute offers nothing.** A screen lock for a few seconds is not
  time away.
- **The segment clock never stops.** When a segment's time is up the view offers
  On to B or Stay on A (two more minutes, taken from B, while B keeps a
  minute); ignoring it changes nothing. The last segment offers nothing.
- **Left-out time leaves the play's seconds** as it closes, and the item's
  clock in the view; the entry's time is the sum of its plays, so a resume
  never takes a gap off twice.
- **Last time is an offer.** Section and variations start empty unless focus or
  segments set them; the chip sets them in one tap.

## Tests

- Blob: a v6 practice in progress is discarded with the one-line notice; a v7
  round-trips through `LiveBridge` (#846), every new field set.
- Storage: a v0.16 saved session with one planned section reads as one segment;
  a v0.17 one round-trips with every field.
- Table tests from notes a musician would type, against the confirm event
  (#1256): "left hand rushed in bar 12, got it at 84" offers bar 12 and 84.
- Each rule above: met read from plays; under a minute offers nothing; leaving
  six minutes out saves the play six minutes shorter; 20 minutes split into A
  and B starts at 10 and 10.

## Open, for the design pass

- The felt choices: Easy, Effortful, Tense stand in, and their stored words
  must be final before the first TestFlight build.
- Where last time's chip sits in the builder.
