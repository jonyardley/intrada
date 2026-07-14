# Design in the Intrada workflow

> Replaces the Pencil (`design/intrada.pen`) step. Design now happens in
> **Claude Design**, and the living reference is checked into the repo as HTML.

## What lives where

| Artifact | Path | Role |
|----------|------|------|
| **Token source of truth** | `ios/Intrada/DesignSystem/Theme.swift` | Code is canonical. Colours, type, spacing, radius. Nothing overrides this. |
| **Living design system** | `design/intrada-design-system.dc.html` (+ `support.js`) | Editable reference — open in Claude Design to iterate. Foundations, type language, component catalogue, screen gallery. **Derived from `Theme.swift`.** |
| **Shareable render** | `design/intrada-design-system.html` | Self-contained, offline, no build. For PR review and quick viewing. Regenerated from the `.dc.html`. |
| ~~Pencil frames~~ | ~~`design/intrada.pen`~~ | Deprecated. Keep for history; do not extend. The `light-mode-exploration.md` record stays as provenance. |

The `.dc.html` is the working file; the `.html` is its exported snapshot. Both
trace back to `Theme.swift` — the reference visualises the tokens, it never
defines new ones.

The Claude Design workspace is the claude.ai/design project **"Intrada"**
(`claude.ai/design/p/cd74e299-f5c1-4915-a603-347db46158d6`). **The repo stays
canonical**: Claude Code bridges the two with `DesignSync` — pushing repo
design files up after they change here, and pulling finished mockups down
(design sessions save to project paths like `mockups/`; they cannot write to
GitHub or the repo). Pulled mockups land under `specs/<feature>/design/` per
the SDLC steps below, with `support.js` copied alongside so they render
standalone.

## Committing it

1. Copy into the repo:
   - `design/intrada-design-system.dc.html`
   - `design/support.js`
   - `design/intrada-design-system.html`
2. Open `design/intrada-design-system.html` in any browser to confirm it renders.
3. Update `specs/design-system.md` and `docs/design-principles.md` to point at the
   new system and name Claude Design as the design tool (retire the Pencil note).

## Adding design to the SDLC

Each feature already moves through `specs/NNN-feature/` (`spec.md` → `plan.md` →
`tasks.md`). Insert a **Design** step between `spec` and `plan`:

1. **Design** — In Claude Design, open `intrada-design-system.dc.html` for context
   and mock the new screens/states using the existing kit. Save the mockup as
   `specs/NNN-feature/design/<screen>.dc.html` (+ exported `.html`).
   - Reuse components and tokens; if something new is needed, that's a signal a
     new primitive/token belongs in `Theme.swift` **and** the design system.
2. **Plan / tasks** — Reference the committed mockup. Build against it.
3. **PR checklist** — add two gates:
   - [ ] If a token or component changed, `Theme.swift` and
         `design/intrada-design-system.dc.html` were updated together.
   - [ ] New screens have a committed Claude Design mockup under their spec folder.

## Keeping code and design in sync

- **New token or component** → change `Theme.swift` first, then update the design
  system reference and re-export the `.html`. Same PR.
- **Exploring a direction** (e.g. the Liquid Glass tab bar) → mock it in Claude
  Design first, get sign-off, then implement in SwiftUI and fold the agreed result
  back into the reference.
- Treat the design system file the way you treat `Theme.swift`: a reviewed,
  versioned part of the codebase — not a throwaway mock.
