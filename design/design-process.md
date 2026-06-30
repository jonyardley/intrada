# Intrada — Design file process & guidelines

> How we keep design files in sync so there is **one canonical view** of how
> Intrada looks and behaves, on every iteration. Read this before starting or
> folding in any feature design. Pairs with `design-workflow.md` (where design
> sits in the SDLC) — this doc is the *file organisation + sync* half.

---

## 1. The core principle: single ownership per concern

Drift happens when the **same surface is drawn in two files**. The moment a
screen exists in both the design system and a feature mock, neither is
authoritative and they silently diverge.

The rule: **every surface has exactly one home.** We never sync copies — we
assign ownership and reference across boundaries.

---

## 2. The three layers (and who owns what)

| Layer | File(s) | Owns | Source of truth for |
|-------|---------|------|---------------------|
| **Tokens** | `ios/Intrada/DesignSystem/Theme.swift` | Colour, type, spacing, radius | Raw values. Code is canonical. |
| **Design system** | `Intrada Design System.dc.html` | The component catalogue **+ one canonical screen per pillar** (Library, Practice, Routines, Progress) | *How it looks.* What the parts are and the rules. |
| **Feature mockups** | `specs/NNN-feature/design/<journey>.dc.html` | The **journeys** for one feature — multi-state flows, edge cases, transitions | *How a journey plays out.* |

**Pillars vs journeys — the distinction that resolves most clashes:**

- A **pillar** is a top-level surface that always exists (the four tabs). Its
  canonical, current-state screen lives in the **design system Screens gallery**,
  one copy.
- A **journey** is a path *through* pillars for a specific feature
  (e.g. "link an exercise → build a session → practise → reflect"). Journeys live
  in **feature files**, assembled from design-system components.

> Yes — have a detailed design page **per journey**, not per screen. One file per
> feature journey keeps related states together and disposable after fold-in. Do
> **not** make a standing "all screens" mega-file; that becomes a second source
> of truth and is exactly what drifts.

---

## 3. The golden rules

1. **Components are defined once, in the design system.** Feature files import/
   reuse them — they never re-style a button, card, header, or ring locally.
2. **A feature file never redraws a canonical (pillar) screen.** If a journey
   passes through the Library, it *references* it ("→ see Design System ·
   Library"), it doesn't paste a copy.
3. **Feature files only contain what's new or in-flight** for that feature: the
   states, pickers, sheets, and transitions that don't yet live anywhere.
4. **When designs conflict, the design system wins.** It is canonical by
   definition. Resolve by updating the DS once, then deleting the divergent copy.

---

## 4. The fold-in ratchet (one-way, every iteration)

When a feature's design is agreed ("the winning design"):

1. **Promote components** → move any new/changed component into the design system
   catalogue (e.g. ScoreRing, the library header, the Related-exercises card,
   the session block). Update `Theme.swift` if a token changed.
2. **Update canonical screens** → if a *pillar* screen changed shape, edit that
   one screen in the DS Screens gallery. (e.g. Library rows swapped bars → ring.)
3. **Strip the duplicate** → in the feature file, delete any frame that is now
   redundant with the DS and replace it with a one-line reference. The feature
   file should shrink after every fold-in, not grow.
4. **Leave only the journey** → the feature file ends up as just its unique
   flow, built from now-canonical parts.

It's a *ratchet*: things move feature-file → design-system, never back. Once a
pattern is canonical, the feature file stops being its source.

---

## 5. Resolving a clash (the playbook)

When you find the same thing designed two ways (e.g. the session-builder screens):

1. **Identify the winner.** Newest agreed design wins; if unsure, ask the
   designer/PM. Write the decision in the feature's `spec.md`.
2. **Make the winner canonical.** If it's a component → DS catalogue. If it's a
   pillar screen → DS Screens. If it's a journey → the feature file.
3. **Delete the loser.** Remove the superseded frames from wherever they now
   conflict. Do not keep "for reference" copies in a second file — reference by
   link instead.
4. **Re-point references.** Any link/anchor to the old version now points at the
   canonical home.

> Applied to the current session-builder clash: the session-builder **journey**
> (add picker → grouped block → swipe/edit removal → start) belongs in the
> feature file; the **SessionBlock component** belongs in the DS (done). Any old
> session/Routines screen the new design supersedes gets updated in the DS once,
> and the stale version deleted — not kept in parallel.

---

## 6. Per-iteration checklist

Run this at the end of every design iteration before sign-off:

- [ ] New/changed components promoted to the design system (and `Theme.swift` if a
      token changed).
- [ ] Any pillar screen that changed shape updated **once** in the DS Screens
      gallery.
- [ ] Every duplicated/superseded frame removed from feature files; replaced with
      a reference line.
- [ ] No surface exists in two files. (Spot-check: search both files for the same
      screen title.)
- [ ] Feature file contains *only* this feature's journey(s).
- [ ] Decisions (winning design, what was retired) recorded in `spec.md`.
- [ ] `CLAUDE.md` "Files" + "Retired" notes updated if a pattern was added/pulled.

---

## 7. Naming & versioning

- **Design system:** one file, always current. `Intrada Design System.dc.html`.
  Bump the `v1.x` footer note when components change; keep the changelog in
  `CLAUDE.md`.
- **Feature mockups:** `specs/NNN-feature/design/<journey-name>.dc.html` — scoped
  to the spec, dated by the spec. Disposable after fold-in (or archived under the
  spec folder, never in the working design root).
- **Exploration / variants:** keep options *within* one file as stacked sections
  (turns), not as forked files, until one wins — then fold the winner in and drop
  the rest.

---

## 8. Quick decision guide

```
Is it a reusable part (button, card, ring, header)?      → Design system · Components
Is it a top-level tab screen in its current state?       → Design system · Screens (one copy)
Is it a multi-state flow for one feature?                → Feature file · journey
Does it already exist somewhere else?                    → Don't redraw — reference it
Did a design just win and replace another?               → Fold winner in, delete the loser
```

When in doubt: **if drawing it would create a second copy of something, stop and
reference instead.**

---

## 9. Sharing one screen across files — the import pattern (Option B)

Linking (Option A) stops drift but still means a screen is *drawn* in one place and
*linked* from another. When a screen genuinely needs to appear, rendered, in more
than one file (e.g. the **Focus player** shows in both the design system and a
feature journey), extract it so there is exactly one definition:

1. Author the screen as its **own** `<Name>.dc.html` in the project root (a full DC
   with its own `<helmet>` — fonts, lucide — and its own logic class for any
   live behaviour, so the behaviour travels with the screen).
2. Mount it wherever it's shown with `<dc-import name="<Name>" hint-size="392px,860px"></dc-import>`.
   `name` is the file basename; the file must be a sibling (same folder) — `dc-import`
   resolves siblings, not sub-paths.
3. Delete every inline copy. Now both places render the same source — zero drift.

**Lessons banked from the first pass (follow these):**
- **Pull from the *latest* source, not the design system's copy.** When a screen was
  reworked in a feature file, *that* is canonical — extract from there, not from an
  older system mockup. (We extracted the wrong Focus player first by reading the DS
  copy.) Confirm the winning source before extracting.
- **A component is documented once.** Don't leave a second example of the same
  component in another group (we had the score shown as both "dots" and a ring) —
  one canonical example in its own section; everywhere else references it.
- **`dc-import` children carry their own icons/JS.** Each imported screen calls
  `lucide.createIcons()` in its own `componentDidMount`; don't rely on the host.
- **Pillar screens that live in only one file don't need extracting** until they're
  shown somewhere else — single-home is already drift-free. Extract on the *second*
  use, not pre-emptively.

### Status of the migration (keep this current)
- ✅ Extracted & imported: **Focus player** (`Focus Player.dc.html`) and **Session summary** (`Session Summary.dc.html`).
- ↪ Linked (Option A): **Item / piece detail** — canonical in `Linked Exercises.dc.html`.
- ◻ Still inline, single-home (extract when shown elsewhere): Library, Practice,
  Routines, Progress, Add piece.
