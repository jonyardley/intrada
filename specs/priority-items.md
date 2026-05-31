# Priority Items — replacing Goals in the Plan layer

> Tier 3 (rips out a core entity + DB schema). Spec rides with the Phase A
> branch per CLAUDE.md. Supersedes `specs/goals.md` and resolves roadmap
> open question #5 ("Goals redesign").

## Problem

Goals were meant to answer "what should I work on, and why?" — the Plan
layer's reason to exist. In practice the feature became too heavy *from a
user's perspective*:

- **Setup burden before value.** To get anything back you had to create a
  goal, link items, and set targets — admin before you're allowed to just
  practise.
- **Conceptual overload.** Goals stacked goals + linked items + per-item
  confidence *and* tempo targets + deadlines + completion + photos. The
  user has to hold a data model in their head.

The underlying job is real — a musician benefits from a sense of direction
before the instrument comes out — but it should be **nearly free to
express, not a form to fill in**. Goals are already parked behind a feature
flag and flagged in the roadmap for a "ground-up rethink"; this spec is
that rethink.

## Approach: direction is a flag on items, not an entity

"Direction" becomes a **priority flag on library items you already own**.
The Plan surface becomes a *view over the library*, not a thing you create
and manage. This dissolves both complaints at once: nothing to set up (you
flag a piece you already have), and no new data model (one boolean on an
existing thing).

### Data model

One field on the existing `Item`:

```rust
pub priority: bool,   // "I'm working on this"
```

A column on the existing `items` table; one field on `Item` / `ItemView`.
Binary on/off — no levels, no manual ordering. (Ordering can come later
*only if* flagging many items proves too flat. YAGNI until then.)

### What gets deleted

The entire heavy goal surface:

- `Goal`, `GoalStatus`, `GoalItem`, `GoalPhoto` types + the
  `goals` / `goal_items` / `goal_photos` tables.
- All goal events (`Add/Update/Complete/Reopen/Delete/LinkItem/
  UnlinkItem/UpdateGoalItemTargets`).
- Goal views: list, detail, create form, edit form, item-targets sheet,
  link-items sheet.
- `/goals/*` routes and the **Goals tab** in the bottom bar.
- Per-item **targets** (confidence + tempo), **deadlines**,
  **completion/reopen**, **goal photos**, the auto-suggest "looks ready"
  banner.
- The `goals` feature flag (the flag *framework* stays — it's freshly
  built and useful — only the `goals` flag is removed).

### What gets salvaged

The two genuinely good things buried inside goals, re-pointed at the
priority set:

- **Least-ready-first ordering** (from "Practise this goal") → orders the
  priority items for a session.
- **Per-item progress derivation** (`latest_score`, `latest_achieved_tempo`
  over session data) → shows how a priority item is going. Already a
  view-model computation; stays as-is.

## How priority wires into the loop

The flag must do more than sort, or it doesn't pull its weight. Three
touchpoints:

- **Plan** — Library becomes the default landing surface (Goals tab is
  gone). Priority items get a pinned, collapsible "Priorities" section at
  the top of the library (or a Priority filter in `LibraryTypeTabs` on
  longer libraries). No new screen.
- **Practice** — A one-tap **"Practise your priorities"** loads all flagged
  items into the session builder, least-ready-first. Replaces "Practise
  this goal" one-to-one.
- **Track** — Exactly one new signal: **"a priority piece you haven't
  practised in N days"** — a neglect nudge linking straight back into a
  session. Deliberately scoped to one signal; the broader analytics rethink
  is separate.

### Naming

**"Priority"**, shown as a **star** on each item. "Focus" was rejected — it
collides with Practice's Focus mode.

## Key decisions

1. **No entity.** Direction is a property of items, not a container. This
   is the whole point — it removes the create/manage lifecycle.
2. **Rip out, don't migrate.** Goals are behind a feature flag with no
   meaningful production data, so we delete the tables rather than convert
   goals → priority flags. (If real goal data turns up on a dev account, a
   one-shot "flag all linked items as priority, then drop" migration is the
   fallback.)
3. **Library is the front door.** With Goals gone, Library is the default
   landing surface. Plan = a well-organised library with a priority view on
   top.
4. **Ship without its own flag.** Priority is small and safe; it ships
   directly. The feature-flag framework stays for future use.

## Deliberately not doing (v1)

- **Deadlines / "by when".** The old goals had target dates + overdue
  badges; priority has no time concept. A musician prepping for a dated
  exam loses the "due" framing. Accepted as the cost of killing the
  ceremony — revisit only if it bites.
- **Targets** (confidence/tempo as goals). Progress is still *derived* and
  shown, but the user no longer sets targets to hit.
- **Grouping priorities into named pursuits** (the "light container" idea).
  Explicitly deferred; only revisit if a flat priority list proves
  insufficient.

## Open questions

- Pinned section vs. filter for the priority view — decide during UI build
  against a realistic library length.
- The "N days" threshold for the neglect nudge — pick a sensible default
  (e.g. 7) and make it a constant, not a setting.

## Rollout

- **Phase A** — core + API + DB: add `priority` to `Item`, the toggle
  event, the column migration; delete the goal tables/types/events/routes;
  remove the `goals` flag. Spec rides on this branch.
- **Phase B** — web/iOS UI: star toggle on items, the priority view in
  Library, "Practise your priorities", Library-as-landing.
- **Phase C** — the one Track neglect signal.

## Testing

- **Core (TDD):** the `priority` toggle event; the priority-set ordering
  (reuse least-ready-first tests).
- **API:** the column migration; the toggle endpoint with auth-rejection +
  cross-user isolation.
- **Web/iOS:** preview-driven verification of the star toggle, the pinned
  priority section, and "Practise your priorities".
