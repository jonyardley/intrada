# Add a piece from a photo

> Tier 3 by domain sensitivity: a new `Item` field crosses the FFI bridge, a
> new GRDB migration lands on the only copy of the user's data, and a new
> `Effect` family is added. Issues [#1355] (photo on an item) and [#1387]
> part 2 (a chart from a photo); [#1390] (one-pass create) is adjacent and
> should land after, or reuse, phase A's capture surface. Roadmap: Layer 1
> (Capture), `pillar:plan`. Graduates "photo-of-your-own-chart OMR" from
> [#1098] theme 4.
>
> **Four phases, each its own issue and PR pair.** A is useful alone and does
> no recognition at all. Design conversation (Claude Design) before A's
> screens PR and again before B's confirm sheet. **Scope: `intrada-core` +
> native iOS only.** API/Turso out of scope.
>
> Status: **not started.** This doc is the design, written 2026-08-29 from a
> feasibility review of the iOS on-device stack (appendix).

[#1098]: https://github.com/jonyardley/intrada/issues/1098
[#1355]: https://github.com/jonyardley/intrada/issues/1355
[#1387]: https://github.com/jonyardley/intrada/issues/1387
[#1390]: https://github.com/jonyardley/intrada/issues/1390

## Problem

Adding a piece is typing. Title, composer, key, tempo, then a separate trip to
the detail screen to type the changes into a syntax the user has to learn
(`ChordChartEditSheet`, and the friction filed as [#1387] part 1). Meanwhile
the thing being copied from is sitting on the music stand, already carrying
every one of those fields in print.

The user's own instinct is to photograph it. Today they cannot: there is no
camera, no photo storage and no image handling anywhere in the shell.

## Approach

One capture surface, four fields, and a hard split between what a photo can
reliably give us and what it cannot.

| Field | Source | Reliability |
|---|---|---|
| The photo itself | Nothing to recognise | Certain |
| Title | Largest text near the top of the page | High |
| Composer | A line matching `by` / `Music by` / `Words and Music` | Good |
| Tempo | A `marking` word, or a `bpm` number near a note glyph | Medium |
| Chord chart | Depends entirely on what was photographed | **Split, see below** |

**The chart splits in two, and only one half is tractable.**

1. **A text chart** (chords over lyrics, or a typed grid of bars). Vision OCR
   reads this well. Reconstruct lines by y-position and the result is close
   enough to the bar-and-pipe grammar `parse_chart` already accepts
   ([`chart.rs:147`](../crates/intrada-core/src/domain/chart.rs)) to hand to
   the user for a quick edit. This is phase D.
2. **A lead sheet** (chord symbols floating over a stave). OCR finds `Cmaj7`
   because it is text, and tells us nothing about barlines, repeats, endings or
   sections, because those are graphics. `parse_chart` needs bar boundaries
   above all else, so a bag of chord symbols with coordinates is not a chart.
   Martinez-Sevilla et al. (2025) put it plainly: chord notation is "a score
   component not handled by existing OMR systems", against a corpus of 293
   handwritten sheets. There is nothing to drop in. **Out of scope; a spike,
   not a phase.**

Recovering barlines geometrically (they are long vertical strokes) is possible
in principle and is real research. We are not betting the feature on it.

## Non-goals

- **No melodies, again.** Same line `specs/chart-to-scaffold.md` already draws:
  we read changes, never tunes.
- **No silent write.** Nothing recognised is ever saved without the user seeing
  it first, for the same reason the scaffold preview exists. A wrong composer
  written silently is worse than no composer.
- **The model never invents.** See decision 5. This is a hard, testable rule,
  not a hope.
- **No photo management.** Exactly one photo per item, per [#1355] iteration 1.
  Multiple photos, ordering and versions are later, and the data shape leaves
  room for them.
- **No sync.** The photo stays on the device. The row is sync-ready; the bytes
  are not, and that is a deliberate deferral.
- **No cloud model.** Not as a fallback, not as an opt-in. If it does not run
  on the device it does not ship in this feature.

## Key decisions

1. **The photo is a file; the item carries only its id.** `Item` gains
   `photo_id: Option<String>` (`#[serde(default)]`, appended last). The bytes
   live at `Application Support/photos/<ulid>.heic`. Blobs in GRDB fatten every
   row read on a table already loaded whole, and make a bad migration
   unrecoverable on a device that is the only copy of the data.
2. **Soft delete tombstones the row and leaves the file.** Invariant 2 is about
   rows. Orphan files are reaped on a later explicit pass, or never on the free
   tier; an orphan file costs disk, an eagerly deleted file costs the user
   their photo.
3. **A new `Item` field does not touch the crash-recovery blob.**
   `SetlistEntry` denormalises item fields rather than embedding `Item`
   ([`session.rs:56`](../crates/intrada-core/src/domain/session.rs)), so
   `ActiveSession` is unaffected and the #1223 class of trap does not apply
   here. Verified, and worth re-verifying if that ever changes.
4. **Recognition is an `Effect`, not Swift logic.** The shell owns the
   frameworks (VisionKit, Vision, Foundation Models) and returns recognised
   text with geometry. The core decides what any of it *means*. This keeps the
   dumb-pipe rule intact with an on-device model in the loop, and it is the
   same shape [#1098] already proposed for inference generally.
5. **The model may choose, never invent.** Every field the on-device LLM
   suggests must appear as a substring of the OCR lines, or the core discards
   it and falls back to its own heuristic. A ~3B model asked to extract will
   sometimes produce a plausible composer that is not on the page; this clamp
   makes that structurally impossible and is a pure function, tested in Rust
   with no device involved.
6. **Heuristics are the mechanism; the LLM is an enhancement.** The app targets
   iOS 17.0; Foundation Models needs iOS 26 *and* Apple Intelligence hardware.
   Every phase must be fully useful with Vision OCR alone, on every supported
   device. Phase C adds accuracy, never capability.
7. **The confirm sheet is the feature.** Recognition fills a form the user
   reviews and edits before saving. Low confidence is shown, not hidden. This
   is the design-principles "spend friction deliberately" call: one screen of
   friction buys trust in everything behind it.
8. **Reuse `ChordChartEditSheet`, do not clone it.** Phase D's output is text
   in the existing sheet, with the existing per-token parse errors. Consolidate
   before you template.

## The contract

Pin this before either side is written.

```rust
// crates/intrada-core/src/recognition.rs

pub enum RecognitionOperation {
    /// Read a captured page. The shell runs OCR and, where the device allows,
    /// on-device structured extraction; the core owns what the result means.
    ReadPage { photo_id: String },
}

pub enum RecognitionOutput {
    Page(PageReading),
    /// No recognition available on this device. Not an error: the user
    /// types the fields, and the photo is still saved.
    Unsupported,
    Failed,
}

pub struct PageReading {
    /// Recognised lines, reading order, with normalised geometry so the core
    /// can reason about position without ever seeing the image.
    pub lines: Vec<RecognisedLine>,
    /// Populated only where Foundation Models ran. `None` on most devices, and
    /// the core must produce a usable draft regardless (decision 6).
    pub suggested: Option<SuggestedFields>,
}

pub struct RecognisedLine {
    pub text: String,
    /// Normalised 0..1, origin top-left.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}

pub struct SuggestedFields {
    pub title: Option<String>,
    pub composer: Option<String>,
    pub tempo_marking: Option<String>,
    pub bpm: Option<u16>,
    pub chart_text: Option<String>,
}
```

The interpretation is one pure function, and it is where the tests live:

```rust
/// A proposal, exactly as `ScaffoldSpec` is: it becomes an `Item` only when
/// the user confirms. Every field carries where it came from, so the sheet can
/// show a low-confidence read differently from a clean one.
pub struct PhotoDraft {
    pub title: Option<DraftField<String>>,
    pub composer: Option<DraftField<String>>,
    pub tempo: Option<DraftField<Tempo>>,
    pub chart_text: Option<DraftField<String>>,
}

pub struct DraftField<T> {
    pub value: T,
    pub source: DraftSource,
    pub confidence: f32,
}

pub enum DraftSource {
    /// Geometry heuristics in the core. The floor, available everywhere.
    Recognised,
    /// The on-device model chose it and it survived the substring clamp.
    Suggested,
}

/// Deterministic. Prefers `suggested` where every field survives the
/// substring clamp (decision 5), otherwise falls back to geometry heuristics.
pub fn read_fields(page: &PageReading) -> PhotoDraft;
```

New events: `ItemEvent::SetPhoto { id, photo_id }`, `ItemEvent::ClearPhoto { id }`,
and `ItemEvent::ReadPhoto { photo_id }` to drive the effect. GRDB migration
`v16_item_photo` adds a nullable `photo_id TEXT`.

`photo_id` is appended **after `variants`**, the current last field, because
the bincode wire is positional. Every one of these types crosses the bridge, so
each needs a real-bridge round-trip in `assert_round_trips` **before** it is
wired to a screen: a stub-bridge test cannot catch a wire break (#846).

Against the offline-first checklist: recognition runs entirely on-device and
issues no `Http` (invariant 1); the photo row carries the item's `updated_at`
and `deleted_at` and is never hard-deleted (invariant 2); the photo id is a
client-minted ulid (invariant 3); `read_fields` and the clamp are core, not
shell (invariant 4); a failed file write resolves `Failed`, never a faked `Ack`
(invariant 5); all of this is new code and so local-first only (invariant 6).

Info.plist keys go through `ios/project.yml` as `INFOPLIST_KEY_NSCameraUsageDescription`
and `INFOPLIST_KEY_NSPhotoLibraryUsageDescription`; both strings are written
against `docs/tone-of-voice.md`, not Apple's boilerplate.

## Phases

Each phase is its own issue and, where it spans core and screens, two PRs
(core first, reviewed before the screens start). **The day figures are rough**:
they assume the shape of a comparable landed slice and no comparable slice has
used Vision or a camera in this codebase before, so treat phase A's as the only
one worth planning against.

- **A. A photo on a piece.** [#1355]. `photo_id`, migration v16, the file
  store, `VNDocumentCameraViewController` capture plus a library picker,
  display on `LibraryDetailScreen`. **No recognition of any kind.** Useful
  alone: a photo of the page you are practising from earns its place with
  nothing else attached. About 2 days.
- **B. Read the page into title, composer and tempo.** The
  `RecognitionOperation` effect, Vision OCR in the shell, `read_fields` and its
  heuristics in the core, the confirm sheet. 2 to 3 days.
- **C. On-device model suggestions.** Foundation Models behind
  `SystemLanguageModel.default.availability` and `if #available(iOS 26)`,
  filling `suggested`, clamped by decision 5. Same sheet, better answers, no
  new surface. About 2 days.
- **D. A chart from a text chart.** Reconstruct lines by geometry into the
  bar-and-pipe grammar, drop the result into `ChordChartEditSheet` for editing.
  3 to 4 days, and the least certain of the four.

**Not a phase: lead sheet barline recovery.** A timeboxed spike, whose only
deliverable is a findings doc saying whether it is worth a phase at all. Weeks,
and it may land on "no". Do not start it before D has taught us how people
actually photograph their charts.

## Open questions

1. **What do people actually photograph?** The whole design assumes a mix of
   printed text charts and Real Book pages. Phase A ships a photo store, which
   means phase A also gives us the answer before D commits to a parser.
2. **Confirm sheet or pre-filled form?** Whether recognition opens its own
   review screen or simply pre-fills `ItemFormScaffold` with the recognised
   values marked as suggestions. Design conversation, before B's screens PR.
   The second reads as less ceremony and folds neatly into [#1390].
3. **Tempo from a note glyph.** `♩= 120` is common in print and OCR mangles the
   glyph often. Whether a bare number near the top is safe to read as bpm, or
   whether it needs the glyph, is a question for real photographs.
4. **Where the camera lives.** Create form, detail screen, or both. Ties
   directly to [#1390]'s one-pass create.
5. **Who reaps orphan files.** Decision 2 leaves the bytes behind on a
   tombstone, which is right for the user and unbounded for the disk. A
   settings-screen figure with an explicit clear, a reap on a later explicit
   pass, or nothing at all on the free tier. Answer before phase A ships, not
   after.
6. **HEIC or JPEG.** HEIC is materially smaller for the same page and is what
   the camera produces natively; JPEG is the safer thing to hand to a future
   export or sync. Cheap to decide, expensive to change once photos exist on
   devices.

## Deferred

- Sync of the photo bytes. The row is sync-ready from day one; the file is not.
- Multiple photos per item, ordering, replacement history.
- Handwritten charts. OCR accuracy on handwriting is a different problem and a
  different bet.
- MusicXML import, which is the reliable path to a chart and has nothing to do
  with a camera. Worth its own issue.
- Any API or web surface.

## Appendix: what iOS gives us on-device

All free, all offline, no key, no cloud cost.

- **VisionKit `VNDocumentCameraViewController`** (iOS 13+): the scanner UI with
  edge detection and perspective correction. Available on every device we
  target.
- **Vision text recognition** (iOS 13+): on-device, accurate on print, returns
  bounding boxes and per-observation confidence. This is the mechanism.
- **`RecognizeDocumentsRequest`** (iOS 26): groups text into paragraphs, lists
  and tables with rows and columns. Would materially help a grid-style chart in
  phase D, on the devices that have it. Verify the availability annotation
  against the SDK before relying on it.
- **Foundation Models** (iOS 26 and Apple Intelligence hardware, iPhone 15 Pro
  and later): the ~3B on-device model with `@Generable` guided generation into
  a Swift struct. Apple names entity extraction as a strength, which is exactly
  phase C's job. A minority of the installed base, hence decision 6.

Sources: [Read documents using the Vision framework (WWDC25)](https://developer.apple.com/videos/play/wwdc2025/272/),
[Optical Music Recognition of Jazz Lead Sheets](https://arxiv.org/abs/2509.05329),
[Apple on-device foundation model updates](https://machinelearning.apple.com/research/apple-foundation-models-2025-updates).
