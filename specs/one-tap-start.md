# One-tap start: today's plan on the Practice hero

> Tier 3 (a bridge shape and a new session event). Rides as the first commit
> of the core PR for #57 and #999, parts of #1974 and #1978. Native iOS only.

## Problem

The Practice hero suggests one block (a piece and up to two related
exercises) and its Start button opens the builder with that block in it. So
the shortest path from opening the app to playing is two taps and a glance at
a screen full of controls, and the block ignores how long the musician said
they have (#1736): someone with a preferred hour sees a fifteen minute
suggestion. The hero also cannot say what today's plan is (#999), because
there is no plan, only a block.

## Approach: a plan of blocks, filled to the preferred length

The core ranks up to four candidate blocks the way it already ranks the one
Up next block, then fills a plan from them to the preferred session length.
The hero shows the plan; Start seeds it and starts playing in one event;
"Change it first" seeds the same plan into the builder.

### Data model (core)

```rust
pub struct SuggestedPlan {
    pub blocks: Vec<SuggestedSession>, // at least one; the first leads
    pub estimated_minutes: u32,        // the sum of the blocks' estimates
    pub item_count: u32,
    pub length_mins: Option<u16>,      // the preference it was filled to
}
```

`SuggestedSession` keeps its name and shape: it is one block.
`ViewModel.up_next` becomes `Option<SuggestedPlan>`.

### Ranking and filling

- `suggestion::rank_blocks(items, clock)` returns up to four blocks, anchors
  in the existing Up next order. An exercise already in an earlier block is
  skipped in a later one, so blocks never share an item; each block's
  estimate counts only its own items.
- `suggestion::plan(blocks, length_mins)` takes the first block always. With
  a length, it then adds blocks in order while the running estimate stays
  within the length plus five minutes, and stops at the first that does not
  fit, so a lower-ranked short block never jumps a higher-ranked long one.
  Without a length it stops after the first block: it never assumes a length.
- The ranking is cached with the other projections (its inputs are the
  library and the clock); the fill runs at view time, so changing the
  preference changes the hero on the next render without a cache key.

### Events

- `SessionEvent::StartFromSuggestion { now }`: from Idle, seeds the plan into
  a build (each block its own group, today's length from the preference) and
  starts the session at `now` through the existing start path. With nothing
  to suggest it is a no-op, as the builder CTA is: the button cannot be on
  screen then, and a race must not strand anything.
- `SessionEvent::StartBuildingFromSuggestion { now }` seeds the whole plan,
  not just the first block, and stays in the builder.
- Both re-derive the plan from the model at `now`, never from the shell's
  copy (#1082).

### Crash recovery

`StartFromSuggestion` ends in `start_session`, so the `ActiveSession` it
builds has the same fields as any other. Nothing new enters the blob graph
and `BLOB_VERSION` stays as it is.

## Screens (the second PR)

- The hero's eyebrow reads "Today's plan" when the plan was filled to a
  length, otherwise "Up next" as now; the count line beside it carries the
  plan's item count and minutes, so the eyebrow repeats neither (T32).
- The lead block keeps its piece title, reason and rows; later blocks show as
  one line each: piece title, item count and minutes, so the total adds up on
  screen (T32).
- Start sends `StartFromSuggestion`; its haptic fires only when a session
  became active, which closes #1413 for this path.
- "Change it first" sends `StartBuildingFromSuggestion`; "Build my own
  instead" is unchanged.

## Key decisions

1. **No routines** (#57 plan comment, decision 1). The plan seeds from Up
   next only; routines (#1348) can feed it later.
2. **Start plays** (decision 2). Changing the plan is the second path.
3. **Blocks, in order, stop at the first misfit** (decision 3). Four blocks
   at most, five minutes of slack.
4. **No named character** on the plan line (decision 4).

## Deliberately not doing

- Routines, spacing (#55), topping up with loose items.
- Planned durations on the seeded entries.
- Storing the plan on the finished session.

## Testing

- **Core, test-first:** no length gives one block; a 30 minute length fills
  and stops within 35; a first block over the length is still the plan; the
  fill stops at the first misfit rather than skipping it; four blocks at most;
  a later block never repeats an earlier block's exercise; the plan's minutes
  and count are the blocks' sums; `StartFromSuggestion` leaves an active
  session holding every planned entry grouped by block, and is refused
  outside Idle and a no-op with nothing to suggest;
  `StartBuildingFromSuggestion` seeds every block; the view's plan follows a
  change to the preference; events round-trip on the bincode wire (#846).
- **iOS, core PR:** the preview fixtures and hero compile against the plan;
  a live bridge test starts a session from the plan in one event.
- **iOS, screens PR:** snapshots of the hero with a one-block plan and a
  filled plan.
- Bindings regenerated with `just ios-gen`, never hand-edited.
