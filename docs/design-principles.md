# intrada Design Principles

> Living document. Started 2026-05-31. These are **guiding principles**, not
> hard gates — they set direction and frame decisions. When a principle pulls
> against another (or against what the app does today), that's a decision to
> make deliberately, not a rule to mechanically enforce. Document those
> decisions in the **Open tensions & decisions** log at the bottom.
>
> Visual/token detail lives in code (`crates/intrada-web/style/input.css`,
> `views/design_catalogue.rs`) and the design rules in `CLAUDE.md`. This doc is
> the *why* and the *interaction* layer those don't cover.
>
> Design is now produced in **Claude Design** — see
> [`design-workflow.md`](design-workflow.md). The native iOS design system lives
> at [`design/intrada-design-system.dc.html`](../design/intrada-design-system.dc.html),
> derived from `ios/Intrada/DesignSystem/Theme.swift`. (Pencil is retired.)

## How to use this doc

- Designing a new surface? Read the interaction principles first, then check
  the open-tensions log for anything relevant.
- Principles conflict? Don't silently pick one. Surface it, decide, and add a
  dated entry to the log so the next person inherits the reasoning, not just
  the outcome.

---

## Visual principles

These are mostly settled and live in code; captured here so the *intent* behind
them isn't lost. (See `CLAUDE.md` → Design System Rules for the enforcement
rules and primitive catalogue.)

- **Dark-on-dark glassmorphism.** Neutral gray-900→near-black gradient,
  whisper-soft white-opacity surfaces (3/5/12%). Backdrop-blur on *chrome only*
  (header, tab bar, overlays) — never on content surfaces.
- **Type is colour-coded.** Gold = Piece, blue = Exercise, teal = Set. The
  mapping repeats across gradient bars, badges, and inline indicators. Colour is
  an *accelerator* for recognition — never the only signal (see accessibility
  tension below).
- **Two accent families.** Warm indigo = interactive/primary; gold/amber = warm
  accent for achievements, streaks, progress.
- **Warmth bias in semantics.** Danger is warm coral, not pure red; success is
  warm teal-green. The palette leans warm even where convention is harsh.
- **Serif headings, sans body.** Source Serif 4 for page titles (signals a
  musical, editorial space); InterVariable for everything else.
- **Reuse before creating; extend, don't clone.** Hand-rolled markup that
  duplicates a primitive is the top source of visual drift.

---

## Interaction principles

The app's job is to get the user *practising*, not to make them operate
software. Every screen should feel like the shortest honest path to the thing
they came to do — except at the two moments where stopping to think *is* the
thing they came to do.

### A. Spend friction deliberately

- **Friction is a tool, not a defect.** There are two kinds, and they get
  opposite treatment:
  - *Bad friction* — admin, setup, planning, navigation, configuration. Remove
    it ruthlessly. The path to practising should be near-frictionless, intuitive,
    and never overwhelming.
  - *Good friction* — setting an intention just before a practice item, and
    reflecting in the moment right after each item. This is where the product's
    value lives. Preserve it; sometimes deliberately add it.

  The goal is not "fewest steps" everywhere. It's *no wasted steps in the admin,
  and deliberate, well-placed steps where they create meaning.*
- **Keep admin & setup flows short.** Adding a piece, building a routine, saving
  a set, navigating the library — these should feel near-frictionless. If one
  grows steps, treat it as a signal to rethink, not to accept.
- **One primary action per screen.** Each screen has one obvious next step,
  visually dominant (hero CTA / circular button). Secondary actions recede. Two
  actions competing to be "the" action means the screen is ambiguous.
- **Defaults beat configuration.** Every flow should be completable with zero
  setup. Sensible defaults up front; tuning is available but never required to
  proceed.
- **Direct manipulation where the platform allows.** Prefer acting on the thing
  itself (swipe, drag, tap-to-toggle) over opening a form. (Platform caveat in
  tensions log — web needs visible affordances.)

### B. Simplicity — defer complexity, don't remove capability

- **Progressive disclosure.** Show the common path first; advanced options live
  behind a sheet, "More", or Settings. The 80% case shouldn't pay the UI cost of
  the 20% case.
- **Reversible by default; confirm only the irreversible.** Reversible actions
  are instant, optimistic, and undoable (toast). Reserve confirmation dialogs for
  genuinely destructive, non-recoverable actions.
- **The app disappears during practice.** Active practice is the reason the app
  exists. Strip non-essential chrome (focus mode). The user operates the music,
  not the interface.

### C. No clutter — easy to reason about

- **Content over chrome.** Maximise the share of the screen showing the user's
  own data (pieces, sessions, progress) versus navigation and decoration.
- **Glanceable, not just readable.** A screen should be parseable at a glance.
  Lean on the type-colour system and consistent layout so users decode by
  pattern, not by reading every label.
- **Consistency is a simplicity tool.** The same gesture means the same thing
  everywhere; every list→detail behaves the same. Novelty in interaction is a
  cost the user pays, not a feature.
- **One concept per screen.** A screen answers one question or completes one
  task. The urge to add a second unrelated section usually means a second screen
  (or a sheet).

---

## Open tensions & decisions

The principles above pull against each other and against today's app in real
places. Each entry: the conflict, the options, and the decision (or `OPEN`).

### T1 — Where friction belongs: admin vs intention/reflection
**Status:** DECIDED 2026-05-31.
Resolved by the *spend friction deliberately* principle (§A). Setup, planning,
and admin are **bad friction** — make them near-frictionless and unoverwhelming.
But there is **good friction** we deliberately keep: setting an intention just
before each practice item, and reflecting in the moment right after it. A
pre-start review/intention step is therefore not "an extra step to eliminate" —
it's the good kind, *provided* it sits at the intention moment and isn't padded
with admin. Open follow-on: keep the *resume-a-known-routine* path short (don't
force the full builder when the user just wants to continue) — distinct from the
intention beat, which stays. Tracked: jonyardley/intrada#760.

### T2 — Mid-session configurability vs "the app disappears during practice"
**Status:** DECIDED 2026-05-31.
Mid-session editing of duration/reps/focus stays **one layer down** — revealed by
a deliberate gesture (e.g. tap-to-reveal), never persistent chrome. The default
live-session screen stays bare so focus mode holds. Config is reachable, not
resident.

### T3 — Decode-by-colour vs accessibility
**Status:** DECIDED (revisit) 2026-05-31.
**Colour is always an accelerator, never the sole carrier of meaning.** Every
type signal must also carry a shape or text cue (dot, badge text, icon). Today's
`InlineTypeIndicator` (dot + colour) and `TypeBadge` (text) already satisfy this.
Marked for possible revisit, but the rule holds for now.

### T4 — Direct manipulation vs one UI codebase (web + iOS)
**Status:** DECIDED 2026-05-31.
The principle is **"the most direct affordance the platform offers"**, with one
hard floor: **no action may be reachable *only* through a hidden gesture.** iOS
gestures (swipe-to-delete, long-press menu) are accelerators layered on top of a
visible, non-gesture path that exists on every platform (e.g. the action lives in
the detail screen / sheet too). Web users always get a visible affordance.
Follow-on: spot-check that every gesture-only action today also has a non-gesture
path on web (rolls into the T6 audit, jonyardley/intrada#761).

### T5 — Content over chrome vs the 2026 visual identity
**Status:** DECIDED 2026-05-31.
Chrome earns its space on **navigation/structural** surfaces (tab bar, headers,
the brand backdrop). On **content** surfaces, every visual element must serve
recognition, hierarchy, or navigation — not pure ornament. The type gradient
bars stay because they *encode type* (functional, not decorative). Test before
adding a visual flourish to a content screen: *does this help the user decode or
navigate?* If it's ornament only, cut it.

### T6 — Reversible-by-default vs today's confirm sheets
**Status:** DECIDED (principle) + audit pending, 2026-05-31.
Rule adopted: **recoverable actions → optimistic + undo toast; only genuinely
irreversible actions → confirm dialog.** New destructive actions default to the
undo-toast path unless the data loss is unrecoverable. Existing destructive
actions (e.g. item delete's confirm sheet) need a per-action audit to reclassify
— tracked as a follow-on issue.
