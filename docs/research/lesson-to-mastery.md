# Research note: lesson-to-mastery (epic #1087)

*Stage 4.1 of [`rethink-plan.md`](../rethink-plan.md). Evidence and shape only:
no comparison against the other candidates and no recommendation. Written
2026-08-14.*

## 1. What it is

The weekly teacher-lesson loop, made first-class. The scenario from
[#1087](https://github.com/jonyardley/intrada/issues/1087): a lesson introduces
a tune (Strasbourg / St. Denis was the worked example) plus the exercises that
build it; the app captures the whole assignment in one pass, tracks each
exercise per piece, suggests what to practise between lessons, and lets
exercises progress along ladders (steps, keys).

Four workstreams, resequenced on 2026-07-15 from A → B → R → C to
**B → R → (A revisited) → C**:

- **A: Add a lesson** (#1080) — capture piece + exercises in one pass.
  **On hold.** A1's core event `AddPieceWithScaffold` merged (#1091) and was
  reverted (#1092) so main carries no unused surface while the shape is
  reconsidered.
- **B: Per-piece tracking** (#1081) — derive (exercise × piece) contexts from
  session-block data that already exists. Read-side only, no migration.
  **Open, labelled horizon:now.**
- **R: Up next card** (#1082) — one suggested session on the Practice tab,
  composed from the B grain. **Open, horizon:next, depends on B.** Unparked
  in the post-revert sweep (2026-08-14) but explicitly parked pending this
  rethink.
- **C: Exercise steps** (#1083) — one ladder mechanism for keys, levels,
  inversions. **C1 and C2 are done**: the `Variant` model, `variant` child
  table (sync-ready, upgrade-path tested), `variant_id` in the entry codec and
  real-bridge tests shipped in #1112/#1118, and the Steps UI plus the
  reflection step picker in #1123. **C3 is also done**: the twelve-key
  preset shipped in #1121 (commit `b08ce2c`, legitimately closing #46) and
  the roadmap marked it landed in #1124. C4 (step management polish) remains
  open.

So workstream C is essentially complete, and it survived the coach pivot and
its revert intact; B, R and a reshaped A are what remain.

## 2. The user problem

Jon has a real weekly lesson. Each one produces a tune plus three to five
exercises that scaffold it; today that is N passes through the add form plus
manual linking, and material leaks in the gap between the lesson and the first
practice session. This is the exact "material gets lost" problem VISION.md
opens with, and the epic serves it end to end: capture the assignment (A),
see how each exercise is going in the context of the piece it serves (B), get
an honest answer to "what do I practise today?" (R), and progress exercises
along their ladders rather than polishing one rating (C).

Against the (superseded but still useful) journey in
[`journeys.md`](../journeys.md): B is journey step 3 (track each exercise
separately, currently score-only), R is step 5 (recommended sessions,
currently missing entirely, "signals exist in core, nothing composes them"),
A deepens steps 1 and 2, and C completes step 3's keys dimension.

## 3. History in this repo

Three prior rounds are directly relevant, and they split cleanly along a
capture-versus-tracking line:

- **The lessons vertical (PR #273 era, 2025)** — lesson capture as an entity
  with date, notes and photos (#267). Built, then superseded by Goals in #711;
  migrations 0067 and 0068 dropped the lesson tables. Goals were themselves
  removed in #769 (roadmap Q5 and Q6 carry the post-mortem). Lesson-as-entity
  has died once already.
- **A1 built and reverted (#1091 / #1092, 2026-07)** — the one-pass capture
  event merged with full transactional semantics and real-bridge tests, then
  was reverted within days, not for quality but because the *shape* was in
  doubt: "rethinking whether lesson-capture is the right shape before building
  more on it". The implementation is recoverable from history.
- **C shipped and survived (#1112, #1118, #1123)** — the steps/variants
  mechanism, a data-model change with derived read surfaces, went through the
  coach pivot and the revert without being touched. `docs/rebuild-review.md`
  explicitly listed `Variant` and its ladder plumbing in the load-bearing keep
  column even when 60% of the domain code was scheduled for deletion.

The pattern: **capture-shaped admin surfaces (lessons, goals) have been built
and removed twice; tracking-shaped structure (variants, the scaffold
relationship, session history) persists**. B and R are tracking-shaped (pure
derivation over existing data); A is capture-shaped. The resequencing to
B-first already encodes this lesson, and any revisited A should be biased
towards "a faster path through existing add/link primitives" rather than a new
lesson entity.

## 4. Pedagogy evidence

What the literature says about the between-lesson gap, strongest first.

**Strong: students' unsupervised practice is poor, and the failure is
regulatory, not motivational.** McPherson & Renwick's three-year video study
of young learners found practice "confined almost exclusively to playing
through pieces once or twice", with most errors ignored or patched by
repeating a note or two ([McPherson & Renwick 2001](https://www.researchgate.net/publication/233093312_A_Longitudinal_Study_of_Self-regulation_in_Children's_Musical_Practice)).
This is the empirical core of `research-foundation.md` §8's self-taught
failure modes (comfort-zone bias, poor session structure, no stopping rules).
An app that carries the teacher's decomposition into the practice room is
aimed squarely at a well-documented gap.

**Strong: the lesson itself under-serves between-lesson practice.** McPherson
& Blackwell (2024, reported in
["Go home and practice"](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2025.1705295/full),
Frontiers in Psychology 2025) coded collegiate lessons and found 83.3% of
feedback was backward-looking assessment, 16.3% forward guidance, and 0.4%
goal-setting; 85.3% was task-level and only 5.5% targeted the self-regulation
skills unsupervised practice needs. The assignment students carry home is
mostly vague. Capturing it in a structured, per-exercise form at the moment
it is fresh is a direct response to this finding.

**Moderate: teaching structure changes behaviour, and somewhat changes
outcomes.** Self-regulation instruction (planning, goal selection, strategy
use, reflection) improved practice behaviours and strategy knowledge in
intervention studies with band students and advanced wind players
([Miksza 2015](https://journals.sagepub.com/doi/10.1177/0305735613500832);
[Prichard 2021](https://journals.sagepub.com/doi/10.1177/0022429420947132)).
Effects on measured performance achievement are positive but smaller than the
behavioural effects. A recent framework test in private violin instruction
([Kabrick & Duffin 2026](https://doi.org/10.1177/19484992251327976)) continues
this line. The honest read: scaffolding between-lesson practice reliably
changes *what students do*; the evidence that it changes *how well they play*
is thinner and slower.

**Moderate: breaking pieces into exercises is well-founded for complex
material.** The part-practice literature (segmenting or fragmenting a complex
task before recombining) shows part methods pay off when the task is high in
complexity and low in inter-component organisation, which describes a jazz
standard's scaffold well
([Fontana et al., whole/part meta-analysis](https://pubmed.ncbi.nlm.nih.gov/20038005/)).
This complements what `research-foundation.md` already carries: Duke's
teacher functions (decompose, sequence), Gagné's prerequisite hierarchies,
and the choice-overload case for a curated "practise this now" answer (§8-§10).
The lesson loop is those principles with the teacher, not an algorithm,
supplying the decomposition; the app only has to remember and replay it.

**Thin: the practice-diary literature specifically.** Practice notebooks and
assignment books are ubiquitous commercially, but direct outcome studies of
diary-keeping are scarce; searches surface products and adjacent SRL work,
not controlled evaluations. Electronic practice logs appear as instruments
in intervention research rather than as tested interventions themselves. The
epic should not claim "practice notebooks are proven to work"; the proven
part is the regulatory gap they address.

**A caution that cuts against pure assignment-capture.** Renwick & McPherson's
case study found a young learner spent about twelve times longer, and used
markedly more advanced strategies, on a self-chosen piece than on assigned
repertoire ([Renwick & McPherson 2002](https://www.researchgate.net/publication/228810889_Interest_and_choice_Student-selected_repertoire_and_its_effect_on_practising_behaviour)).
It is a single-child case study, and about children rather than adult
self-directed learners (a distinction `research-foundation.md` §8 already
flags as an assumption), but it argues the loop must stay autonomy-supportive:
the assignment is material the app remembers, never a regime it enforces.
R's "never a gate, dismissible, Build my own instead" framing already matches
this, and matches SDT (§4).

## 5. Shape sketch

Finishing the epic, in the settled order:

1. **B1** (core) — derive (exercise × piece) contexts from entry `group_id`
   → block → piece; `ExerciseContextView` in the ViewModel plus an "On its
   own" bucket; codec guard test that `group_id` round-trips. No migration,
   no new storage. **This is the smallest honest next slice**: read-side
   only, immediately visible value on data Jon already has, and the grain
   any future scheduler needs regardless of which direction Stage 4.2 picks.
2. **B2** (iOS) — "By piece" rows and context pills on exercise detail;
   per-this-piece score on the piece card.
3. **R1 + R2** — `up_next: Option<SuggestedSession>` in `PracticeView` with
   core-owned reason strings ("weakest step", "6 days since you played it"),
   one event to build the setlist, one dismissible card. No new storage.
4. **A revisited** — reshaped before rebuild. Given the history in §3, the
   likely honest shape is a fast batch path through existing add/link
   primitives (A1's transactional event is recoverable from #1091) rather
   than any lesson entity. Whether it earns a dedicated screen is the open
   design question the hold exists to answer.
5. **C4** — step management polish (rename, reorder, archive). C3's
   twelve-key preset already shipped (#1121).

Core/FFI/schema surface: B and R are ViewModel additions plus derivation
logic, so they cross the FFI bridge (domain-sensitivity override applies:
tier up, real-bridge round-trip tests per the #846 class) but touch **no
schema**. C1's dangerous work (migration on only-copy data, codec widening)
is already paid. A re-adds one bridge-crossing event whose semantics were
already proven once. Nothing in the epic needs the API; it is local-first
throughout, with variants consciously scoped local-only until sync.

## 6. Risks and open questions

- **Is lesson-capture the right shape?** That is the A hold, and it is still
  unanswered. Two entity-shaped captures (lessons #273, goals #711/#769) have
  been deleted before. The Renwick & McPherson caution adds a pedagogical
  angle: a capture flow that frames practice as "the teacher's list" may
  undercut the autonomy that sustains practice. A reshaped A should capture
  material, not authority.
- **Up-next and the coach taint.** #1082 was flagged in the roadmap-banner
  triage as possibly coach-shaped; the 2026-08-14 sweep unparked it but left
  it parked pending this rethink. The distinction that keeps it honest: the
  card proposes and explains, one tap starts, dismissal is free, and the
  manual builder remains the primary surface. The coach failed by gating;
  R fails the same way only if scope creeps from suggestion to prescription.
  The anti-coach discipline (each slice shipped and used before the next) is
  the guard.
- **Single-user evidence.** Jon has a real weekly lesson, so every slice gets
  authentic use within a week of shipping. That is strong personal signal and
  a fast feedback loop, but it is n=1: the loop's shape (one tune plus
  scaffold per week, jazz keyboard) may not generalise, and the pedagogy
  evidence above is mostly about children and students in formal tuition.
- **Outcome evidence is behavioural, not performance-level.** The strongest
  claim the research supports is "structure changes what happens between
  lessons"; the claim "and therefore you play better" is plausible but
  thinly evidenced. Worth keeping the product's promise calibrated to that.
- **Sequencing risk is low.** B is independently valuable even if R and A
  never ship, and C is essentially complete; the epic degrades gracefully.

## 7. Sources

- [Epic #1087](https://github.com/jonyardley/intrada/issues/1087), sub-issues
  [#1080](https://github.com/jonyardley/intrada/issues/1080),
  [#1081](https://github.com/jonyardley/intrada/issues/1081),
  [#1082](https://github.com/jonyardley/intrada/issues/1082),
  [#1083](https://github.com/jonyardley/intrada/issues/1083); PRs #1091,
  #1092, #1112, #1118, #1123.
- [`docs/research-foundation.md`](../research-foundation.md) §3, §4, §8, §9,
  §10 (Duke, Gagné, Katz & Assor, Kirschner et al., Macnamara et al.).
- [`docs/rebuild-review.md`](../rebuild-review.md) (variant ladder in the
  keep column); [`docs/roadmap.md`](../roadmap.md) Q5/Q6 (goals and lessons
  history); [`docs/journeys.md`](../journeys.md) steps 2, 3, 5.
- McPherson, G. E., & Renwick, J. M. (2001). A longitudinal study of
  self-regulation in children's musical practice. *Music Education Research*,
  3(2). [PDF](https://www.researchgate.net/publication/233093312_A_Longitudinal_Study_of_Self-regulation_in_Children's_Musical_Practice)
- Renwick, J. M., & McPherson, G. E. (2002). Interest and choice:
  student-selected repertoire and its effect on practising behaviour.
  *BJME*, 19(2). [Link](https://www.researchgate.net/publication/228810889_Interest_and_choice_Student-selected_repertoire_and_its_effect_on_practising_behaviour)
- Blackwell (2025). "Go home and practice": how shaping feedback to students
  can foster independent musicianship. *Frontiers in Psychology*.
  [Link](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2025.1705295/full)
  (carries the McPherson & Blackwell 2024 feedback-coding figures)
- Miksza, P. (2015). The effect of self-regulation instruction on the
  performance achievement, musical self-efficacy, and practicing of advanced
  wind players. *Psychology of Music*, 43(2).
  [Link](https://journals.sagepub.com/doi/10.1177/0305735613500832)
- Prichard, S. (2021). The impact of music practice instruction on middle
  school band students' independent practice behaviors. *JRME*, 69(2).
  [Link](https://journals.sagepub.com/doi/10.1177/0022429420947132)
- Kabrick, H. R., & Duffin, L. C. (2026). Testing a framework for teaching
  self-regulation skills in private violin instruction.
  [DOI](https://doi.org/10.1177/19484992251327976)
- Fontana, F. E., et al. Whole and part practice: a meta-analysis.
  *Perceptual and Motor Skills*.
  [PubMed](https://pubmed.ncbi.nlm.nih.gov/20038005/)
- Miksza, P., Prichard, S., & Sorbo, D. (2012). An observational study of
  intermediate band students' self-regulated practice behaviors. *JRME*,
  60(3). [Link](https://journals.sagepub.com/doi/abs/10.1177/0022429412455201)
