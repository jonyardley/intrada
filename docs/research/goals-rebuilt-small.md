# Research note: goals, rebuilt small

*Stage 4.1 of [`../rethink-plan.md`](../rethink-plan.md). One of five candidate
next-directions; no comparison or recommendation here (that is Stage 4.2).
Evidence and shape only. Written 2026-08-14.*

## 1. What it is

The deliberately-small goal shape ruled in roadmap Open Question 5
(re-resolved 2026-07-14) and VISION.md's "The Scheduling Intelligence":

- An **outcome statement** ("learn Body and Soul", "improvise fluently over
  rhythm changes") **linked to library items**, with an **optional target
  date**. Nothing more.
- **Consumed by session planning**: the goal exists to feed "what should I
  practise?", never as an admin surface of its own.
- **None** of the apparatus that sank the previous versions: no per-item
  confidence or tempo targets, no progress percentages, no completion
  ceremony, no photos.
- The per-item **priority star** stays as the zero-ceremony layer beneath; a
  goal is the named ambition sitting above it.

## 2. The user problem

This is journey step 4, "Goals drive planning" (`docs/journeys.md`), status
"Missing – to be rebuilt small per the 2026-07 ruling". It serves the Plan
pillar and, in the vision's five layers, is the bridge between Layer 2 (Plan)
and Layer 3 (Space): the scheduler VISION.md describes combines spacing
urgency, interleaving, **goal alignment**, and time fitting, and today the
goal-alignment input simply does not exist.

What breaks without it:

- The library answers "what do I own?" but not "what am I working towards?".
  The priority star (live on iOS: star toggle and filter in `LibraryScreen`,
  star badge on `LibraryItemCard`) marks *which* items matter but cannot say
  *why* or group them under a named ambition. Two concurrent ambitions (an
  exam programme and a jazz tune) collapse into one flat starred set.
- Journey step 5 (recommended sessions) has no intent signal to draw on.
  The research foundation's choice-overload section (§9) is explicit that
  goal-driven filtering is one of the two levers against decision paralysis,
  and VISION.md names deciding-what-to-practise as one of the three core
  problems.
- A musician preparing for a dated event (exam, audition, gig) has nowhere
  to put the date. `specs/priority-items.md` accepted this loss knowingly
  ("Deliberately not doing: deadlines… revisit only if it bites").

## 3. History in this repo

Goals have been built twice and removed twice. This is the strongest internal
evidence and it cuts both ways: the *job* keeps being re-affirmed, the
*apparatus* keeps being deleted.

- **Build one** — an early goals subsystem across core, API, web and iOS.
  Removed in **#213** ("Remove goals feature for redesign"): delete the domain
  types, routes, views, navigation, and drop the table, "to be revisited from
  scratch with a clearer vision of what value they add".
- **The lessons vertical (#273)** came between the two builds and was itself
  superseded by Goals in #711 (migrations 0067–0068 dropped the lesson
  tables). Two adjacent Plan-layer verticals, both gone.
- **Build two** — Goals **#711–#740**: a `Goal` entity with `GoalStatus`,
  linked `GoalItem`s carrying per-item confidence *and* tempo targets,
  deadlines with overdue badges, completion/reopen, `GoalPhoto`s in R2, an
  auto-suggest "looks ready" banner, four views, a tab. Removed in **#769**
  (2026-07-14) with append-only drop migrations 0081–0083 that deleted live
  production data, reviewed and approved as a clean slate.
- **The diagnosis** (`specs/priority-items.md`) is unusually crisp:
  **setup burden before value** (create a goal, link items, set targets
  before anything comes back) and **conceptual overload** (goals + linked
  items + confidence + tempo targets + deadlines + completion + photos held
  in the user's head). "The underlying job is real… but it should be nearly
  free to express, not a form to fill in."
- **What survived**: the per-item `priority: bool` (#765, Phase A merged;
  the iOS star surface is live post-revert), and two salvaged mechanics:
  least-ready-first ordering (deleted with #769, reconstructable from
  history) and per-item progress derivation (`latest_score`,
  `latest_achieved_tempo`), which stayed. **#763** (priority UI, written
  against the deleted Leptos shell; its iOS substance largely exists) and
  **#764** (the one "neglected priority" Track signal) remain open.
- The coach era then rebuilt goal-like intent a third way ("intent declared
  at three altitudes, back-chained through the graph") and that whole
  direction was reversed in #1344.

## 4. Pedagogy evidence

What is already in [`../research-foundation.md`](../research-foundation.md)
and what fresh evidence adds. Strong and thin separated.

### Strong

- **Goal-directed practice beats mindless repetition.** The qualitative core
  of deliberate practice that survived the Macnamara critiques: focused
  attention on specific goals, edge-of-ability work, feedback incorporation
  (research-foundation §3, citing Ericsson et al. 1993 and Ambrose et al.
  2010). Hallam's definition of effective practice is explicitly
  goal-referenced: "that which achieves the desired end-product… without
  interfering negatively with longer-term goals", and her work finds
  metacognitive planning, not hours, distinguishes experts
  ([Hallam 1997/2012](https://journals.sagepub.com/doi/10.1177/0305735612443868)).
- **Goal setting is the forethought phase of self-regulated learning.**
  McPherson & Renwick applied Zimmerman's three-phase SRL cycle
  (forethought → performance → self-reflection) to instrumental practice;
  their longitudinal work (157 children over 3 years) found early
  differences in self-regulatory engagement, goal setting included, account
  for much of subsequent progress
  ([McPherson & Renwick, SRL in music practice](https://www.researchgate.net/publication/321873981_Self-regulated_learning_in_music_practice_and_performance);
  [Araújo 2016 overview](https://pmc.ncbi.nlm.nih.gov/articles/PMC5006062/)).
  The SRL literature also recommends setting goals **hierarchically and
  temporally**: anchored to future events like recitals and exams, then
  decomposed into session objectives. That is precisely outcome statement +
  linked items + optional date.
- **Specific goals outperform vague ones** (Locke & Latham's goal-setting
  theory, robust across hundreds of studies;
  [Locke & Latham 2006](https://home.ubalt.edu/tmitch/642/articles%20syllabus/locke%20latham%20new%20dir%20gs%20curr%20dir%20psy%20sci%202006.pdf)).
  A named outcome ("learn Body and Soul") is a specific goal; "get better at
  jazz" is not. Two load-bearing caveats: goal setting **requires feedback**
  to work (Intrada's derived progress supplies this without user-set
  targets), and on **novel or complex tasks, assigned specific-difficult
  goals can hurt** performance by crowding out strategy development, which
  argues for self-authored goals and against the app assigning them.
- **Process goals beat outcome goals during acquisition.** Zimmerman &
  Kitsantas' dart-throwing studies: novices given process goals outperformed
  outcome-goal groups on skill, self-efficacy (d ≈ 0.68 vs product goals)
  and intrinsic interest (d ≈ 0.74), with a developmental shift toward
  outcome goals as skill automatises
  ([Kitsantas & Zimmerman 2006](https://ssrlsig.org/wp-content/uploads/2018/02/kitsantas-zimmerman-2006-enhancing-self-regulation-of-practice.pdf);
  [Zimmerman & Kitsantas 1997](https://www.researchgate.net/publication/232582156_Developmental_phases_in_self-regulation_Shifting_from_process_to_outcome_goals_Journal_of_Educational_Psychology_89_29-36)).
  Implication: the goal entity is the *container*; the practice work itself
  stays process-framed (the linked items and the session loop), which is
  exactly what linking a goal to library items does. A goal surface that
  pushed outcome numbers at the user mid-acquisition would fight this
  evidence.
- **Implementation intentions close the intention-behaviour gap.** If-then
  plans linking a situation to an action raise goal attainment with
  medium-to-large effect (d = 0.65 across 94 tests,
  [Gollwitzer & Sheeran 2006](https://www.researchgate.net/publication/37367696_Implementation_Intentions_and_Goal_Achievement_A_Meta-Analysis_of_Effects_and_Processes)).
  "Consumed by session planning" is the app performing this translation:
  the goal (intention) becomes a one-tap concrete session (the plan), so
  the user never has to bridge the gap themselves.

### Where evidence argues against features

- **Target dates are the risky part.** Externally imposed deadlines reduced
  subsequent intrinsic interest in the classic
  [Amabile, DeJong & Lepper 1976](https://www.hbs.edu/faculty/Pages/item.aspx?num=7343)
  study, and SDT treats deadlines, imposed goals and pressured evaluation as
  shifting the perceived locus of causality external, undermining intrinsic
  motivation ([Ryan & Deci 2000](https://selfdeterminationtheory.org/SDT/documents/2000_RyanDeci_SDT.pdf)).
  Self-set dates in autonomy-supportive contexts are less harmful, but the
  old build's **overdue badges** were the pressured-evaluation framing this
  literature warns about, and they conflict with the vision's
  comeback-not-streak principle. Evidence supports: date as quiet context
  for scheduling, never as a red badge.
- **No evidence supports the heavy apparatus.** Nothing in the goal-setting,
  SRL or SDT literature requires user-set confidence targets, progress
  percentages, completion ceremonies or attached photos for goals to work.
  The mechanisms that carry the effect are specificity, feedback, and
  translation into action, all deliverable by statement + items + derived
  progress. Meanwhile choice-overload (research-foundation §9) and the
  autonomy findings actively favour fewer concepts and fewer forms.

### Thin

- No published study evaluates goal features in music practice *apps*; the
  competitive table in research-foundation §12 notes goal frameworks as a
  differentiator, not an evidenced mechanism.
- The sport goal-setting meta-analytic literature
  ([systematic review 2022](https://www.tandfonline.com/doi/full/10.1080/1750984X.2022.2116723))
  shows positive but heterogeneous effects; transfer to self-directed adult
  musicians is assumed, the same caveat research-foundation already attaches
  to its self-taught failure-mode evidence.
- Whether an optional, self-set target date helps or hurts *this* audience
  is genuinely unknown; the Amabile evidence is about imposed deadlines on
  intrinsically interesting tasks, not self-chosen exam dates.

## 5. Shape sketch

**Smallest honest slice**: the entity plus its consumer, shipped together
(a goal nobody consumes is admin, the product-level version of "nothing
unread stays in the tree"):

- Core: a `Goal` carrying a client-minted ulid, outcome statement, linked item ids,
  optional target date, `updated_at`/`deleted_at`. Events: add, update
  (statement/date/links), delete. Local-first only: GRDB table via a new
  migration, persistence ops, no HTTP.
- One quiet surface in Plan: create in seconds (one text field, pick items,
  optionally a date), listed above or beside the priority view.
- The consumer: **"Practise this goal"**, one tap that loads the linked
  items into the session builder, least-ready-first (reconstruct the
  ordering #769 deleted). This is the direct heir of "Practise your priorities"
  (#763) and could share its mechanism.

**A plausible second slice**: the date consumed by planning (items linked to
a dated goal surface more as the date nears: quiet weighting, no overdue
state), and/or goal context shown on linked items ("part of: Body and Soul").
The Track-side neglect signal stays with #764's one-signal discipline.

**Repo surface**: a new domain entity crossing the FFI bridge plus a GRDB
migration is **Tier 3** by the domain-sensitivity override. That means a
short spec riding with Phase A, the core-first/screens-second PR split, the
offline-first PR checklist (client ulid, tombstones, `PersistenceOutput::
Failed`), real-bridge round-trip tests for the new types, and keeping `Goal`
out of the `ActiveSession` crash-recovery blob's transitive graph (the
wire-pin hazard, #1345).

## 6. Risks and open questions

- **Third-time failure mode: scope accretion.** Both deaths came from the
  same accretion (targets, deadlines-with-badges, completion, photos). The
  ruling's "nothing more" needs to be enforced in the spec: anything beyond
  statement + items + optional date is a new product decision, not an
  extension.
- **Setup burden is the other proven killer.** If creating a goal takes more
  than a few seconds, or value only arrives after linking N items, it
  repeats #769's diagnosis. The star costs one tap; a goal must justify
  every tap beyond that.
- **The star has not been given its full chance.** `specs/priority-items.md`
  explicitly deferred "grouping priorities into named pursuits… only revisit
  if a flat priority list proves insufficient", and #764's neglect signal is
  still unshipped. The honest open question: is there evidence *from use*
  that the flat starred set is insufficient, or would a goal entity arrive
  before its predecessor was tested? (Noted here, not answered: that is
  Stage 4.2's territory.)
- **The planning consumer is itself unbuilt.** Journey step 5 (recommended
  sessions) is missing; goal alignment is one input to a scheduler that does
  not exist. Slice 1 dodges this by making the builder the consumer, but the
  full "consumed by session planning" promise depends on a future planner,
  a sequencing dependency this note flags for the comparison.
- **Coach-pivot lessons bind hard here** (`../rebuild-review.md`, reversal
  in #1344). The coach failed by deciding too much and gating the user.
  Goals sit exactly on that edge: they must *inform* suggestions the user
  can ignore, never prescribe or gate. The Locke & Latham complex-task
  caveat and the SDT autonomy evidence both point the same way: goals are
  authored by the musician, and the app's use of them stays dismissible.
- **Date semantics.** Optional target date with no overdue state is the
  evidence-aligned shape, but "what happens when the date passes?" needs a
  deliberate answer (probably: nothing visible beyond the date itself)
  before build, or an overdue badge sneaks back in.
- **Goal completion.** With no completion ceremony, how does a finished goal
  leave the surface? Soft-delete on a user's say-so is the small answer, but
  it is a decision to make, not to discover.

## 7. Sources

Internal:

- `docs/roadmap.md` Open Question 5; `VISION.md` "The Scheduling
  Intelligence"; `docs/journeys.md` steps 4–5; `docs/research-foundation.md`
  §3, §4, §9, §10; `docs/rebuild-review.md`; `specs/priority-items.md`
- Issues/PRs: #213 (first removal), #711–#740 (second build), #769 (second
  removal), #273 (lessons vertical), #765 / #763 / #764 (priority star),
  #1344 (coach revert)

External:

- [Locke & Latham (2006), New directions in goal-setting theory](https://home.ubalt.edu/tmitch/642/articles%20syllabus/locke%20latham%20new%20dir%20gs%20curr%20dir%20psy%20sci%202006.pdf)
- [Goal-setting for musicians (Psyc for Musos summary of Locke & Latham applied to practice)](https://psycformusos.com/2020/11/16/goal-setting-for-musicians-how-to-practise-proactively-not-reactively/)
- [McPherson & Renwick, Self-regulated learning in music practice and performance](https://www.researchgate.net/publication/321873981_Self-regulated_learning_in_music_practice_and_performance)
- [Araújo (2016), Setting the stage for SRL and metacognition instruction in musical practice](https://pmc.ncbi.nlm.nih.gov/articles/PMC5006062/)
- [Silva & Marinho (2025), SRL processes of advanced musicians: PRISMA review](https://journals.sagepub.com/doi/10.1177/10298649241275614)
- [Hallam et al. (2012), The development of practising strategies in young people](https://journals.sagepub.com/doi/10.1177/0305735612443868)
- [Zimmerman & Kitsantas (1997), Developmental phases in self-regulation: shifting from process to outcome goals](https://www.researchgate.net/publication/232582156_Developmental_phases_in_self-regulation_Shifting_from_process_to_outcome_goals_Journal_of_Educational_Psychology_89_29-36)
- [Kitsantas & Zimmerman (2006), Enhancing self-regulation of practice](https://ssrlsig.org/wp-content/uploads/2018/02/kitsantas-zimmerman-2006-enhancing-self-regulation-of-practice.pdf)
- [Gollwitzer & Sheeran (2006), Implementation intentions and goal achievement: meta-analysis](https://www.researchgate.net/publication/37367696_Implementation_Intentions_and_Goal_Achievement_A_Meta-Analysis_of_Effects_and_Processes)
- [Amabile, DeJong & Lepper (1976), Effects of externally imposed deadlines on subsequent intrinsic motivation](https://www.hbs.edu/faculty/Pages/item.aspx?num=7343)
- [Ryan & Deci (2000), Self-determination theory and the facilitation of intrinsic motivation](https://selfdeterminationtheory.org/SDT/documents/2000_RyanDeci_SDT.pdf)
- [Weinberg et al. (2022), Goal setting in sport: systematic review and meta-analysis](https://www.tandfonline.com/doi/full/10.1080/1750984X.2022.2116723)
