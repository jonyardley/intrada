# Stage 4.2 — choosing the next direction

*Phase R, Stage 4.2 ([`rethink-plan.md`](../rethink-plan.md)): the one-page
comparison of the five research notes in this folder. **Decided 2026-08-14:
Jon adopted the ordered sequence below.** Feature names follow the
plain-language rule (CLAUDE.md → Conventions); each note's codename appears
once, in brackets, for the git history.*

## What the research changed about the choice

The five notes were commissioned as equals. They did not come back as equals:

- **The chord-chart feature is already built.** Enter a tune's chord changes
  and the app writes its practice exercises — shells, guide-tone lines,
  scales, constrained improvisation (note:
  [`chart-to-scaffold-phase-b.md`](chart-to-scaffold-phase-b.md)). It shipped
  in #1110 and #1111 on 2026-07-17 and froze, unused, behind the coach pivot;
  the issue asking for it (#1106) was simply never closed. The one missing
  part is generating those exercises in all twelve keys (#1107). What remains
  is *using it for real*, not building it.
- **The metronome is already scheduled** as audit Phase 4 (#1366, spec-first),
  so it ships through the Stage 3 pipeline whichever direction won. Its note
  ([`metronome.md`](metronome.md)) mostly de-risks that work: the click engine
  survived deletion intact (recoverable from `8af4891^`), the tempo plumbing
  already spans the core, and tempo-as-a-tracked-measure is genuinely
  unoccupied ground among competitors.
- **The other three are stages of one pipeline, not rivals.** Per-piece
  tracking (#1081) works out how each exercise is going for each tune it
  serves; the Up next card (#1082) is the one place suggestions appear; the
  getting-cold signal ([`space-layer.md`](space-layer.md)) and goals
  ([`goals-rebuilt-small.md`](goals-rebuilt-small.md)) are *inputs to that
  card*, with nowhere to land until it exists. The real question was never
  which one, but where the pipeline starts.

## Side by side

| Candidate (research note) | Smallest step | Size | Evidence | Biggest risk |
|---|---|---|---|---|
| The weekly-lesson loop (`lesson-to-mastery`, #1087) | Show per-piece progress from existing data (#1081) | Days | Strong: unsupervised practice measurably fails without structure | Lesson *entry* is the twice-deleted kind of feature |
| The getting-cold signal (`space-layer`) | Staleness estimate from existing history | Days | Spacing between sessions is solid; no formula is validated for repertoire | The app forming opinions is the road the coach died on |
| Goals, kept small (`goals-rebuilt-small`) | A goal + "Practise this goal" | Weeks: new entity, FFI + schema | Goal-setting theory robust; the old heavy apparatus unsupported | Third build of a twice-deleted feature; the star never got its trial |
| Metronome + tempo (`metronome`) | The click, inside the Focus Player | 1–2 days | Tempo as a measure is solid; practice-strategy claims contested | Musicians hear milliseconds; and it's already scheduled anyway |
| Exercises from chord charts (`chart-to-scaffold-phase-b`) | Use the shipped flow; fix the stale issues | Minutes to hours | Matches how jazz is actually taught; formal trials thin | Built but never used in anger |

## The cases in brief

- **The weekly-lesson loop** — strongest on every axis that has bitten this
  repo before. Its first step is the smallest in the field: no new data
  entry, no new storage, just reading the practice history that already
  exists and slicing it by piece. It is a *tracking* feature, and the
  repo's pattern is blunt: features that track what you did survive; features
  that ask you to fill in admin (lesson entry #273, goals #769) have been
  built and deleted twice. Jon's real weekly lesson means every step gets
  honest use within days.
- **The getting-cold signal** — the right second act. Cheap and honest as a
  first step, but its natural home is the Up next card's reason lines
  ("six days since you played it"), so shipping it first would build a signal
  with nowhere good to land.
- **Goals, kept small** — the evidence supports exactly the small shape ruled
  in roadmap Q5, but two facts say not yet: the priority star has not had its
  full trial (#764 unshipped), and the planning surface goals feed does not
  exist. Building goals a third time before either exists would repeat the
  mistake that killed the second build.
- **Metronome** — ships regardless via the audit backlog; the research folds
  into that spec. The click alone is worth pulling forward: the engine is
  already written and tested, and it ends leaving the app mid-session.
- **Exercises from chord charts** — the fix-the-books and the validation ride
  along free: the weekly lesson's tunes are jazz standards, so lesson use
  *is* chord-chart use.

## The adopted order — quick wins first, big bets last

Each later step is gated on real use of the one before, not on a schedule.

Do now (days, not weeks):

1. **Fix the stale issues** — close #1106 (built and merged in #1110);
   rewrite #1107 down to the one unbuilt part, generating chord-chart
   exercises in all twelve keys (its blocker, exercise steps #1083, has since
   shipped). Minutes. **Done 2026-08-14.**
2. **Use the chord-chart flow** on the week's lesson tune — enter the chart,
   commit the exercises, practise them through the builder; file friction as
   issues. No code; this answers whether the generated material is any good.
   Entry-friction notes feed the chart-entry rethink (#1387): a friendlier
   manual input is its near half, designed once real use has shown where the
   current text format grates. **First real use (2026-08-14, Like Someone in
   Love) filed two:** the parser rejects the Real Book parenthesised-turnaround
   convention (noted on #1387), and the chord chart plus related exercises are
   only discoverable *after* creating the item — adding a piece should capture
   both in one pass (#1390, design-first; rides with this step's findings
   rather than re-ordering the list).
3. **The click** — bring the deleted metronome engine back into the Focus
   Player (recoverable from `8af4891^`). iOS-only, no schema, Tier 2; pulled
   forward from audit Phase 4 because the hard part already exists. 1–2 days.
4. **Tempo capture** — the end-of-item tempo stepper pre-fills from the click
   and shows even when the item has no declared target. Small core touch.
   About a day.
5. **Per-piece tracking (#1081)** — the core derivation (read-only, real
   bridge tests) then the "By piece" screens. 3–5 days; the first step that
   needs the Tier-3 spec discipline.

Then, each building on the last:

6. **The Up next card (#1082)** — one dismissible suggested session with its
   reasons in plain words; held to suggest-never-gate. Needs step 5. 2–3 days.
7. **The tempo trend** — draw the already-computed tempo history per item;
   blocked on roadmap Q3 (does a mastery score mean anything without its
   tempo?) being answered first.
8. **The getting-cold signal** — a graded staleness estimate replacing the
   binary 14-day flag, landing as Up next reason lines rather than a surface
   of its own. 1–2 days once step 6 exists.

Big bets, last, each a fresh decision gated on lived use:

9. **Remembering when to resurface pieces** — the cold shelf in the builder,
   then stored per-item scheduling state (the real Tier 3 of the
   `space-layer` note).
10. **A chart from a photo (#1387)** — photograph a lead sheet and the app
    reads the changes into a chart for review, entirely on-device.
    Feasibility spike first (OCR + the existing parser vs an on-device
    model); changes only, never melodies.
11. **Quick lesson entry (#1080) and goals** — the twice-deleted
    admin-shaped features; revisited only if use says the star list and a
    fast add path are not enough.

## Decisions (Jon, 2026-08-14)

- Adopt the order above; the direction underneath is the weekly-lesson loop.
- Fix the stale chord-chart issues immediately (done: #1106 closed, #1107
  re-scoped).
- Plain language in docs and issues from here on: rule in CLAUDE.md →
  Conventions, glossary in [`../reference.md`](../reference.md); older docs
  renamed as touched.
