# Add a piece from a photo

> Tier 3 by domain sensitivity: a new `Item` field crosses the FFI bridge, a
> store migration lands, and a new `Effect` family is added. Issues [#1355]
> (photo on an item) and [#1387] part 2 (a chart from a photo). Epics #1972
> and #2025.
>
> **Four phases.** A, B and C shipped (#1443, #1449, #1455, #1457, #1469,
> #1476). **D is unstarted.**

[#1355]: https://github.com/jonyardley/intrada/issues/1355
[#1387]: https://github.com/jonyardley/intrada/issues/1387
[#1442]: https://github.com/jonyardley/intrada/issues/1442
[#1446]: https://github.com/jonyardley/intrada/issues/1446
[#1454]: https://github.com/jonyardley/intrada/issues/1454

## Approach

One capture surface, four fields, and a hard split between what a photo can
reliably give us and what it cannot.

| Field | Source | Reliability |
|---|---|---|
| The photo itself | Nothing to recognise | Certain |
| Title | Largest text near the top of the page | High |
| Composer | A line matching `by` / `Music by` / `Words and Music` | Good |
| Tempo | A `marking` word, or a `bpm` number after an `=` | Medium |
| Chord chart | Depends on what was photographed | **Split, see below** |

**The chart splits in two, and only one half is tractable.**

1. **A text chart** (chords over lyrics, or a typed grid of bars). OCR reads
   this well. Reconstruct lines by y-position and the result is close enough
   to the bar-and-pipe grammar `parse_chart` accepts to hand to the musician
   for a quick edit. This is phase D.
2. **A lead sheet** (chord symbols over a stave). OCR finds `Cmaj7` but not
   the barlines, which are graphics. **Out of scope; a spike, not a phase.**

## Non-goals

- **No melodies.** We read changes, never tunes
  ([`chart-to-scaffold.md`](chart-to-scaffold.md)).
- **No silent write.** Nothing recognised is saved without the musician seeing
  it first.
- **The model never invents.** See decision 5.
- **One photo per item.** The data shape leaves room for more.
- **No sync of the bytes, and no cloud model.**

## Key decisions

1. **The photo is a file; the item carries only its id.** `Item` gains
   `photo_id: Option<String>` (`#[serde(default)]`, appended last). The bytes
   live at `Application Support/photos/<ulid>.jpg` (JPEG, quality 0.8, long
   edge capped at 2048px, open question 6). Blobs in the database fatten every
   row read and make a bad migration unrecoverable.
2. **Soft delete tombstones the row and leaves the file.** Orphan files are
   reaped on a later explicit pass ([#1442]); an orphan file costs disk, an
   eagerly deleted file costs the musician their photo.
3. **A new `Item` field does not touch the crash-recovery blob.**
   `SetlistEntry` denormalises item fields rather than embedding `Item`, so
   `ActiveSession` is unaffected. Re-verify if that ever changes.
4. **Recognition is an `Effect`, not shell logic.** The shell owns the
   frameworks and returns recognised text with geometry. The core decides what
   any of it *means*.
5. **The model may choose, never invent.** Every field the on-device LLM
   suggests must appear as a substring of the OCR lines, or the core discards
   it and falls back to its own heuristic. *Pinned in phase B: the match is
   against all the lines joined, case- and whitespace-insensitively, not per
   line: a name Vision wrapped across two lines must still be accepted. The
   cost is that a suggestion may span a line boundary; that is the right
   trade. Phase C settles what the clamp then claims: a surviving suggestion
   carries the OCR confidence of the weakest line it spanned, not a perfect
   1.0 ([#1454]), so the field with the least evidence behind it is no longer
   the one that looks strongest. The clamp constrains where a suggestion came
   from, never what shape it is in. "Music by Kosma" is on the page, so the
   composer clamp has to strip the credit prefix the same way the heuristic
   does.* A ~3B model asked to extract will sometimes produce a plausible
   composer that is not on the page; this clamp makes that structurally
   impossible and is a pure function, tested in Rust with no device involved.
6. **Heuristics are the mechanism; the LLM is an enhancement.** Foundation
   Models needs iOS 26 and Apple Intelligence hardware. Every phase must be
   fully useful with OCR alone, on every supported device. Phase C adds
   accuracy, never capability.
7. **The review surface is the feature.** Recognition fills a form the
   musician reviews and edits before saving. Low confidence is shown, not
   hidden. That form is `ItemFormScaffold` on the add path, not a confirm
   sheet of its own (open question 2).
8. **Reuse `ChordChartEditSheet`, do not clone it.** Phase D's output is text
   in the existing sheet, with the existing per-token parse errors.

## The contract

`RecognitionOperation::ReadPage { photo_id }` returns a `PageReading`: the
`RecognisedLine`s with normalised geometry and confidence, plus `suggested`
where the on-device model ran. `recognition::read_fields` is the one pure
function that turns it into a `PhotoDraft`, each field carrying its source
and confidence. Every type has a real-bridge round trip (#846).

## Phases

- **A. A photo on a piece.** [#1355]. `photo_id`, the file store, capture,
  display. The shell mints the ulid and writes the bytes, which never cross
  the bridge; a failed write sends no event.
- **B. Read the page into title, composer and tempo.** The
  `RecognitionOperation` effect, OCR in the shell, `read_fields` and its
  heuristics in the core, pre-filling the add form.
- **C. On-device model suggestions.** Foundation Models where available,
  filling `suggested`, clamped by decision 5. A clamped suggestion still wins
  over the heuristic even when it reads weakly, because the model is choosing
  *which* text to use, not whether OCR got it right.
- **D. A chart from a text chart.** Unstarted. Reconstruct the recognised
  lines by geometry into the bar-and-pipe grammar `parse_chart` accepts, and
  drop the result into `ChordChartEditSheet` for editing, with the existing
  per-token parse errors (decision 8). `RecognizeDocumentsRequest` (iOS 26)
  groups text into tables and would help a grid-style chart on the devices
  that have it; verify its availability against the SDK before relying on it.
  The least certain of the four phases.

**Not a phase: lead sheet barline recovery** (#1438). A timeboxed spike, whose
only deliverable is a findings doc saying whether it is worth a phase at all.
Do not start it before D has taught us how people actually photograph their
charts.

## Open questions

1. **What do people actually photograph?** The first was a handwritten Real
   Book page. Still open: how far OCR gets on handwriting.
2. **Confirm sheet or pre-filled form?** **Answered: pre-fill the add form;
   no confirm sheet.** Add only, not Edit. The scan entry point sits above the
   form, which unblocks [#1446]. Still open: how a recognised field is marked,
   whether a low-confidence read is marked differently, and whether the mark
   clears once the musician edits.
3. **Tempo from a note glyph.** **Answered: the number must follow an `=`, and
   the glyph itself is never required.** A bare number near the top is a page
   or bar number at least as often as a tempo.
4. **Where the camera lives.** **Answered: the detail screen, in phase A**;
   the add surface's entry point is [#1446].
5. **Who reaps orphan files.** **Answered: a later explicit pass, [#1442].**
6. **HEIC or JPEG.** **Answered: JPEG, quality 0.8, long edge capped at
   2048px.** Re-encoding also drops the source EXIF, so no photo carries its
   location into the app.

## Deferred

- Sync of the photo bytes.
- Multiple photos per item.
- Handwritten charts.
- MusicXML import.
