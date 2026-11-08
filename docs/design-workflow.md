# Design in the Intrada workflow

Every new screen is sketched once in Claude Design, signed off by Jon once,
and built on both apps from the catalogue components against that one mockup.
After merge, the design system is refreshed from the repo. Mockups and the
design system live in Claude artifacts; the repo stays the source of truth.

Until #2637 lands (due before 14 December 2026, when standalone
claude.ai/design closes), the old design system page stays in `design/` only
so `scripts/check-design-tokens.sh` can read it. Nothing else here uses it.

## Who owns what

| Thing | Lives in | Why there |
|-------|----------|-----------|
| **Tokens** | `ios/Intrada/DesignSystem/Theme.swift`, with Android's `Theme.kt` checked against it | One source both apps and the design system read (#2603). |
| **Components** | `design/components/<Name>/README.md`, with snapshots from both apps | `just check` fails when a page and the code disagree (T37, #2620). |
| **The sketch kit** | `design/sketch-kit/`: tokens, icons, phone frames and web copies of five components | Mockups draw on the real look; the tokens and icons are made from the code by `just sketch-kit` (T38). |
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
   `DesignSync` only moves files and cannot make a mockup. Draw it from the
   sketch kit: `ScreenScaffold` with `device` set gives each phone frame and
   its tab bar, and the tokens give every other colour, size and gap. A part
   the kit lacks is drawn from tokens and named after the catalogue component
   it stands for, so the build knows what to reach for.
3. **Jon's word on the mockup.** The canvas link goes on the issue with the
   line "Waiting for Jon's word on the mockup". A yes becomes "Jon approved the
   mockup on YYYY-MM-DD"; a no is a revision of the same canvas, never a new
   one.
4. **Save it.** The build PR's first commit copies the canvas's `project/`
   files into `specs/<feature>/design/`, so the build and the review read the
   same picture.
5. **Build.** From the catalogue components and theme values only;
   `scripts/check-theme-values.sh` fails a typed colour or spacing number. A
   new or changed component starts with its page (below), then SwiftUI, then
   Compose, with snapshots on both.
6. **Jon's word on the screenshot.** "Eyes before ready" in
   `.claude/rules/ios-ui.md`.
7. **After merge.** Refresh the design system (below). Delete the feature's
   mockup once the snapshots cover its screens, so a stale picture never
   outlives the code.

## Refreshing the design system

Run this after any merge that changes a component page, `Theme.swift`, the
Android icons or the sketch kit. The design system is published from `main`
and never edited by hand in Claude Design outside a refresh.

1. **Use the committed kit.** The refresh publishes the `design/sketch-kit/`
   files as committed on `main`; nothing is regenerated. `just check` already
   fails a branch whose kit is stale, so those files match the code.
2. **Read the live copy.** Use the Artifact tool's `read` on the
   [design system](https://claude.ai/artifact/BRrktWYfTdnpj8sREEBsSs), then
   `read` `project/design-system.json` and each file you will replace. A
   publish to an artifact this session has not read is refused.
3. **Copy into one folder** in your scratchpad, at the design system's paths:
   - `design/components/<Name>/README.md` to `project/components/<Name>/README.md`,
     reshaped: drop the front matter, turn relative links into GitHub links on
     `main`, and add an "In the repo" section in its place. The published
     page has no repo around it, so relative links break;
   - `design/sketch-kit/tokens.json` to `project/tokens.json`;
   - `design/sketch-kit/components/` (`bundle.js`, `bundle.css`, `tokens.css`,
     `index.d.ts`, and each `<Name>/preview.html`) to `project/components/`.
     Without `tokens.css` a canvas gets unstyled components;
   - never the kit's `icons/` folder: `bundle.js` carries the icons;
   - the fonts live in the design system at `project/fonts/`, copied from
     `ios/Intrada/Resources/Fonts/` and republished only when the font files
     change. `tokens.css` points its font faces there;
   - each new or changed snapshot, uploaded as an asset first, with its
     record added under `assetGroups.Snapshots` in the index.

   A new component page needs a `preview.html` with a `@dsCard` group, or the
   gallery shows no card for it, and a row in the design system's
   `project/README.md` and `project/assets/Snapshots/README.md`. None of these
   has a source in the repo, so write them in the copy you read in step 2. The
   kit's five components are the exception: their `preview.html` lives in
   `design/sketch-kit/components/<Name>/`. Every other page's preview lives
   only in the design system.
4. **Publish once** with `url` set to the design system, `root` set to that
   folder, and `files` naming only the paths you changed. Send `index.d.ts`
   with `contentType` `text/plain`: a `.ts` file is refused. Send
   `project/design-system.json` last and only when its own keys change: the
   `libraries` (React 18 for the kit), an asset record, and `lastChange` with
   who, when and the `main` commit.
5. **Check it.** Open the design system and look at each changed page and
   preview.

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
screen uses it, with its usage note in `design/sketch-kit/token-notes.json`
and a run of `just sketch-kit`. A change to one of the kit's five components
changes its web copy in `design/sketch-kit/components/` in the same PR.

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
