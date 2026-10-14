# Chart-to-scaffold generation

> Tier 3. Parent [#1098]. Phases A (#1109), B (#1106) and C (#1107, less the
> twelve-key ladder) shipped. What is left is the twelve-key step ladder
> (#1107) and the chord chart work in epic #2025. **Scope: `intrada-core` and
> native iOS.** Design: [`design/`](chart-to-scaffold/design/).
>
> Headline promise: **"add a standard, receive its curriculum."** Deterministic
> music theory in the Rust core, no ML, fully offline.

[#1098]: https://github.com/jonyardley/intrada/issues/1098

## Goal

The musician enters or pastes a standard's chord changes onto a piece. The core
derives a **previewed** set of scaffold exercises, key-aware and each with a
theory rationale, which the musician confirms to create them as ordinary
exercises linked to the piece.

## Non-goals

- **No melodies.** We ship *changes, not tunes*: chord progressions are not
  meaningfully copyrightable (the iReal Pro model); Real Book melodies are. The
  "learn the melody" step is generated as a **titled placeholder exercise with
  no notated content**.
- **The LLM never decides.** Deterministic theory in Rust, testable to the note.
- **No silent generation.** Derivation always produces a **preview** the
  musician reviews and commits.
- **No chart playback, re-harmonisation or analysis view.**

## Key decisions

1. **The chart lives on the piece.** `Item` gains `chord_chart:
   Option<ChordChart>` (`#[serde(default)]`), riding the piece's `updated_at`
   and `deleted_at`. No separate entity, no new table.
2. **A typed, validated chord model, not free text.** `ChordChart` is a
   structured sequence (sections, bars, chord symbols), parsed *once* on entry
   from a text chart into typed `ChordSymbol { root, quality, extensions,
   bass }`. Parsing and spelling is a pure core function with an exhaustive
   test vector. Unparsable tokens are surfaced, not swallowed.
3. **Generation is a pure function of (chart, key) to `Vec<ScaffoldSpec>`.** No
   I/O, no model mutation; a `ScaffoldSpec` is a *proposed* exercise. The
   commit step turns confirmed specs into real `Item`s.
4. **Key-awareness reuses the step and variation mechanism, it does not
   reinvent it.** Each exercise is derived in the chart's key; "in all twelve
   keys" is the exercise's ladder, not twelve separate exercises.
5. **Exercises link through the existing mechanism**, so per-piece tracking
   comes free.
6. **Reuse over duplication on commit.** Re-running derivation after a chart
   edit **reconciles** against the piece's linked exercises rather than
   duplicating, and never overwrites a hand-made exercise.

## The parse rules

- The entry format is a minimal bar-and-pipe grammar we control (Resolved 3).
- `parse_chart` reads sections, bars between `|` marks and chord symbols.
  #1935 sends an unlabelled, non-empty line without `|` through the same bar
  path, so prose reaches `parse_chord` and fails with the existing error.
- Parse errors set `last_error` with the first bad token and its position,
  never a partial chart. The host must be a piece and the chart non-empty.
- `ChordSymbol` keeps `raw: String`, so an unparsed or edited token
  round-trips losslessly.

## What gets generated

| Exercise | Derivation rule |
|----------|-----------------|
| **Learn the melody** | Placeholder only, no notated content (copyright) |
| **Shells (each inversion)** | 3rd and 7th of each chord; inversions are the ladder |
| **Guide-tone lines** | 3rds to 7ths (and back) across adjacent changes by nearest voice leading |
| **Scales to chord tones** | Chord-scale per change, ii-V-I aware; the target degree is the ladder |
| **Constrained improv** | Chord tones of each change, then rhythm only, as one ladder |

Chord-scale selection is rule-based, with ii-V-I and tritone-sub recognition
as bounded special cases. The common vocabulary is covered (maj7, 7, m7, m7♭5,
dim7, m(maj7), 6, altered dominant, sus4, sus2, aug, 7♯5); anything outside it
falls back to the chord's arpeggio and is flagged in the preview.

## Commit

`CommitScaffold` carries the ticked `kinds: Vec<ScaffoldKind>`, not spec
content: the core re-derives from the stored chart, so no derived content
crosses the bridge. It creates the exercises and the piece update atomically
through `PersistenceOperation::SaveItems`, resolving one `Ack` or `Failed`; a
failure leaves the preview and nothing half-committed. Dedup is by title, and
the reserved `scaffold:<kind>` tag reconciles by kind after a chart edit,
hidden from the tag vocabulary. Phase B shipped the commit sheet, which offers
the exercises as suggestions with "Add N exercises".

## Resolved (2026-07-17)

1. **Constrained improv** is **one exercise with a constraint ladder**
   (chord tones, then rhythm only).
2. **Generated-from marker** is **a reserved tag**; a first-class field only
   if reconciliation needs it.
3. **Chart entry format** is **a minimal bar-and-pipe grammar we control**.
4. **Melody placeholder**: **generate the titled placeholder**.
5. **Key ladder**: **wait for the step mechanism** rather than ship one key and
   retrofit. #1107 records that the mechanism has since shipped.

## The twelve keys

Still to build (#1107): each generated exercise carries the twelve keys as its
ladder, so "in all twelve keys" is progress through one exercise rather than
twelve separate ones.

## Deferred

- A public-domain melody corpus for pre-1931 tunes (#1098 theme 4).
- MusicXML import; a chart from a photo is phase D of
  [`piece-from-photo.md`](piece-from-photo.md).
- A generated backing track (#1098 theme 5).
