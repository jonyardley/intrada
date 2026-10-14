# One-tap start: today's plan on the Practice hero

> Tier 3. Issues #57 and #999.

The core ranks up to four candidate blocks the way it ranks the Up next block,
then fills a plan from them to the preferred session length
(`SuggestedPlan`, `suggestion::plan`). Start seeds the plan and starts playing
in one event (`SessionEvent::StartFromSuggestion { now }`); "Change it first"
seeds the same plan into the builder. Both re-derive the plan from the model,
never from the shell's copy. Nothing new enters the crash-recovery blob.

## Key decisions

1. **No routines** (#57 plan comment, decision 1). The plan seeds from Up
   next only; routines (#1348) can feed it later.
2. **Start plays** (decision 2). Changing the plan is the second path.
3. **Blocks, in order, stop at the first misfit** (decision 3). Four blocks
   at most, five minutes of slack. Without a preferred length the plan is the
   first block alone.
4. **No named character** on the plan line (decision 4).

## Deliberately not doing

- Spacing (#55), topping up with loose items.
- Planned durations on the seeded entries.
- Storing the plan on the finished session.
