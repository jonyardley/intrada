# Design in the Intrada workflow

> Replaces the Pencil (`design/intrada.pen`) step. Design now happens in
> **Claude Design**, and the living reference is checked into the repo as HTML.

## What lives where

| Artifact | Path | Role |
|----------|------|------|
| **Token source of truth** | `ios/Intrada/DesignSystem/Theme.swift` | Code is canonical. Colours, type, spacing, radius. Nothing overrides this. |
| **Living design system** | `design/intrada-design-system.dc.html` (+ `support.js`) | Editable reference — open in Claude Design to iterate. Foundations, type language, component catalogue, screen gallery. **Derived from `Theme.swift`.** |
| **Shareable render** | `design/intrada-design-system.html` | Self-contained, offline, no build. For PR review and quick viewing. Exported from the `.dc.html`. |
| ~~Pencil frames~~ | ~~`design/intrada.pen`~~ | Deprecated and removed. The `light-mode-exploration.md` record stays as provenance. |

The `.dc.html` is the working file; the `.html` is its exported snapshot. Both
trace back to `Theme.swift` — the reference visualises the tokens, it never
defines new ones.

## Committing it

1. Sync the editable files from the Claude Design project into the repo:
   - `design/intrada-design-system.dc.html`
   - `design/support.js`
2. **Export** the shareable render from Claude Design (Download / Export) and
   save it as `design/intrada-design-system.html`. Export it from the UI rather
   than the sync API — the self-contained file inlines React + fonts and exceeds
   the API's 256 KiB read cap, which silently truncates it.
3. Open `design/intrada-design-system.html` in any browser to confirm it renders.
4. Keep `specs/design-system.md` and `docs/design-principles.md` pointing at this
   system, with Claude Design named as the design tool.

## Adding design to the SDLC

The tier system in `CLAUDE.md` governs ceremony. For any **UI** work (Tier 2+),
insert a **Design** step before Plan mode — replacing the old "Pencil design
first" step:

1. **Design** — In Claude Design, open `intrada-design-system.dc.html` for context
   and mock the new screens/states using the existing kit. Save the mockup
   alongside the work (e.g. `specs/<feature>/design/<screen>.dc.html`, or attach
   the exported `.html` to the PR for a Tier 2 change with no spec folder).
   - Reuse components and tokens; if something new is needed, that's a signal a
     new primitive/token belongs in `Theme.swift` **and** the design system.
2. **Plan / tasks** — Reference the committed mockup. Build against it.
3. **PR checklist** — add two gates:
   - [ ] If a token or component changed, `Theme.swift` and
         `design/intrada-design-system.dc.html` were updated together (and the
         `.html` re-exported).
   - [ ] New screens have a committed Claude Design mockup (under the spec folder,
         or linked from the PR).

## Keeping code and design in sync

- **New token or component** → change `Theme.swift` first, then update the design
  system reference and re-export the `.html`. Same PR.
- **Exploring a direction** (e.g. the Liquid Glass tab bar) → mock it in Claude
  Design first, get sign-off, then implement in SwiftUI and fold the agreed result
  back into the reference.
- Treat the design system file the way you treat `Theme.swift`: a reviewed,
  versioned part of the codebase — not a throwaway mock.
