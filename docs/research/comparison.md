# Stage 4.2 — choosing the next direction

*Phase R, Stage 4.2 ([`rethink-plan.md`](../rethink-plan.md)): the one-page
comparison of the five Stage 4.1 notes in this folder. Jon picks; Stage 4.3
(Tier-3 spec, then design, then slices) follows the pick. Written 2026-08-14.*

## What the research changed about the choice

The five notes were commissioned as equals. They did not come back as equals:

- **Chart-to-scaffold Phase B is already shipped.** #1110 merged 2026-07-17,
  Phase C (#1111) the same week; the coach pivot froze it before it was ever
  used critically. #1106 is stale and should close as shipped; the only unbuilt
  residue is the twelve-key ladder wiring (#1107), plausibly Tier 2 now that
  the `Variant` mechanism it was blocked on has landed. What remains here is
  *validation through use*, not a build direction.
- **The metronome is already committed** as audit Phase 4 (#1366, spec-first,
  Tier 3), so it ships through the Stage 3 pipeline whichever direction wins.
  Its note mostly de-risks that work: the click engine is recoverable intact
  from `8af4891^`, the tempo plumbing (target, per-entry capture, computed
  history) already spans the core, and the differentiating claim, tempo as a
  tracked measure, is genuinely unoccupied ground. Picking it as *the
  direction* would re-badge scheduled work.
- **The other three are stages of one pipeline, not rivals.** Lesson-to-mastery
  B derives the (exercise × piece) grain; R (the Up next card, #1082) is the
  one consumer surface; spacing urgency (Space layer) and goal alignment
  (goals) are *inputs to that consumer*. Neither input has anywhere to land
  until B and R exist. The real question is where the pipeline starts.

## Side by side

| Candidate | Smallest slice | Slice tier | Evidence strength | Killer risk |
|---|---|---|---|---|
| Lesson-to-mastery (B→R) | B1: derive per-piece contexts, read-only | 2 (bridge override) | Strong on the regulatory gap it serves | A's capture shape still unproven (twice-deleted class) |
| Space layer | Passive "getting cold" signal, pure projection | 2 (bridge override) | Principle robust; every formula unvalidated for repertoire | Coach-shaped opinion; sparse self-rated data |
| Goals rebuilt small | `Goal` entity + "Practise this goal" | 3 (new entity, FFI + schema) | Goal-setting theory robust; apparatus unsupported | Third build of a twice-deleted feature; star untested |
| Metronome + tempo | Resurrect `ClickEngine` into Focus Player | 2 (iOS-only, zero schema) | Measurement claim solid; strategy claims contested | Audio quality bar; already scheduled anyway |
| Chart-to-scaffold residue | Use the shipped flow; close #1106, re-scope #1107 | Docs/sim only | Tradition strong, trials thin | Shipped but never validated in anger |

## The cases in brief

- **Lesson-to-mastery** — the strongest candidate on every axis that has bitten
  this repo before. B1 is the smallest honest slice in the field (read-side
  derivation over data that already exists, no migration). It is
  tracking-shaped, and the repo's pattern is blunt: capture-shaped surfaces
  (lessons #273, goals #769) get deleted, tracking-shaped structure (variants,
  scaffold links) survives. Jon's weekly lesson gives every slice authentic
  use within days, the fastest feedback loop available to an n=1 product. And
  B is a prerequisite investment: whatever wins later, the scheduler needs
  this grain.
- **Space layer** — the right second act. Slice 1 (graded freshness replacing
  the binary 14-day flag) is cheap and honest, but its natural consumer is R's
  reason strings, so shipping it first would build a signal with nowhere good
  to land. The full scheduler is the known-unknown end state; nothing published
  validates any interval formula for repertoire.
- **Goals rebuilt small** — evidence supports exactly the small shape ruled in
  roadmap Q5, but two facts say not yet: the flat priority star has not been
  given its full chance (#764 unshipped, and `specs/priority-items.md`
  explicitly deferred grouping until the flat list proves insufficient), and
  the planning consumer goals feed does not exist. Building the third goals
  feature before either exists would repeat the setup-burden-before-value
  diagnosis that killed the second.
- **Metronome** — ships regardless via audit Phase 4. The note's findings
  (engine recoverable, schema gap smaller than #1366 assumed, analytics slice
  blocked on roadmap Q3) fold into that spec when Stage 3 reaches it.
- **Chart-to-scaffold** — the bookkeeping (#1106 closed, #1107 re-scoped) and
  the validation ride along free: the weekly lesson's tunes are jazz
  standards, so lesson-to-mastery use *is* chart-to-scaffold use. The ladder
  wiring (#1107) becomes a natural follow-on once real use says the generated
  material is good.

## Recommended order — quick wins first, big bets last

*(Reshaped 2026-08-14 on Jon's direction: one sequence across the candidates,
smallest first, each later step gated on use of the one before. The direction
underneath is lesson-to-mastery B-first; the early steps pull the cheap,
already-paid-for wins forward.)*

Do now (days, not weeks):

1. **Chart-to-scaffold bookkeeping** — close #1106 as shipped (#1110),
   re-scope #1107 to the twelve-key ladder residue (#1083 dependency met).
   Minutes.
2. **Use the shipped scaffold flow** on the week's lesson tune — chart entry,
   commit, practise through the builder; file friction as issues. No code;
   this is the validation slice its note asks for.
3. **Metronome slice 1: the click** — resurrect `ClickEngine` (recoverable
   from `8af4891^`) into the Focus Player. iOS-only, zero schema, Tier 2;
   pulled forward from audit Phase 4 because the engine already exists. 1–2
   days.
4. **Tempo capture prefill** — reflection stepper pre-fills from the click and
   shows without a declared target. Small core touch. About a day.
5. **B1 + B2: per-piece tracking** — the (exercise × piece) derivation (core,
   read-only, bridge tests) then the "By piece" UI. 3–5 days total; the first
   step needing the Tier-3 spec discipline.

Then, each building on the last:

6. **R: Up next card** — one dismissible suggested session, reasons in plain
   words, held to suggest-never-gate. Needs B's grain. 2–3 days.
7. **Tempo trend surface** — render the computed `tempo_history`; blocked on
   roadmap Q3 (score vs score-at-tempo) being answered first.
8. **Space slice 1** — the graded "getting cold" signal, landing as Up next
   reason strings rather than a surface of its own. 1–2 days once R exists.

Big bets, last, each a fresh decision gated on lived use:

9. **Space slices 2–3** — the cold shelf in the builder, then persisted
   scheduling state (the real Tier 3 of the Space layer).
10. **A revisited and goals** — the twice-deleted capture-shaped features;
    revisited only if use says the flat star list and a fast add path are
    insufficient.

## Decisions

**1. Adopt the order above** — it takes lesson-to-mastery as the Stage 4.3
direction, pulls the metronome click forward from audit Phase 4, and defers
the capture-shaped builds to the end.

&nbsp;&nbsp;a. Adopt as sequenced (recommended)
&nbsp;&nbsp;b. Adopt but keep the metronome on its audit Phase 4 slot
&nbsp;&nbsp;c. Different direction underneath (Space / goals / ladder first)

**2. Chart-to-scaffold bookkeeping** — step 1 above, decision-independent.

&nbsp;&nbsp;a. Do it now (recommended)
&nbsp;&nbsp;b. Hold until the direction pick
