# Research note: the Space layer (spaced repetition and mastery decay)

*Stage 4.1 of [`rethink-plan.md`](../rethink-plan.md). Evidence and shape only;
no comparison against the other candidates and no recommendation. 2026-08-14.*

## 1. What it is

Layer 3 of the five-layer vision (Capture → Plan → **Space** → Show → Guide):
the app models how mastery of each library item decays between practices and
resurfaces material before it is forgotten, feeding that signal into session
planning. VISION.md frames it as "items you're learning surface frequently;
items you've consolidated appear less often but before you've forgotten them",
and "The Scheduling Intelligence" names spaced-repetition urgency as the first
of four scheduler inputs (with interleaving, goal alignment, and time fitting).
[`roadmap.md`](../roadmap.md) lists the Space layer as an active gap: today
"Space" exists only as manual behaviour (the user happening to return to
things), plus a binary neglect signal in analytics.

## 2. The user problem

Repertoire rot. A piece is learned, scored 8/10, and then silently decays
while attention moves to this month's material. Nothing in the app changes as
that happens: the library shows the same last score whether it was earned
three days or three months ago, so the display becomes progressively more
dishonest about the present. The costs today:

- **Stale scores masquerade as current mastery.** `latest_score` has no age
  on most surfaces; a 4 from June reads the same as a 4 from yesterday.
- **The neglect signal is binary and shallow.** `compute_neglected_items`
  (`crates/intrada-core/src/analytics.rs`) flags items untouched for 14 days,
  capped at five, sorted by staleness. It knows nothing about how well an
  item was known when last played, so a fragile piece and a consolidated one
  go cold on the same clock.
- **The decision burden returns.** The vision's second problem statement
  (deciding what to practise is paralysing) is only half-answered by the
  builder: the user still has to remember what is going stale, which is
  exactly the memory task the app exists to absorb.

## 3. What exists in the repo

A decay model has real inputs to consume today, all core-owned:

- **Per-item practice history.** `build_practice_summaries`
  (`crates/intrada-core/src/app.rs`) already folds every session into an
  `ItemPracticeSummary` per item: `session_count`, `total_minutes`,
  `latest_score`, dated `score_history`, `latest_tempo`, dated
  `tempo_history`, `last_practiced_at`. This is precisely the trace a decay
  estimate needs, and it is recomputed as a pure projection.
- **Per-entry evidence.** `SetlistEntry`
  (`crates/intrada-core/src/domain/session.rs`) carries `score`,
  `achieved_tempo`, rep counts and durations; `PracticeSession` carries
  timestamps. Sessions persist locally (GRDB), offline-first.
- **A priority flag** on `Item` as the existing zero-ceremony "this matters"
  signal, and `compute_neglected_items` / `compute_score_changes` as the
  existing staleness and trend surfaces.
- **A designed insertion point.** The Up next card (#1082, epic #1087) is a
  scaffold-aware suggested session on the Practice tab, explicitly built so
  that "the full scheduler (spacing, interleaving, goals) later swaps the
  brain behind it". Its reason-string pattern ("6 days since you played it")
  is the intended voice for a spacing signal.

Missing:

- **Any decay or interval state.** Nothing models forgetting; there is no
  per-item review interval, stability, or due date, persisted or derived.
- **Grain below the item, for pieces.** Exercises have it: the Variant/steps
  mechanism (#1083, shipped) gives per-key steps with per-step scores. Pieces
  do not: no sections (#50), so the schedulable unit for a piece is the whole
  piece. Prior art (below) says the useful grain for pieces is the section.
- **A settled memory signal.** Roadmap Open Question 3 (scoring + tempo
  coupling) is unresolved: whether "how well do I know this" means score,
  or score-at-tempo, changes what the decay model decays.
- **Sparse-data handling.** `score` is optional per entry; many entries carry
  time but no rating. Any model must degrade gracefully to "time since
  practised" alone.

## 4. Pedagogy evidence

The repo's own evidence base ([`research-foundation.md`](../research-foundation.md)
§1, §2, §6) already covers the core literature; summarised and extended here.

**Spacing is robust in general, moderated for motor skills.** Donovan &
Radosevich (1999; 63 studies) found spaced practice beats massed with d=0.46,
but the effect shrinks as task complexity rises. Cepeda et al. (2006; 317
experiments) confirmed the benefit and found optimal gaps grow with the
desired retention interval. For motor learning specifically, Lee & Genovese
(1988) found distribution-of-practice effects differ sharply between discrete
and continuous tasks, and the classic finding (Adams, 1987) is that continuous
skills are retained far better over long gaps than discrete ones. Music mixes
both kinds, which cautions against one curve for the whole library.

**Across-session returns have direct music evidence.** Simmons (2012) showed
non-pianists learning a keyboard sequence gained accuracy only when sessions
were 24 hours apart, implicating sleep-based procedural consolidation; the
research-foundation doc adds the honest caveat that spacing effects are not
always demonstrable for complex motor sequences. Wellmann & Skillicorn (2024)
argue retrieval practice should transfer to jazz performance and recommend
spaced returns at 24-hour-plus intervals, but this is a theoretical proposal,
not yet an empirical result.

**Interleaving is adjacent and mixed.** Carter & Grahn (2016) found expert
raters preferred interleaved-schedule outcomes with advanced clarinetists
(n=10) while most participants preferred blocked practice; Stambaugh (2009)
found benefits for beginners but Stambaugh & Demorest (2010) found none for
intermediates; Mathias & Goldman (2025) suggest ramping interference within a
session. Relevant because a scheduler that spaces material also decides its
ordering, and because blocked practice *feeling* better than it works is the
same self-report bias that will contaminate mastery ratings.

**The algorithm transfer is the weak link, and the repo already says so.**
Research-foundation §1 notes: SM-2 was designed for verbal flashcards, and
"no published research validates SM-2 parameters for motor skill scheduling".
That remains true of the newer generation. FSRS (the DSR-model scheduler now
default in Anki) is validated on flashcard review logs, reports 20-30% fewer
reviews for equal retention, and needs on the order of a thousand reviews
before per-user parameter fitting beats its defaults. Duolingo's half-life
regression (Settles & Meeder, 2016) fitted forgetting half-lives with machine
learning over 13 million learner traces. Both assume graded recall of a
discrete fact and data volumes a solo musician will never produce. Nothing in
the published literature validates any of these schedulers for repertoire.

**Prior art applying SRS to repertoire exists but is small and self-aware.**
Piano Practice Assistant (named in the repo's competitive table) implements
mastery decay with user-controlled overrides, schedules at the *subsection*
level, and its developer states plainly that spaced repetition "for complex
psychomotor skills like piano performance" is "not particularly well-studied".
Musicians on Piano World and in Malcolm Sailor's write-up of Anki for jazz
practice report the same two lessons: a piece is not a card (it needs
sub-piece grain), and opaque intervals get abandoned (users want to see and
override the algorithm). Phiano is a newer music-specific SRS app in the same
vein. The pattern is a handful of indie tools and forum experiments, not an
evidence base.

## 5. Shape sketch

The honest reading of the evidence is: spacing across sessions is
well-supported as a *principle*; every specific scheduling formula is an
informed guess. That argues for shipping the principle passively first and
earning the right to schedule.

- **Slice 1 — a passive "getting cold" signal.** A per-item freshness
  estimate derived from `last_practiced_at`, `score_history` recency, and
  practice count: a graded replacement for the binary 14-day neglect flag.
  Pure core projection over existing data, no new storage. Surfaces as an age
  next to scores, a "getting cold" grouping in the library, and richer
  neglected-items output. FFI surface: ViewModel additions only (still a
  bridge change, so the domain-sensitivity override applies).
- **Slice 2 — cold items feed the builder.** The freshness signal ranks a
  quiet "due a return" shelf in the session builder and supplies reason
  strings to the Up next card (#1082): one tap adds an item, dismissal costs
  nothing. No interval state yet; the ranking is recomputed from history.
- **Slice 3 — the full scheduler.** Persisted per-item scheduling state
  (interval or stability, updated by post-practice ratings), ordering that
  interleaves material types, goal weighting: the brain-swap #1082
  anticipated. This is
  the Tier 3 step: new GRDB table or columns (with `updated_at` /
  `deleted_at` and an upgrade-path test), migration, new events, ViewModel
  changes. Parameters stay visible and user-tuneable, and the core should log
  prediction-versus-next-score residuals from day one, because the parameters
  can only be calibrated empirically (research-foundation §1 calls this a
  known unknown).

Slice 1 is independently valuable and cheaply reversible, which is the
anti-coach-pivot discipline the rethink plan mandates. Slices are gated on
use, not on a schedule.

## 6. Risks and open questions

- **The coach-pivot failure mode.** #1344 reversed a product where the app
  decided what you practised and gated progress on evidence
  ([`rebuild-review.md`](../rebuild-review.md) banner, roadmap banner). A Space
  layer is the app forming an opinion about what you should play next, which
  is the same road. The line that held before the pivot and should hold here:
  the app **surfaces evidence and suggests; it never prescribes, gates, or
  auto-builds**. "Cold for three weeks, last scored 6" is a fact the user
  reads; "you must review this today" is the coach again. The builder stays
  the primary surface; every suggestion is one dismissible tap; SDT autonomy
  support (research-foundation §4: "suggest and support, not dictate") is the
  test.
- **Borrowed authority.** A due date or retention percentage looks like
  measurement but would be a heuristic wearing a lab coat. Copy must present
  the signal as staleness, not as a claim about the user's memory
  ([`tone-of-voice.md`](../tone-of-voice.md)); showing "getting cold" is honest
  where "72% retained" would not be.
- **Data sparsity breaks fitted models.** One user, irregular practice,
  optional ratings. FSRS needs roughly a thousand reviews to personalise;
  HLR needed millions of traces. A fixed, visible heuristic with manual
  override is the only defensible v1; anything "adaptive" is later, if ever.
- **Self-report noise.** Ratings taken right after massed reps overstate
  retention (the blocked-practice illusion in Carter & Grahn). A score logged
  minutes after ten repetitions is a performance measure, not a retention
  measure; the model should weight scores from *returns* over scores from
  drilling.
- **Wrong grain.** Prior art says sections; for pieces the repo has only
  whole items (exercises already carry per-key steps via #1083). Scheduling
  whole pieces is coarse but shippable; deciding whether Slice 3 waits for
  sections (#50) is a real sequencing question.
- **What decays?** Roadmap Q3 unresolved: score alone, or score-at-tempo. A
  piece "kept" at half tempo is not kept.
- **Error asymmetry.** Too-eager resurfacing nags and erodes trust (the
  crying-wolf failure); too-lax is today's silent rot. Continuous-skill
  retention (Adams, 1987) suggests consolidated repertoire decays slowly, so
  a flashcard-shaped curve will over-alarm by default. Bias lax, let the user
  tighten.
- **Cold start.** Items never scored or never practised have no history;
  the signal must say "no data" rather than inventing a curve.

## 7. Sources

In-repo: [`VISION.md`](../../VISION.md) (Layer 3, The Scheduling
Intelligence), [`docs/roadmap.md`](../roadmap.md) (gap statement, Q3),
[`docs/research-foundation.md`](../research-foundation.md) (§1, §2, §4, §6 and
its reference list), [`docs/rebuild-review.md`](../rebuild-review.md), issue
#1082 / epic #1087, `crates/intrada-core/src/analytics.rs`,
`crates/intrada-core/src/app.rs`, `crates/intrada-core/src/domain/session.rs`.

External:

- Cepeda et al. (2006), distributed practice meta-analysis — https://doi.org/10.1037/0033-2909.132.3.354
- Donovan & Radosevich (1999), spacing × task complexity meta-analysis (via research-foundation §1)
- Lee & Genovese (1988), distribution of practice, discrete vs continuous — https://pubmed.ncbi.nlm.nih.gov/2489826/
- Simmons (2012), distributed practice and consolidation in musicians — https://doi.org/10.1177/0022429411424798
- Carter & Grahn (2016), blocked vs interleaved for advanced players — https://doi.org/10.3389/fpsyg.2016.01251
- Mathias & Goldman (2025), contextual interference in violin practice — https://doi.org/10.1177/00224294231222801
- Settles & Meeder (2016), half-life regression at Duolingo — https://research.duolingo.com/papers/settles.acl16.pdf
- FSRS scheduler (open-spaced-repetition, DSR model) — https://github.com/open-spaced-repetition/free-spaced-repetition-scheduler
- Piano Practice Assistant, spaced repetition for musicians — http://pianopracticeassistant.com/spaced-repetition/
- Malcolm Sailor, Anki for jazz practice — https://malcolmsailor.com/2023/11/22/anki-for-jazz-practice.html
- Piano World forum, piano practice with SRS (Anki) — https://forum.pianoworld.com/ubbthreads.php/topics/2012638/all/piano-practice-with-spaced-repetition-systems-anki.html
- Procedural skill retention and decay, meta-analytic review (2026, in press) — https://psycnet.apa.org/manuscript/2026-23054-001.pdf
