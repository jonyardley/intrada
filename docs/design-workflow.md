# Design in the Intrada workflow

## What lives where

| Artifact | Path | Role |
|----------|------|------|
| **Token source of truth** | `ios/Intrada/DesignSystem/Theme.swift` | Code is canonical. Colours, type, spacing, radius, motion. Nothing overrides this. |
| **Living design system** | `design/intrada-design-system.dc.html` (+ `support.js`) | Editable reference, opened in Claude Design to iterate: the component catalogue and one canonical screen per pillar. **Derived from `Theme.swift`.** |
| **Shareable render** | `design/intrada-design-system.html` | Self-contained, offline, no build. For PR review. Regenerated from the `.dc.html`. |
| **Feature mockups** | `specs/<feature>/design/<journey>.dc.html` | The journeys for one feature: flows, edge cases, transitions. |

The reference visualises the tokens; it never defines new ones.

Design happens in the claude.ai/design project **"Intrada"**
(`claude.ai/design/p/d09e8b80-a0a0-4dee-ab79-670e49210e0f`), which moves into a
Design System artifact before standalone claude.ai/design closes on
14 December 2026 (#2603). **The repo stays canonical**: `DesignSync` pushes
repo design files up after they change and pulls finished mockups down to
`specs/<feature>/design/`, with `support.js` alongside. `design/CLAUDE.md`
stays repo only, since Claude Design refuses that name.

## One home per surface

Drift happens when the same surface is drawn in two files, so every surface has
exactly one home. A **pillar** is a top-level tab that always exists; its
current screen lives once, in the design system. A **journey** is a path
through pillars for one feature; it lives in that feature's mockup, built from
design system components. A feature file never redraws a pillar screen or
restyles a component: it references them. When designs conflict, the design
system wins.

**The fold-in ratchet.** When a feature's design is agreed, promote its new
components to the design system, update any pillar screen that changed shape,
and strip every frame the design system now covers from the feature file.
Things move from feature file to design system, never back, so a feature file
shrinks after every fold-in.

## Motion and dark mode

Named motion tokens live in `Theme.swift` (`IntradaMotion`). Under Reduce
Motion every reveal renders its final state with no animation (#2239). Motion
earns its place only when it carries meaning: progress, a change of state, a
celebration. Light is the default; dark mode is parked until after MVP, and the
"After dark" section of `design/Intrada Concepts.dc.html` shows the tokens
invert cleanly.

## Committing it

1. Copy the `.dc.html`, `support.js` and the `.html` into `design/`, and open
   the `.html` in a browser to confirm it renders.
2. Update `docs/design-principles.md` if the change alters how the app should
   feel, and append a T-numbered decision there rather than deciding silently.

### Regenerating the shareable export

The `.html` is Claude Design's standalone bundle: one
`<script type="__bundler/template">` holding the page as a JSON string, assets
inlined, over DesignSync's 256 KiB `get_file` cap. Two routes:

- **Claude Design's Share, Export, Standalone HTML**, in a browser. Needs an
  interactive claude.ai login.
- **Patch the bundle in place**: decode the template string with `json.loads`,
  apply the edits made to the `.dc.html`, re-encode with
  `json.dumps(..., ensure_ascii=False)`, replace `"</"` with `"<\\u002F"`, and
  splice it back. Decode again and diff against the `.dc.html`: only the
  bundler's normalisations should differ. Then check it in a browser.

Either way, push the `.dc.html` up with `DesignSync` first.

## Adding design to the SDLC

Insert a **Design** step between the spec and the build:

1. **Design.** In Claude Design, open `intrada-design-system.dc.html` for
   context and mock the new screens with the existing kit, in an iPhone frame
   and an Android frame. Save it under `specs/<feature>/design/`. Something new
   belongs in `Theme.swift` and the design system.
2. **Build** against the committed mockup.
3. **PR checklist:**
   - [ ] A changed token changed `Theme.swift` and
         `design/intrada-design-system.dc.html` together.
   - [ ] A changed component changed its page in `design/components/` first.
   - [ ] New screens have a committed mockup, iPhone and Android, under their
         spec folder.

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

- **New component:** its page first, as in "Changing a component".
- **New token:** `Theme.swift` first, then the design system reference and the
  re-exported `.html`, in the same PR.
- **Exploring a direction:** mock it in Claude Design, get sign-off, build it
  in SwiftUI and Compose, then fold the result into the reference.
