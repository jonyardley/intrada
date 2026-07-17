# Chart-to-scaffold generation

> Tier 3 lightweight spec. Status: **Phases A + B + C (less the twelve-key
> ladder) implemented.** A (#1109): parser + derivation +
> `SetChordChart`/`ClearChordChart` + preview ViewModel + GRDB migration + iOS
> chart-entry & read-only preview. B (#1106): `SaveItems` atomic batch
> persistence op + `CommitScaffold` (title-dedup / no clobber) + the selectable
> "Add N" commit sheet. C (#1107): broader chord vocabulary + ii–V–i / tritone-sub
> scale recognition + reserved generated-from tag / regenerate-on-edit reconcile
> — **twelve-key step ladder still deferred, blocks on C ([#1083]).** **Scope:
> `intrada-core` + native iOS only.** Web/API (Turso) out of scope (see Deferred).
> Bet 1 of the on-device AI wow-factor brainstorm ([#1098]). Design mocks:
> [`design/`](chart-to-scaffold/design/) (`chart-entry.dc.html`,
> `scaffold-preview.dc.html`, `DECISIONS.md`; also in Claude Design →
> `mockups/chart-to-scaffold/`).
>
> Headline promise: **"add a standard, receive its curriculum."** Pure
> deterministic music theory in the Rust core — **no ML, fully offline.**

[#1098]: https://github.com/jonyardley/intrada/issues/1098

## Problem

A jazz standard is not practised, it is *built* (VISION.md → Piece Scaffolding;
[journeys.md](../docs/journeys.md) step 2). The scaffold is a known, teachable
sequence: learn the melody, shells in each inversion, guide-tone lines, scales
down to each chord tone on every change, constrained improvisation. Today the
user hand-creates every one of those exercises and links them by hand
([piece-linked-exercises.md](piece-linked-exercises.md)) — tedious, and it
assumes theory knowledge many learners are still acquiring. Yet the whole
curriculum is *derivable from the changes*: given the chord progression, the
exercises and their key-aware content follow by rule.

## Goal

The user enters (or pastes) a standard's chord changes onto a piece. The core
derives a **previewed** set of scaffold exercises — key-aware, each mapped to a
theory rationale — which the user confirms (edit/deselect first) to batch-create
them as real, separately-tracked exercises linked to the piece. The exercises
are ordinary library items from that point on: practised, scored, and reused
exactly like hand-made ones.

## Non-goals

- **No melodies.** We ship *changes, not tunes* — chord progressions are not
  meaningfully copyrightable (the iReal Pro model); Real Book melodies are. The
  "learn the melody" step is generated as a **titled placeholder exercise with
  no notated content** (a public-domain melody corpus is a later, separate bet —
  #1098 theme 4).
- **The LLM never decides.** This bet is deliberately *not* AI — it is
  deterministic theory in Rust, testable to the note. (The AI bets ride later.)
- **No silent generation.** Derivation always produces a **preview** the user
  reviews and commits; we never inject exercises into the library unprompted
  (agency; anti-clutter — [design-principles.md](../docs/design-principles.md)).
- **No chart *playback* / backing track.** Rendering audio from the changes is a
  separate, bigger bet (#1098 theme 5).
- **No re-harmonisation or analysis surface.** We parse the chart to generate;
  we don't offer a chord-analysis view of its own.
- **Online/web out of scope** (see Deferred), same as piece-linked-exercises.

## Key decisions

1. **The chart lives on the piece as an additive, sync-ready field.** `Item`
   gains `chord_chart: Option<ChordChart>` (`#[serde(default)]`). It rides the
   piece's `updated_at`/`deleted_at` — no separate entity, no new table
   (mirrors the `linked_exercise_ids` choice). Whole-field LWW at single-user
   scale is fine.
2. **A typed, validated chord model — not free text.** `ChordChart` is a
   structured sequence (sections → bars → chord symbols with duration), parsed
   *once* on entry from a text chart into typed `ChordSymbol { root, quality,
   extensions, bass }`. Parsing/spelling is a pure core function with an
   exhaustive test vector. Unparsable tokens are surfaced, not swallowed.
3. **Generation is a pure function of (chart, key) → `Vec<ScaffoldSpec>`.** No
   I/O, no model mutation; a `ScaffoldSpec` is a *proposed* exercise (title,
   kind, key, rationale, and the derived per-change content). The commit step
   turns confirmed specs into real `Item`s. This keeps the theory engine a
   unit-testable island and the preview cheap to recompute.
4. **Key-awareness reuses the C step/variant ladder ([#1083]), it does not
   reinvent it.** Each generated exercise is derived in the chart's key; "in all
   twelve keys" is the exercise's *step ladder* (the #1083 keys preset), not
   twelve separate exercises. **This makes #1083 (C) a hard dependency** — see
   Sequencing.
5. **Exercises link via the existing mechanism.** Committed specs push their ids
   onto the piece's `linked_exercise_ids`; per-piece tracking comes free from B1
   ([#1081], merged #1095). Zero new tracking machinery.
6. **Reuse over duplication on commit.** Before creating, match against the
   piece's already-linked exercises by (kind, generated-signature) so
   re-running derivation after a chart edit **reconciles** rather than
   duplicating. The signature is stored on the exercise (a tag or a small
   `generated_from` marker) so a hand-made "Shells" isn't clobbered.

[#1083]: https://github.com/jonyardley/intrada/issues/1083
[#1081]: https://github.com/jonyardley/intrada/issues/1081

## What gets generated

Derived per chart. Each is one `ScaffoldSpec` → one linked exercise, key-aware
(step ladder = keys), scored separately. Names are the user-facing titles.

| Exercise | Derivation rule | Notes |
|----------|-----------------|-------|
| **Learn the melody** | placeholder only | No notated content (copyright). Title + link so tracking works. |
| **Shells (each inversion)** | 3rd + 7th of each chord; inversions = the step ladder | The core voice-leading skeleton. |
| **Guide-tone lines** | connect 3rds→7ths (and 7ths→3rds) across adjacent changes by nearest voice-leading | The line *is* the derived content. |
| **Scales to chord tones** | chord-scale per change (ii–V–I aware where detectable), target = land on 3rd / 5th / 7th / root; target degree = the step ladder | The largest generator; one "run to the Nth" ladder. |
| **Constrained improv** | limited to chord tones of each change; then rhythm-only | Two specs, or one with a constraint step ladder — open question. |

Chord-scale selection is rule-based (chord quality → scale, with ii–V–I and
tritone-sub recognition as bounded special-cases), **not** exhaustive jazz
theory. v1 covers the common vocabulary (maj7, 7, m7, m7♭5, dim7, m(maj7), 6,
alt/altered dominant); anything outside it falls back to the chord's arpeggio
and is flagged in the preview. Ambition is capped deliberately — every extra
chord quality is more test surface on the one code path a wrong note ruins.

## Data model (intrada-core)

- `Item.chord_chart: Option<ChordChart>` (`#[serde(default)]`, pieces only).
- `ChordChart { key: String, modality: Modality, sections: Vec<ChartSection> }`;
  `ChartSection { label: Option<String>, bars: Vec<Bar> }`;
  `Bar { chords: Vec<ChartChord> }`; `ChartChord { symbol: ChordSymbol, beats: u8 }`.
- `ChordSymbol { root: PitchClass, quality: ChordQuality, extensions: Vec<..>,
  bass: Option<PitchClass> }` — the parsed, canonical form; **`raw: String`
  retained** so an unparsed/edited token round-trips losslessly.
- Events:
  - `SetChordChart { piece_id, raw_chart: String }` → parse → store on the piece
    (`updated_at`, persist via the persistence `Effect`); parse errors set
    `last_error`, never a partial chart.
  - `ClearChordChart { piece_id }`.
  - `CommitScaffold { piece_id, specs: Vec<ScaffoldSpec> }` → batch-create the
    confirmed exercises + push ids onto `linked_exercise_ids`, one transaction.
- Derivation: `fn derive_scaffold(&ChordChart) -> Vec<ScaffoldSpec>` (pure).
- Validation (`validation.rs`): host must be a `Piece`; chart non-empty; every
  bar's beats sum to the metre; parse surfaces the first bad token with its
  position. Reuse the piece-vs-exercise guards already there.
- ViewModel: for a piece with a chart, `scaffold_preview:
  Option<ScaffoldPreviewView>` (the derived specs + per-spec rationale +
  already-linked/duplicate flags). Recomputed from the stored chart; the shell
  renders it, never derives.

**FFI hazard (read before adding fields):** `ChordChart`, `ScaffoldSpec`, and
everything they contain cross the bincode bridge. No JSON-only serde attrs; add
each to `assert_round_trips` **before** wiring to a screen (the #846 silent-drop
class — see CLAUDE.md → project-specific gotchas). `CommitScaffold` is a write
payload → real-bridge round-trip test as a build precondition.

## Batch commit / persistence

`CommitScaffold` creates N exercises + one piece update atomically. The reverted
`AddPieceWithScaffold` ([#1080]) had a transactional batch-save effect; it was
removed with the revert (#1092), so this needs a persistence op decision:

- **Option A (recommended):** add `PersistenceOperation::SaveItems(Vec<Item>)`
  resolving one `Ack`/`Failed` — a genuine transaction; the shell wraps GRDB in
  one write. Restores the batch primitive the scaffold (and future importers)
  need.
- **Option B:** issue N `save_item` commands + one for the piece. Simpler, but
  **not atomic** — a mid-batch failure leaves orphaned exercises and a
  half-linked piece, violating invariant 5's "never a silent partial success"
  spirit. Rejected unless A proves heavy.

A local failure resolves `Failed` and the core surfaces it (invariant 5); the
preview stays, nothing is half-committed.

## Offline-first / invariants

- Chart entry, derivation, and commit all go through the persistence `Effect`,
  never HTTP (invariant 1). Derivation is pure and needs no I/O at all.
- Chart rides the piece's `updated_at`/`deleted_at`; generated exercises are
  ordinary sync-ready items with client-minted ulids (invariants 2, 3).
- All theory/derivation/reconciliation is core Rust, shareable to Android
  (invariant 4) — no generation logic in Swift.
- **Invariant 6 (both modes) consciously deferred** for the new handlers, as
  with piece-linked-exercises: implement + test `local_first` now; the online
  branch lands with the Deferred web/API work. New events must still compile
  against existing online plumbing.

## iOS UI (SwiftUI) — sketch, design mock precedes build

- **Chart entry** on the piece `LibraryDetailScreen`: a "Chord chart" card —
  paste/edit a text chart; parse errors shown inline against the offending
  token. Tokens-only styling; no literals.
- **Scaffold preview** sheet: the derived exercises as selectable rows (title +
  key + one-line rationale + a duplicate/"already linked" flag), a fallback
  badge on any spec that hit the arpeggio fallback, and a sticky **"Add N
  exercises"** commit button (mirrors piece-linked-exercises' Picker A). Nothing
  is created until commit; deselect/edit before committing.
- After commit, the exercises appear in the existing **Linked exercises**
  section — no new tracking surface.
- Ships with snapshot tests (empty / parsed-chart / preview / fallback-flagged),
  VoiceOver labels, Dynamic Type, iPad `SplitView`.

## Testing

- **Parser:** an exhaustive symbol test vector (roots incl. enharmonics,
  qualities in scope, slash/bass chords, malformed tokens → surfaced error).
- **Derivation (the crown jewels):** golden-file tests — a known standard's
  changes (a public-domain progression) → expected `ScaffoldSpec`s, asserting
  guide-tone voice-leading and chord-tone targets *to the pitch class*. One
  golden per generator.
- **Handlers (`local_first`):** `SetChordChart` parse+store, `ClearChordChart`,
  `CommitScaffold` creates + links + dedups on re-run (no duplicates).
- **Reconciliation:** re-deriving after a chart edit reuses matching exercises,
  doesn't clobber a hand-made one.
- **Bridge:** `assert_round_trips` extended for `Item` (with `chord_chart`),
  `ScaffoldSpec`, `CommitScaffold`; a real `LiveBridge` round-trip for the
  commit payload.
- **iOS:** GRDB migration upgrade-path test (populated prior-version DB → column
  defaulted, data intact); the snapshot tests above.

## Resolved (2026-07-17)

1. **Constrained improv** = **one exercise with a constraint step ladder**
   (chord-tones → rhythm-only), consistent with #1083's "progress = advance the
   step".
2. **Generated-from marker** = **reserved tag in v1**; promote to a first-class
   `generated_from: Option<String>` field only if reconciliation needs it (saves
   a bridge type + migration up front).
3. **Chart entry format** = **a minimal bar-and-pipe grammar we control** —
   smallest parser, no third-party syntax to chase.
4. **Melody placeholder** = **generate the titled placeholder** — the scaffold's
   first rung shouldn't be missing.
5. **Key ladder** = **block on C ([#1083])** — the key ladder is core to the
   promise; don't ship key-in-one and retrofit.

## Sequencing

- **Hard dependency on C ([#1083], key ladders / steps)** for key-aware content
  as a step ladder rather than twelve exercises. **Resolved: block on C** — the
  key ladder is core to the promise (Resolved #5). Phase A can begin (parser +
  derivation in the chart's key are C-independent); the key-ladder wiring waits
  for C.
- Rides B1 ([#1081], merged) for per-piece tracking — already in place.
- Sits after the current B → R → C run of the lesson-to-mastery epic
  ([#1087]), per #1098's sequencing.

[#1080]: https://github.com/jonyardley/intrada/issues/1080
[#1087]: https://github.com/jonyardley/intrada/issues/1087

## Phasing

Spec rides with **Phase A**'s first commit (CLAUDE.md workflow). Suggested
phases, each its own PR after A:

- **A — chart + parser + derivation + preview (this spec).** `ChordChart` field,
  the parser, `derive_scaffold`, `SetChordChart`/`ClearChordChart`, the preview
  ViewModel, GRDB migration + codec, iOS chart-entry + preview UI (read-only
  preview, **no commit yet**), the parser/derivation golden tests. Shippable as
  "see your standard's curriculum."
- **B — commit (implemented, #1106).** `SaveItems` batch persistence op,
  `CommitScaffold`, dedup / reconciliation, the "Add N exercises" flow. Turns
  preview into real linked exercises. **Payload shape changed from the sketch:**
  `CommitScaffold` carries `kinds: Vec<ScaffoldKind>` (the ticked rows), not
  `Vec<ScaffoldSpec>` — the core re-derives from the stored chart
  (deterministic), so no spec content crosses the bincode bridge and the shell
  can't corrupt derived content (invariant 4). Dedup is by exercise title (the
  same key the preview's `already_linked` flag uses); the reserved generated-from
  tag stays deferred (Resolved #2) — Phase C's regenerate-on-edit is what needs
  it.
- **C — vocabulary + polish.** Broader chord-quality coverage, ii–V–I /
  tritone-sub recognition, fallback-flag UX, regeneration on chart edit.
  Shipped less the twelve-key ladder: sus4/sus2/aug/7♯5 qualities + a
  6/9 slash-guard (shrinking the arpeggio fallback); context-aware chord-scale
  selection (dominant→minor gets altered, tritone-sub gets lydian-dominant);
  the reserved `scaffold:<kind>` generated-from tag so re-deriving after a chart
  edit reconciles by kind (rename-robust) and the tag stays hidden from the tag
  vocabulary. **Twelve-key step ladder stays deferred — it blocks on C
  ([#1083], unstarted); #1107 remains open for it (Resolved #5).**

## Deferred / out of scope (tracked)

- **Web/API (Turso) parity** — chart column, derivation-on-server or
  derive-on-read, online commit branch + "test both modes" (invariant 6). When
  web un-pauses.
- **Public-domain melody corpus** — real "learn the melody" content for pre-1931
  tunes (#1098 theme 4).
- **MusicXML import, then photo-of-chart OMR** (#1098 theme 4, "later").
- **Generated backing track** from the changes (#1098 theme 5).
