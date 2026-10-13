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
(`claude.ai/design/p/d09e8b80-a0a0-4dee-ab79-670e49210e0f`), made in Jon's
account on 13 October 2026 from an export of the earlier project
(`1b2cc9dd…`, which lives in another account; `0e60f6f5…` and `cd74e299…`
404). The export was older than `design/`, so the repo's files were pushed up
over it (#2599). Retired coach-era mock-ups sit in the project's `archive/`
folder, and two large standalone exports (Practice Record v0.17 Screens,
Practice Capture Journeys) live only in its `exports/` folder. Claude Design
refuses a file named `CLAUDE.md`, so `design/CLAUDE.md` stays repo only.
Standalone claude.ai/design closes on 14 December 2026, so the project moves
into a Design System artifact before then (#2603). **The repo stays
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
3. Update `docs/design-principles.md` if the change alters how the app should
   feel, and append a T-numbered decision there rather than deciding silently.

### Regenerating the shareable export

`design/intrada-design-system.html` is Claude Design's "standalone HTML" bundle:
one `<script type="__bundler/template">` holding the whole page as a JSON string,
plus every font, icon and image inlined as an asset (hence about 360 KB, well
over DesignSync's 256 KiB `get_file` cap). Two routes:

- **Claude Design's own Share → Export → Standalone HTML**, driven in a browser
  (what #1213 did via Playwright). Needs an interactive claude.ai login, so it is
  unavailable in a headless or non-interactive session.
- **Patch the bundle in place**: decode the template string with
  `json.loads`, apply the same edits made to the `.dc.html`, re-encode with
  `json.dumps(..., ensure_ascii=False)` then `"</" → "<\\u002F"`, and splice it
  back. Verify by decoding again and diffing against the `.dc.html`: the only
  differences should be the bundler's own normalisations (head rewritten with
  inlined fonts, asset URLs replaced by uuids, `viewBox` → `sc-camel-view-box`,
  entities decoded, boolean attributes quoted). Then load the result in a browser
  and check the page height and a new component against the `.dc.html`.

Either way, push the updated `.dc.html` to the project with `DesignSync` first,
so the project and the repo do not drift.

## Adding design to the SDLC

Each feature already moves through `specs/NNN-feature/` (`spec.md` → `plan.md` →
`tasks.md`). Insert a **Design** step between `spec` and `plan`:

1. **Design** — In Claude Design, open `intrada-design-system.dc.html` for context
   and mock the new screens/states using the existing kit, with an iPhone frame
   and an Android frame for each, since every feature ships on both. Save the
   mockup as `specs/NNN-feature/design/<screen>.dc.html` (+ exported `.html`).
   - Reuse components and tokens; if something new is needed, that's a signal a
     new primitive/token belongs in `Theme.swift` **and** the design system.
2. **Plan / tasks** — Reference the committed mockup. Build against it.
3. **PR checklist:** add three gates:
   - [ ] If a token changed, `Theme.swift` and
         `design/intrada-design-system.dc.html` were updated together.
   - [ ] If a component changed, its page in `design/components/` changed
         first (see "Changing a component").
   - [ ] New screens have a committed Claude Design mockup, iPhone and Android
         frames, under their spec folder.

## Changing a component

The component catalogue in `design/components/` is the source for how each
component looks and behaves on both apps (#2620, T37). A change runs in this
order, in one PR:

1. **The page.** Change `design/components/<Name>/README.md`: an agent drafts,
   Jon approves the words. A new component starts from `_template.md` and a
   row in the index.
2. **SwiftUI, then Compose.** Change the iPhone component, then the Android one.
3. **Snapshots on both.** Record the pictures the page names, and look at them.
4. **`just check`.** `scripts/check-components.sh` fails when a page and the
   code disagree on a file, a symbol or a snapshot.
5. **After merge,** republish the changed pages and pictures to the Claude
   Design system. It is a published view, never edited as the source.

## Keeping code and design in sync

- **New component** → its page first, as in "Changing a component" above.
- **New token** → change `Theme.swift` first, then update the design
  system reference and re-export the `.html`. Same PR.
- **Exploring a direction** (e.g. the Liquid Glass tab bar) → mock it in Claude
  Design first, get sign-off, then implement in SwiftUI and Compose and fold the agreed result
  back into the reference.
- Treat the design system file the way you treat `Theme.swift`: a reviewed,
  versioned part of the codebase — not a throwaway mock.
