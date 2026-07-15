# Design brief — Linked exercises

> Companion to [`piece-linked-exercises.md`](piece-linked-exercises.md). For the
> Claude Design pass: only the *new* surfaces, with the real SwiftUI primitives
> to reuse (the Paper & Score system / `Theme.swift` tokens are already loaded).
> Scope: native iOS only.

## What to mock

1. **Piece detail → "Linked exercises" section.** A new `.cardSurface()` card in
   `LibraryDetailScreen`'s detail `VStack`, slotted **after notes/tags, before
   the Delete button**. Pieces only. Three states:
   - **Empty** — section header + a quiet in-card prompt + an "Add exercise"
     button.
   - **Populated** — header + ordered tracked-exercise rows (divided by
     `HairlineDivider`) + an "Add exercise" footer affordance.
   - **Edit / reorder** — rows with drag grips (system edit-mode look).

2. **Tracked-exercise row** — the row inside the section. Contents:
   - Exercise **title** (+ optional key/tempo meta line, like
     `LibraryItemCard`'s `metaLine`).
   - **Score indicator** — the latest practice score (see the ring, below).
   - *Optional* micro-stats (last practised / best tempo) **only if they fit
     cleanly** — title + score is the floor.
   - Tap row → exercise detail. Swipe → unlink. No status pill (dropped).

3. **Multi-select add-exercise picker** (sheet). Searchable list of exercises
   (`kind == Exercise`), already-linked ones excluded/greyed, **checkmark
   multi-select** to add several at once, plus a "Create new" path. Model the
   chrome on the existing `TagFilterSheet`; create-new routes into
   `LibraryAddScreen`.

4. **The 0–10 score ring** (this is issue
   [#1009](https://github.com/jonyardley/intrada/issues/1009)). The one genuinely
   new visual — design it here. A radial ring (fraction = score ÷ 10) with the
   **numeral centred**, **monochrome** (don't recolour by level; the numeral
   carries the meaning — colour-blind-safe, preserving the current `MasteryMeter`
   intent), and an empty "—" state when never practised. It replaces the stepped
   bars on `LibraryItemCard` (library rows) and also appears on the exercise
   detail.

## Reuse these (don't redraw)

| Need | Primitive |
|------|-----------|
| Screen chrome | `ScreenScaffold(title:subtitle:)` |
| Card container | `.cardSurface()` / `CardSurface` |
| Row dividers | `HairlineDivider()` |
| Type signal | `TypeBadge(kind:)`, `ItemKind.bar` |
| Tags | `TagChip(_:style:)`, `TagPills(tags:)` |
| Compact row reference | `SetlistQueueRow` |
| Segmented control (if needed) | `SegmentedPills` |
| Score indicator (→ ring) | `MasteryMeter(level:steps:)` |
| Sheet pattern | `TagFilterSheet` |
| Create-new flow | `LibraryAddScreen` |

## The real decisions to make in the mock

- **The score ring** — its proportions, weight, and how the numeral reads at row
  scale and on library cards. This is the headline new component.
- **Row density** — title + score is the minimum; add last-practised / best-tempo
  only if the row stays clean.
- **Multi-select interaction** in the picker — selection affordance, "add N"
  confirmation.

## Tokens

Likely **no new tokens** — the ring reuses the existing score colours
(`masteryFill` / `masteryTrack`) and standard ink/paper tokens. If anything new
is needed, add it to `Theme.swift` **and** the design reference together.

## Out of scope for this mock (tracked separately)

- 0–10 scoring rescale + overall session score —
  [#1008](https://github.com/jonyardley/intrada/issues/1008).
- Reverse "linked from N pieces" view on an exercise — deferred fast-follow.
