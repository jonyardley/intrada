# Design in the Intrada workflow

Every new screen starts from a design brief, is explored on a Claude Design
canvas, sketched in SwiftUI from the real components, reviewed by a fresh
design reviewer, signed off by Jon on that sketch, and built from it.
After merge, the design system is refreshed from the repo. Canvases and the
design system live in Claude artifacts; the repo stays the source of truth.

## Who owns what

| Thing | Lives in | Why there |
|-------|----------|-----------|
| **Tokens** | `ios/Intrada/DesignSystem/Theme.swift` | One source the app and the design system read (#2603). |
| **Components** | `design/components/<Name>/README.md`, with snapshots from the app | `just check` fails when a page and the code disagree (T37, #2620). |
| **The sketch kit** | `design/sketch-kit/`: tokens, icons, phone frames and web copies of five components | Mockups draw on the real look; the tokens and icons are made from the code by `just sketch-kit` (T38). |
| **A screen being designed** | Its Design canvas for the options, then its SwiftUI sketch on a branch, both linked from the issue | A build session finds it from the issue without asking. |
| **A screen that has shipped** | The app and its snapshot tests | A mockup goes stale the day the build merges. |
| **The design system** | The [Design System artifact](https://claude.ai/artifact/BRrktWYfTdnpj8sREEBsSs) | A view to draw from, republished after merge and never edited there, so it cannot drift from the repo. |

## The route

1. **Plan comment.** A UI issue's plan comment names the screens that need a
   sketch. A screen made only of catalogue components on an existing pattern
   can go straight to the build; the plan says so.
2. **Design brief.** Before anything is drawn, a few lines on the issue under
   the heading "Design brief": what the musician is doing at that moment
   (often at the instrument, hands busy, between plays), what the screen or
   control is for, the existing screens it sits beside, the design principles
   in play, and how it should feel. Every later step and the design reviewer
   read it, since a sketch can pass every token check and still be wrong for
   the moment it is used in (#3018).
3. **Explore on canvas.** Rough options side by side on a Design canvas, an
   iPhone frame for each.
   Jon starts one in claude.ai; a session starts one with the Artifact tool's
   Design type. `DesignSync` only moves files and cannot make a mockup. Draw
   it from the sketch kit: `ScreenScaffold` with `device` set gives each phone
   frame and its tab bar, and the tokens give every other colour, size and
   gap. A part the kit lacks is drawn from tokens and named after the
   catalogue component it stands for. Jon picks an option; the canvas is for
   choosing, not for sign off. A screen whose shape is already settled skips
   this step.
4. **Sketch it in SwiftUI.** The chosen option is drawn, in two or three
   variations, from the real catalogue
   components in one debug only file, `ios/Intrada/Sketches/<Name>Sketch.swift`,
   wrapped in `#if DEBUG` because the app target takes in the whole folder,
   with fixture data and no core changes, plus one snapshot test that renders
   it, at the top level as `ios/IntradaTests/<Name>SketchSnapshotTests.swift`
   where the snapshot check looks (`just ios-snapshots-record <Class>`, about 20 seconds a revision once
   built). The pictures go on a claude.ai Artifact page made with the Artifact
   tool and linked from the issue, every state the screen has (empty,
   error, the longest content). A canvas drawing hides real sizes: in the
   #2794 trial the real number buttons were 32 points tall, not the 20 drawn,
   and broke the layout the canvas promised. A control the catalogue lacks
   is not drawn here: it goes through "A new control" (below) first. Keep one
   design session with the sketch open and redraw with live previews, since a
   fresh agent per small change rereads the docs and rebuilds the app before
   drawing anything (#3004); the design reviewer checks each picture as it lands.
5. **Design review.** A fresh `design-reviewer` agent
   (`.claude/agents/design-reviewer.md`, Opus, medium) that did not draw the
   sketch reviews its screenshots against the brief and the written design
   language. Whatever does not fit goes back to the sketch and is reviewed
   again. Only options that pass reach the owner, two or three per screen, on one
   Artifact page, side by side with the review notes and snapshots of the
   neighbouring existing screens. Checks on tokens and files cannot see
   weight or how a control is drawn: the #3004 sketch passed them all and
   still looked wrong.
6. **Jon's word on the sketch.** The page link goes on the issue with the line
   "Waiting for Jon's word on the sketch". A yes becomes "Jon approved the
   sketch on YYYY-MM-DD"; a no is a revision of the same sketch. The build
   PR is the sketch branch itself, rebased onto fresh `origin/main`; the
   sketch is never merged as it stands, and the build PR deletes the sketch file, its snapshot test
   and its recorded images under `ios/IntradaTests/__Snapshots__/` together
   once the real screen and its snapshots exist, or
   `scripts/check-snapshots.sh` fails on an orphan image.
7. **Build.** From the catalogue components and theme values only;
   `scripts/check-theme-values.sh` fails a typed colour or spacing number. A
   changed component starts with its page (below), then SwiftUI, with its
   snapshots. No new component's look arrives during a build: a control
   with no chosen look stops the build and goes through "A new control".
8. **Jon's word on the screenshot.** "Eyes before ready" in
   `.claude/rules/ios-ui.md`.
9. **After merge.** Refresh the design system (below).

### A new control

A control the catalogue lacks is designed as a catalogue component before
any screen uses it. Its page starts from `design/components/_template.md`
with `status: proposed`; two or three looks are drawn in one sketch file,
`ios/Intrada/Sketches/<Name>Sketch.swift`, with its snapshot test as in step
4. The page's front matter names the sketch file and its snapshot until the
build replaces them. The design reviewer reviews the looks as in step 5; the
owner picks one, recorded on the issue as "Look B picked on YYYY-MM-DD".
Only then does a screen
sketch draw it. A control that ships once is copied as precedent, so its
look is chosen before it ships, not during a build (#3018).

## Refreshing the design system

Run this after any merge that changes a component page, `Theme.swift`, the
kit's icon drawables or the sketch kit. The design system is published from `main`
and never edited by hand in Claude Design outside a refresh. A rehearsal
before merge reads the branch in place of `main`: use `HEAD` where step 3 says
`origin/main`, and stop before step 5. Before planning design system work,
list the live artifact's files, so it is never taken for unpublished.

1. **Use the committed kit.** The refresh publishes the `design/sketch-kit/`
   files as committed on `main`; nothing is regenerated. `just check` already
   fails a branch whose kit is stale, so those files match the code.
2. **Read the live copy.** Use the Artifact tool's `read` on the
   [design system](https://claude.ai/artifact/BRrktWYfTdnpj8sREEBsSs), list
   its files (with `scope` set to `files`), then `read`
   `project/design-system.json`. A publish to an artifact this session has
   not read is refused.
3. **Find what changed.** Run `git fetch origin`, then `git diff --stat
   <commit>..origin/main -- design/ ios/Intrada/Resources/Fonts/
   ios/IntradaTests/__Snapshots__/`, where
   `<commit>` is the hash written after `main@` in `lastChange.via` in
   `project/design-system.json`. With no hash there, refresh every file
   below. `design/explorations/` and every other file under `design/` that
   step 4 does not name, such as `design/sketch-kit/README.md`, `icons/` and
   `token-notes.json`, are never published. Step 4 copies only the changed
   files; `read` the live copy of each one you will replace.
4. **Copy into one folder** in your scratchpad, at the design system's paths:
   - `design/components/<Name>/README.md` to `project/components/<Name>/README.md`,
     reshaped. Drop the front matter. Turn each relative link into a GitHub
     link, `https://github.com/jonyardley/intrada/blob/main/<path>`: the
     published page has no repo around it, so relative links break. End the
     page with an `## In the repo` heading, then one line in this form, with
     the page's own path and commit, then each front matter field as a
     bullet, such as `- **status:** stable`:

     ```markdown
     Published from [`design/components/Card/README.md`](https://github.com/jonyardley/intrada/blob/main/design/components/Card/README.md) at main@35370b0b; edit it there, never here.
     ```

   - `design/sketch-kit/design-system.md` to `project/README.md`, the design
     system's own readme, with the same link change and an "In the repo"
     section holding only the "Published from" line;
   - `design/sketch-kit/tokens.json` to `project/tokens.json`;
   - `design/sketch-kit/components/` (`bundle.js`, `bundle.css`, `tokens.css`,
     `index.d.ts`, and each `<Name>/preview.html`) to `project/components/`.
     Without `tokens.css` a canvas gets unstyled components. This
     `project/components/tokens.css` is the kit's; it is not the
     `project/tokens.css` the Design System type makes from `tokens.json`,
     which the type says never to write;
   - never the kit's `icons/` folder: `bundle.js` carries the icons;
   - the fonts live in the design system at `project/fonts/`, copied from
     `ios/Intrada/Resources/Fonts/` and republished only when the font files
     change. `tokens.css` points its font faces there;
   - each new or changed snapshot, uploaded as an asset first, with its
     record added under `assetGroups.Snapshots` in the index.

   A new component page needs a `preview.html` with a `@dsCard` group, or the
   gallery shows no card for it, and a row in the design system's
   `project/assets/Snapshots/README.md`; its row in the readme's component
   table comes from the repo. The Snapshots readme and every
   preview but the kit's five have no source in the repo, so write them in
   the copy you read in step 2. The kit's five previews live in
   `design/sketch-kit/components/<Name>/`.
5. **Publish once** with `url` set to the design system, `file_path` set to
   one of the changed files under that folder, `root` set to that folder, and `files` naming
   only the paths you changed; the tool needs `file_path` as well as `files`.
   Send `index.d.ts` with `contentType` `text/plain`: a `.ts` file is
   refused. Read `project/design-system.json` again just before, then put it
   last in `files`, every time, with
   `lastChange` set to who, when, a one line note and a `via` naming
   `main@<short commit>`, which the next refresh diffs from; add
   an asset record or change `libraries` (React 18 for the kit) only when
   those change.
6. **Check it.** Open the design system and look at each changed page and
   preview. After a token change, look at the token's swatch and at the
   previews that use it, such as the banners for `danger`.

## Changing a component

The component catalogue in `design/components/` is the source for how each
component looks and behaves (#2620, T37). A change runs in this
order, in one PR:

1. **The page.** Change `design/components/<Name>/README.md`: an agent drafts,
   Jon approves the words. A new component starts from `_template.md`, a
   row in the index and a row in the component table in
   `design/sketch-kit/design-system.md`, the design system's readme, and
   reaches step 2 only with its look picked through "A new control" above;
   approved words alone are not enough.
2. **SwiftUI.** Change the iPhone component.
3. **Snapshots.** Record the pictures the page names, and look at them.
4. **`just check`.** `scripts/check-components.sh` fails when a page and the
   code disagree on a file, a symbol or a snapshot.

A new kit icon needs an `ic_*.xml` vector drawable in
`design/sketch-kit/drawables/`, since the kit reads its icons from there.

A new token goes into `Theme.swift` in the same PR, before any
screen uses it, with its usage note in `design/sketch-kit/token-notes.json`
and a run of `just sketch-kit`. A changed token runs in this order, in one PR:

1. **The theme.** Change the value in `Theme.swift`, and its
   note in `token-notes.json` if the note is no longer true.
2. **The kit.** Run `just sketch-kit`, which rewrites `tokens.json` and
   `tokens.css`.
3. **Snapshots.** Even a small shift fails every snapshot that shows the
   colour: a slightly deeper `danger` failed 6 on iPhone
   (#2772).
   Run `just ios-test`, re-record each failing
   class with `just ios-snapshots-record <Class>`,
   then look at each new picture.
4. **`just check`**, then the refresh after merge.

A change to one of the kit's five components
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
