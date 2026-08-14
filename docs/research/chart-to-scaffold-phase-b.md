# Research note: chart-to-scaffold Phase B (#1106)

*Stage 4.1 of [`rethink-plan.md`](../rethink-plan.md). Evidence and shape only:
no comparison against the other candidates and no recommendation. Written
2026-08-14.*

## 1. What it is

Phase B of chart-to-scaffold (spec:
[`specs/chart-to-scaffold.md`](../../specs/chart-to-scaffold.md), parent
[#1098](https://github.com/jonyardley/intrada/issues/1098)): turn the
read-only scaffold preview from Phase A into real, committed, linked
exercises. Enter a standard's changes on a piece; the core derives shells,
guide-tone lines, scales-to-chord-tones, constrained improv and a melody
placeholder; the user ticks rows and taps "Add N"; the ticked specs become
ordinary library exercises, linked to the piece, deduplicated on re-run so a
second commit or a hand-made "Shells" is never clobbered.

**The headline factual finding first: Phase B is already shipped and live on
HEAD.** PR [#1110](https://github.com/jonyardley/intrada/pull/1110) merged
2026-07-17, and Phase C (vocabulary, ii–V–I recognition, regenerate-on-edit)
followed the same week in [#1111](https://github.com/jonyardley/intrada/pull/1111).
Issue #1106 was never closed, which is presumably why the rethink plan's
Stage 4 row names it as if outstanding. The candidate this note evaluates is
therefore really "chart-to-scaffold, finished and revived": the genuinely
unshipped remainder (the twelve-key ladder wiring, below) plus making a
feature that landed the day before the coach pivot froze it into something
that gets real use. Section 3 has the code-level evidence.

## 2. The user problem

Jon's own jazz-piano use, and VISION.md's Piece Scaffolding section verbatim:
a standard is not practised, it is built. Melody, shells in each inversion,
guide-tone lines, scales down to each chord tone on every change, constrained
improvisation. Today's alternative is hand-creating five-plus exercises and
linking each one, which is tedious and assumes theory knowledge the learner
may still be acquiring, even though the whole curriculum is derivable by rule
from the changes. This is Capture (Layer 1) work: journey step 2 in
[`journeys.md`](../journeys.md) ("exercises scaffold a piece", marked Built)
with the derivation automating the entry cost. It also touches Layer 5's
promise in miniature, since the rationale strings teach the theory the user
lacks, but deterministically, with no model involved.

## 3. What exists

More than the rethink plan assumed. Verified against HEAD:

- **Phase A (#1109, merged 2026-07-17)** — `ChordChart` on `Item`, the
  bar-and-pipe parser with per-token errors, `derive_scaffold` (pure, golden
  tests to the pitch class), `SetChordChart`/`ClearChordChart`, the preview
  ViewModel, GRDB migration, iOS chart entry (`ChordChartEditSheet`) and the
  preview sheet. `crates/intrada-core/src/domain/chart.rs` is ~1,160 lines
  with 30+ tests; [`docs/rebuild-review.md`](../rebuild-review.md) called it
  "the sleeper asset" during the coach-era assessment.
- **Phase B (#1110, merged 2026-07-17)** — everything issue #1106 asks for is
  in the tree: `PersistenceOperation::SaveItems(Vec<Item>)` as an atomic
  batch upsert (this *was* the restoration of the batch primitive reverted in
  #1092), `ItemEvent::CommitScaffold { piece_id, kinds }` which re-derives
  from the stored chart so no spec content crosses the bridge, dedup against
  already-linked exercises, the selectable `ScaffoldPreviewSheet` with the
  "Add N" commit, six core handler tests and a real-`LiveBridge` round-trip
  for the commit payload.
- **Phase C (#1111, merged 2026-07-17)** — broader chord vocabulary,
  context-aware chord-scale selection (ii–V–I, tritone-sub), and the reserved
  `scaffold:<kind>` tag so re-deriving after a chart edit reconciles by kind,
  rename-robust. The commit handler dedups on kind tag *or* title, so the
  spec's "hand-made Shells isn't clobbered" case is implemented and tested.
- **The steps/variants ladder (#1083 C1–C4, merged #1112/#1118/#1123 and the
  C3/C4 follow-ups)** — the `Variant` model, per-step scores, the Steps UI,
  and the twelve-key presets ("Add 12 major keys") all landed and survived
  the revert. This was the hard dependency the spec blocked the key ladder on.
- **Not shipped: the twelve-key ladder wiring.** `derive_scaffold` still
  derives in the chart's own key only, and `CommitScaffold` creates each
  exercise with `variants: vec![]`. The spec's Resolved #5 called the key
  ladder "core to the promise"; its dependency is now met but the wiring
  never happened, and [#1107](https://github.com/jonyardley/intrada/issues/1107)
  stays open for it.
- **Almost no real use.** The feature merged on 2026-07-17; the coach pivot
  consumed the product from then until the 2026-08-13 revert, and the
  restored surface returned with the builder. Jon's Stage 1 audit walkthrough
  mentions chord charts once, as a parked design topic
  ([`audit-2026-08.md`](../audit-2026-08.md)). The derived curriculum has
  never been practised through critically.

Bookkeeping: #1106 should be closed as shipped and #1107 re-scoped to the
ladder residue whatever Stage 4.2 decides.

## 4. Pedagogy evidence

**The practice-tradition consensus is strong; formal experimental evidence
for this specific scaffold is thin. Method books are cited here as practice
evidence, explicitly in lieu of controlled studies.**

- **The scaffold sequence is the standard jazz pedagogy.** Melody first, then
  roots, then shells (3rd and 7th), then guide-tone lines, then chord-scales
  and arpeggios, then constrained improvisation is how the tradition teaches
  tunes: Levine's [Jazz Theory Book](https://www.amazon.com/Jazz-Theory-Book-Mark-Levine/dp/1883217040)
  (chord-scale theory as the derivation rule), the
  [Aebersold](https://www.jazzbooks.com/) play-along method,
  [Learn Jazz Standards on guide tones](https://www.learnjazzstandards.com/blog/learning-jazz/jazz-theory/use-guide-tones-navigate-chord-changes/)
  and [mapping a standard](https://www.learnjazzstandards.com/ljs-podcast/learn-jazz-standards/ljs-106-mapping-jazz-standard-improv-success/),
  [PianoGroove on shell voicings](https://www.pianogroove.com/jazz-piano-lessons/shell-voicings-for-jazz-piano/),
  [The Jazz Piano Site on guide tones](https://www.thejazzpianosite.com/jazz-piano-lessons/jazz-improvisation/guide-tones/),
  and [melody-first tune learning](https://www.voicelidjazzguitar.com/jazz-guitar-blog/how-to-learn-jazz-standards-melody-first-someday-my-prince).
  Hal Crook's Berklee text
  [Ready, Aim, Improvise!](https://halcrook.com/publications) teaches
  "advancing via restriction", which is exactly the constrained-improv spec.
  Smither's peer-reviewed
  [Guide-Tone Space (MTO 25.2)](https://mtosmt.org/issues/mto.19.25.2/mto.19.25.2.smither.html)
  gives the guide-tone skeleton a formal music-theoretic treatment as an
  improvisational scaffold, about as close to academic validation of the
  practice as the literature gets.
- **Part-whole transfer evidence is mixed, and honesty requires saying so.**
  The Wickens et al. (2013) meta-analysis of
  [part-task training](https://pubmed.ncbi.nlm.nih.gov/23691838/) found
  fragmenting *concurrent* task components produces negative transfer, while
  *segmented* or *simplified* part practice transfers acceptably. A harmonic
  skeleton is a simplification of the whole tune (structure retained,
  fidelity reduced) rather than a fragmentation, which is the favourable
  case, but no study tests shells-then-guide-tones against whole-tune
  practice directly.
- **The repo's own foundation supports the shape.**
  [`research-foundation.md`](../research-foundation.md) already carries: the
  qualitative core of deliberate practice (specific goals, edge of ability,
  feedback; Ericsson 1993 as refined by Macnamara 2014/2019), Duke's teacher
  functions (diagnose, **decompose**, **sequence**, regulate) as what
  self-directed musicians lack, and the guided-instruction literature
  (Kirschner, Sweller & Clark 2006; Alfieri et al. 2011) showing scaffolded
  guidance beats unassisted discovery for novice and intermediate learners.
  Scaffold generation automates decompose and sequence for a learner who
  cannot yet do either, which is squarely the gap §9 of that doc names.
- **Novelty check: no direct competitor does this.**
  [iReal Pro](https://www.irealpro.com/) is charts plus a backing band, the
  commercially proven changes-not-melodies model the spec borrows, but it
  generates no exercises.
  [Mapping Tonal Harmony Pro](https://mdecks.com/mapharmony.phtml) is the
  nearest neighbour (analysis, target notes, twelve-key workouts) but is a
  theory surface, not a practice notebook: nothing is committed, linked or
  tracked. Drill apps ([Chord Trainer](https://jazzchords.app/),
  [251 Chord](https://www.guitarworld.com/news/jazz-guitarist-matt-warnock-launches-251-chord-app-iphone-and-ipad),
  [ToneGym's generator](https://www.tonegym.co/tool/item?id=progression-generator))
  drill generic material, not *your tune*. Deriving a tracked, per-tune
  curriculum inside the practice log appears genuinely uncommon.

**Relation to the coach post-mortem.** The coach failed because the app
decided the *schedule* and gated the practice
([`rebuild-review.md`](../rebuild-review.md)). Chart-to-scaffold generates
*material*, into a preview the user edits and commits (the spec's "no silent
generation" non-goal), and the committed exercises are ordinary items the
user schedules however they like. Agency over what to practise today is
untouched; the distinction is worth keeping explicit in Stage 4.2.

## 5. Shape sketch

The Tier 3 surface (batch persistence op, bridge-crossing event, GRDB
migration) is already built and paid for. If this candidate is picked, the
work is smaller and different from the #1106 text:

- **Slice 1 — use it, then true up the books.** Drive the shipped flow with a
  real standard (chart entry, commit, practise the exercises through the
  builder), file friction as issues, close #1106 as shipped, re-scope #1107
  to the ladder residue. Docs and simulator only; no code required to start.
- **Slice 2 — the twelve-key ladder wiring.** `CommitScaffold` attaches a
  keys ladder (the existing `Variant` mechanism and presets) to the key-aware
  kinds, and derivation becomes ladder-aware per step. Core-only or nearly:
  `Variant` already crosses the bridge, so this is plausibly Tier 2 with the
  domain-sensitivity override, not a new Tier 3.
- **Slice 3 and beyond (the spec's deferred list)** — regeneration UX polish,
  then the bigger bets: a public-domain melody corpus for pre-1931 tunes so
  "Learn the melody" stops being an empty placeholder, MusicXML import, and
  eventually generated backing from the changes (#1098 themes 4 and 5).

## 6. Risks and open questions

- **Shipped but unvalidated.** The feature has had essentially zero critical
  use. The derivation is tested to the pitch class, but whether the generated
  exercises are *good practice material* (titles, rationales, granularity,
  the arpeggio fallback rate on real charts) is unknown. Slice 1 answers this
  cheaply before any new code.
- **Dedup edge cases.** No-clobber by reserved tag or title is implemented
  and tested, but a user who renames a generated exercise *and* strips its
  tag re-creates a duplicate on the next commit. Probably acceptable; worth a
  test if reconciliation is touched again.
- **Scope creep into generation quality.** The spec deliberately capped the
  chord vocabulary because every added quality is test surface on the one
  path a wrong note ruins. Phase C already widened it once; further widening
  needs the same discipline.
- **Single-genre bias.** This is a jazz-shaped feature in an
  instrument-agnostic app. VISION.md commits to a keyboard-and-jazz-first
  perspective, so it is aligned, but classical repertoire gets nothing from a
  chord chart, and the audit's parked "sections + chord charts" topic hints
  the same piece-structure machinery may be wanted for sections. A direction
  built here deepens the jazz lane rather than broadening the app.
- **The melody gap.** The scaffold's first rung stays an empty placeholder
  until the public-domain corpus bet; the copyright boundary (changes, not
  melodies) holds but caps the "receive its curriculum" promise.

## 7. Sources

- [specs/chart-to-scaffold.md](../../specs/chart-to-scaffold.md); PRs [#1109](https://github.com/jonyardley/intrada/pull/1109), [#1110](https://github.com/jonyardley/intrada/pull/1110), [#1111](https://github.com/jonyardley/intrada/pull/1111); issues [#1098](https://github.com/jonyardley/intrada/issues/1098), [#1106](https://github.com/jonyardley/intrada/issues/1106), [#1107](https://github.com/jonyardley/intrada/issues/1107), [#1083](https://github.com/jonyardley/intrada/issues/1083)
- [docs/research-foundation.md](../research-foundation.md) §§ deliberate practice, teacher functions, guided instruction
- [docs/rebuild-review.md](../rebuild-review.md) — chart.rs as "the sleeper asset"; the coach post-mortem
- Levine, [The Jazz Theory Book](https://www.amazon.com/Jazz-Theory-Book-Mark-Levine/dp/1883217040); [Aebersold Jazz](https://www.jazzbooks.com/); Crook, [Ready, Aim, Improvise!](https://halcrook.com/publications)
- Smither, [Guide-Tone Space, Music Theory Online 25.2](https://mtosmt.org/issues/mto.19.25.2/mto.19.25.2.smither.html)
- Wickens et al. (2013), [Effectiveness of part-task training, Human Factors 55(2)](https://pubmed.ncbi.nlm.nih.gov/23691838/)
- [Learn Jazz Standards: guide tones](https://www.learnjazzstandards.com/blog/learning-jazz/jazz-theory/use-guide-tones-navigate-chord-changes/) and [mapping a standard](https://www.learnjazzstandards.com/ljs-podcast/learn-jazz-standards/ljs-106-mapping-jazz-standard-improv-success/); [PianoGroove: shell voicings](https://www.pianogroove.com/jazz-piano-lessons/shell-voicings-for-jazz-piano/); [The Jazz Piano Site: guide tones](https://www.thejazzpianosite.com/jazz-piano-lessons/jazz-improvisation/guide-tones/); [VoiceLid: melody first](https://www.voicelidjazzguitar.com/jazz-guitar-blog/how-to-learn-jazz-standards-melody-first-someday-my-prince)
- Tools survey: [iReal Pro](https://www.irealpro.com/); [Mapping Tonal Harmony Pro](https://mdecks.com/mapharmony.phtml); [Chord Trainer](https://jazzchords.app/); [251 Chord](https://www.guitarworld.com/news/jazz-guitarist-matt-warnock-launches-251-chord-app-iphone-and-ipad); [ToneGym progression generator](https://www.tonegym.co/tool/item?id=progression-generator)
