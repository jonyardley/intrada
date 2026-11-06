# Design in the Intrada workflow

Every new screen is designed once, signed off by Jon once, and built on both
apps against that one mockup. Mockups and the design system live in Claude
artifacts; the repo stays the source of truth.

Until #2637 lands (due before 14 December 2026, when standalone
claude.ai/design closes), the old design system page stays in `design/` only
so `scripts/check-design-tokens.sh` can read it. Nothing else here uses it.

## Who owns what

| Thing | Lives in | Why there |
|-------|----------|-----------|
| **Tokens** | `ios/Intrada/DesignSystem/Theme.swift`, with Android's `Theme.kt` checked against it | One source both apps and the design system read (#2603). |
| **Components** | `design/components/<Name>/README.md`, with snapshots from both apps | `just check` fails when a page and the code disagree (T37, #2620). |
| **A screen being designed** | Its Design canvas, copied into `specs/<feature>/design/` | A build session finds it from the issue without asking. |
| **A screen that has shipped** | The apps and their snapshot tests | A mockup goes stale the day the build merges. |
| **The design system** | The [Design System artifact](https://claude.ai/artifact/BRrktWYfTdnpj8sREEBsSs) | A view to draw from, republished after merge and never edited there, so it cannot drift from the repo. |

## The route

1. **Plan comment.** A UI issue's plan comment names the screens that need a
   mockup. A screen made only of catalogue components on an existing pattern
   can go straight to the build; the plan says so.
2. **Mockup.** A Design canvas drawn from the Intrada design system, with an
   iPhone frame and an Android frame for each screen. Jon starts one in
   claude.ai; a session starts one with the Artifact tool's Design type.
   `DesignSync` only moves files and cannot make a mockup.
3. **Jon's word on the mockup.** The canvas link goes on the issue with the
   line "Waiting for Jon's word on the mockup". A yes becomes "Jon approved the
   mockup on YYYY-MM-DD"; a no is a revision of the same canvas, never a new
   one.
4. **Save it.** The build PR's first commit copies the canvas's `project/`
   files into `specs/<feature>/design/`, so the build and the review read the
   same picture.
5. **Build.** A new or changed component starts with its page (below), then
   SwiftUI, then Compose, with snapshots on both.
6. **Jon's word on the screenshot.** "Eyes before ready" in
   `.claude/rules/ios-ui.md`.
7. **After merge.** Republish the changed component pages and tokens to the
   design system; do a bare `read` of the artifact first, or the publish is
   refused. Delete the feature's mockup once the snapshots cover its screens,
   so a stale picture never outlives the code.

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

A new token goes into `Theme.swift` and `Theme.kt` in the same PR, before any
screen uses it.

## Explorations

Work on the app's overall look that is not one feature lives in
`design/explorations/`. It is never a mockup to build against: a direction Jon
chooses becomes tokens and component pages first, then screens.

## Motion and dark mode

Named motion tokens live in `Theme.swift` (`IntradaMotion`). Under Reduce
Motion every reveal renders its final state with no animation (#2239). Motion
earns its place only when it carries meaning: progress, a change of state, a
celebration. Light is the default; dark mode is parked until after MVP, and the
"After dark" section of `design/explorations/Intrada Concepts.dc.html` shows
the tokens invert cleanly.
