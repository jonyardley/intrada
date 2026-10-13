# One place of truth for intrada's components

Research for #2603 and #2599, 13 October 2026. Read only on the repo; every outside claim carries a link, and anything not confirmed says so.

## The answer

Keep the component truth in the repo, one plain Markdown spec per component in a folder that mirrors the Claude Design "Design System" artifact's own layout (`design/components/<Name>/README.md`), with pictures produced by the snapshot tests both apps already run, and a check in `just check` that fails when a listed primitive lacks a spec, a SwiftUI symbol, a Compose symbol or a snapshot on either platform (or a recorded reason it does not apply). The Design System artifact then becomes a published view of those files, refreshed by an agent after a merge, which Claude Design and people browse; it is never edited as the source. This is how Skyscanner's Backpack runs two native libraries (a README per component on each platform, screenshots generated from stories), how GOV.UK and Polaris keep their docs (Markdown beside the code), and it is the only option where the thing agents read, the thing checks read and the thing people browse cannot silently disagree. It costs nothing in cash, needs no new dependency to start, and is roughly five agent sessions of work plus Jon's review of the specs.

## What intrada has today (checked in the worktree at e3cd2990)

- Tokens: `ios/Intrada/DesignSystem/Theme.swift` is the source; `ThemeTokenParityTest.kt` reads the Swift file from a Kotlin test and fails when `Theme.kt` lacks a token without a written reason. `scripts/check-design-tokens.sh` requires every Swift colour on `design/intrada-design-system.dc.html`. Jon's decisions on #2603: stay on Claude Design, and `Theme.swift` stays the token source.
- Component truth: the list of 17 primitives and the rules for using them are prose in `.claude/rules/ios-ui.md`; the design system page has a names-only table ("Everything the app builds its screens from, by its code name").
- Pictures: iOS has 112 `#Preview` blocks and 5 component level snapshots (`ComponentSnapshotTests/`: add row button, score ring three ways, week strip). Android has no `@Preview` at all and 73 Roborazzi references, several already component level (card shadow, card surface, hairline divider, field card, form error banner, banner, instrument glyph).
- A rough grep by name: 12 of the 17 listed primitives have a same named Compose function; TypeBadge, BottomSheet, SegmentedPills, PlaceholderContent and SegmentedProgress have none (SectionTitle does exist, checked by the lead) (they may exist under other names; not checked further).
- The October audit (`docs/audit-2026-10-design.md`) found 40 iPhone and 30 Android findings, led by hand-rolled copies of primitives and one job done several ways (a ticked choice has five looks, I10).
- `scripts/check-snapshots.sh` already walks both snapshot trees for orphans and size; it is the natural neighbour for a component check.

## 1. How mature systems document a component

What the pages share, with the order they use:

| System | Sections on a component page | Source |
|---|---|---|
| GOV.UK | When to use, How it works, one subsection per variant (default, start, secondary, warning, inverse, disabled, grouping, double click), Research on this component, Recent changes, Help improve this component. Accessibility sits inside the variants, not in its own section. Each component is a folder in the repo with `index.md` and one folder per live example. | [button page](https://design-system.service.gov.uk/components/button/), [repo folder](https://github.com/alphagov/govuk-design-system/tree/main/src/components/button) |
| Shopify Polaris | Front matter with a one line description, keywords and one example file per variant, each with a "use for" sentence; then Best practices, Content guidelines, Related components, Accessibility (labelling, keyboard). The React repo is now archived. | [button.mdx](https://github.com/Shopify/polaris/blob/main/polaris.shopify.com/content/components/actions/button.mdx) |
| Atlassian | Usage, Parts (anatomy), paired Do and Don't examples, Accessibility, Best practices, Content guidelines. Also publishes `llms.txt` for agents. | [button usage](https://atlassian.design/components/button/usage), [llms.txt](https://atlassian.design/llms.txt) |
| Apple HIG | Best practices, Style, Content, Role, Platform considerations (per platform), Resources, Change log. | [buttons](https://developer.apple.com/design/human-interface-guidelines/buttons) (headings read from its JSON feed) |
| Material 3 (Android docs in the repo) | Anatomy, Key properties, Variants, Code implementation, Making it accessible, Customising and theming. The m3.material.io pages would not render for me, so this is from the Material Components Android Markdown. | [CommonButton.md](https://github.com/material-components/material-components-android/blob/master/docs/components/CommonButton.md) |
| Microsoft Fluent 2 | A separate usage page per platform for the same component (iOS, Android, web): Types, Layout, behaviour guidance, Accessibility, Content. | [iOS button](https://fluent2.microsoft.design/components/ios/core/button/usage) |
| Nord (Nordhealth; the closest to a small team I found, team size not confirmed) | Usage, Content guidelines, Variants, Additional considerations, Examples (variants, sizes, states). Serves every page as raw Markdown plus a short `llms.txt` index and a large `llms-full.txt`. | [button.md](https://nordhealth.design/raw/components/button.md), [llms.txt guide](https://nordhealth.design/raw/docs/developer/working-with-ai/llms-txt.md) |

The common skeleton, which intrada's spec should copy: a one sentence purpose; when to use and when not; anatomy (named parts); variants, each with a "use for" line; states; behaviour (interaction, motion, haptics); content rules; accessibility; do and don't pairs; usage code; related components; change log. Agent facing systems add a Markdown index at a stable path (Cloudscape, CMS and Nord all publish `llms.txt`: [Cloudscape](https://cloudscape.design/gen-ai/ai-tools/llms-txt-files/), [CMS](https://design.cms.gov/using-ai/llms-txt/)).

## 2. Native catalogue tooling, as of 13 October 2026

Maintenance read from GitHub today.

| Tool | Platform | What it does | Status | Source |
|---|---|---|---|---|
| swift-snapshot-testing | iOS | Already in use. | Release in September 2026 | [repo](https://github.com/pointfreeco/swift-snapshot-testing) |
| Roborazzi | Android | Already in use. Can also generate tests from `@Preview` through ComposablePreviewScanner, marked "Experimental". | Release on 29 September 2026 | [preview support](https://takahirom.github.io/roborazzi/preview-support.html) |
| Prefire | iOS | Turns `#Preview` blocks into swift-snapshot-testing tests and an in app "Playbook" catalogue. | Active, release yesterday | [repo](https://github.com/BarredEwe/Prefire) |
| SnapshotPreviews (now Sentry) | iOS | Renders every Xcode preview to PNG with no test code. | Active, release last week | [repo](https://github.com/getsentry/SnapshotPreviews) |
| ComposablePreviewScanner | Android | Finds `@Preview` functions for screenshot tests. | Active | [repo](https://github.com/sergio-sastre/ComposablePreviewScanner) |
| Compose Preview Screenshot Testing (Google) | Android | `@PreviewTest` on a preview, HTML diff report. Alpha; its standalone plugin is already deprecated in favour of AGP test suites. | Alpha | [Android docs](https://developer.android.com/studio/preview/compose-screenshot-testing) |
| Showkase (Airbnb) | Android | Generated in app component browser from previews. | Last release August 2025, last push March 2026 | [repo](https://github.com/airbnb/Showkase) |
| Playbook-ios | iOS | Storybook style catalogue plus snapshots. | Last push December 2024: treat as stale | [repo](https://github.com/playbook-ui/playbook-ios) |
| Storybook for iOS (aj-bartocci) | iOS | Storybook style SwiftUI viewer; ships a Claude skill. 43 stars. | Last push January 2026 | [repo](https://github.com/aj-bartocci/Storybook-SwiftUI) |
| DocC | iOS | Can hold articles and images, but I could not load Apple's page on images in DocC, and it builds per Swift target, not across two platforms. Not confirmed as a fit. | n/a | [swift.org DocC](https://www.swift.org/documentation/docc/) |

Findings:

- **Nothing renders SwiftUI and Compose side by side.** I found no tool that does; the only cross platform catalogue was Skip's showcase ([Skip](https://skip.dev/docs/modules/skip-fuse-ui)), which translates SwiftUI into Compose and so does not apply to two hand written shells. Pairing is done by file convention, as Backpack does.
- **Preview scanners help iOS only today.** iOS has 112 previews; Android has none, so ComposablePreviewScanner, Showkase and Google's tool would have nothing to read until previews are written.
- **Effort to adopt the two worth considering:** Prefire is a Swift package plus a build plugin and reuses swift-snapshot-testing, which intrada already has; Roborazzi's preview generation is a Gradle flag plus ComposablePreviewScanner, but Android first needs `@Preview` functions, of which it has none.
- **No new dependency is needed to start.** The existing snapshot tests can produce the catalogue pictures if each primitive gets one test per state, named by convention. Adding Prefire later would turn the iOS previews into tests for free, but it is a new dependency and needs Jon's word.

## 3. Docs as code, design tool first, or a docs platform

| Model | Who owns truth | How drift is caught | Cost | Fit for intrada |
|---|---|---|---|---|
| Docs as code (GOV.UK, Polaris, Material Android, Backpack) | The repo; the site is built from it | Review in the same pull request as the code, plus checks and generated screenshots | Nothing beyond time | Matches "the repo stays canonical" in `docs/design-workflow.md` and the `Theme.swift` decision |
| Design tool first (Figma with Code Connect) | The design file; code maps to it | Code Connect links code to components; Swift and Compose mapping needs Figma's $55 plan ([Code Connect](https://developers.figma.com/docs/code-connect/), [pricing](https://www.figma.com/pricing/)) | $55 a seat a month, per #2603 | Rejected by Jon on #2603 for cost |
| Docs platform: zeroheight | The platform, fed from design tools and Storybook | Integrations; Free plan has 1 editor, 1 styleguide, 1 token set and an MCP server at 500 calls a month; Starter $49 an editor a month on the plan card, $59 monthly in the page's own FAQ | Free to $59 | Nothing on native code on the pricing page ([pricing](https://zeroheight.com/pricing/)) |
| Docs platform: Supernova | The platform | Pipelines and exporters; Starter is free with 3 editors, 1 design system and 20 documentation pages; custom code exporters need Pro at $399 a month | Free to $399 | 20 pages would not hold intrada's 17 primitives plus foundations and patterns; native export not confirmed ([pricing](https://www.supernova.io/pricing)) |

## 4. What Claude Design's Design System artifact can hold

From the type's own instructions (read through the Artifact tool today) and Anthropic's articles:

- **It holds files under `project/`:** a README brand book, `tokens.json` (colours with themes, type, spacing, radius, shadow, each token with a usage note), and for each component `components/<Comp>/README.md` (guidelines, first sentence is the summary) and `components/<Comp>/preview.html` (a small web page that renders the component), plus a shared JS bundle, assets and fonts. Limits: 1,008 files, 256 MiB.
- **Previews are web pages.** A preview runs in the browser against a JavaScript bundle, so a SwiftUI or Compose view cannot render there. A preview can show an uploaded image (`/_blob/<id>`), so native components can appear only as screenshots the repo produced. The artifact can therefore show intrada's components but cannot be the source of how they look.
- **How agents read and write it:** another agent reads `project/README.md` first, then the files it points to; Claude Code writes with the Artifact tool, one publish holding only the changed files, the index last. The index records `lastChange` with a `via`; a `via` starting "CI" marks a pipeline that republishes. I found no documented way for CI to publish, so the refresh is agent driven unless confirmed otherwise.
- **Claude Design and Claude Code:** Claude Design is now a template inside Claude, usable in Claude Code and the Artifacts tab; standalone claude.ai/design closes on 14 December 2026; `/design-sync` pulls a design system in and `/design` brings a design into the codebase ([support article](https://support.claude.com/en/articles/14604416-get-started-with-claude-design), [Anthropic article](https://claude.com/resources/articles/claude-design-stays-on-brand-for-daily-work)). A third party post says `/design-sync` handles React component libraries only ([wmedia.es](https://wmedia.es/en/tips/claude-code-design-sync-your-components)); Anthropic's pages name no frameworks, so **not confirmed**.
- **Not read:** the type's `from-code.md` and `format.md` references (both read attempts failed without creating an artifact), so preview card details beyond the instructions are unconfirmed.

## 5. Keeping two native platforms in step from one spec

- **Backpack (Skyscanner)** keeps one README per component on each platform with the same skeleton (Default, a Day and Night screenshot table, Usage with code), and screenshots generated by script from annotated stories: `@ComposeStory` on Android, a screenshot target on iOS. Contributors must regenerate screenshots when a component's look changes. ([Android Badge](https://github.com/Skyscanner/backpack-android/blob/main/docs/compose/Badge/README.md), [SwiftUI Badge](https://github.com/Skyscanner/backpack-ios/blob/main/Backpack-SwiftUI/Badge/README.md), [Android contributing](https://github.com/Skyscanner/backpack-android/blob/main/CONTRIBUTING.md), [iOS contributing](https://github.com/Skyscanner/backpack-ios/blob/main/CONTRIBUTING.md)). Its SwiftUI library covers only a subset of components, which shows parity slips without a check.
- **Fluent 2** writes a separate usage page per platform for the same component ([iOS](https://fluent2.microsoft.design/components/ios/core/button/usage)).
- **intrada already has the pattern in miniature:** `ThemeTokenParityTest.kt` reads Swift source and fails on a missing counterpart unless a reason is written down. A component registry checked the same way is the "checks, not memory" Jon asked for.

## Three options for intrada

### Option A: spec in the repo, pictures from snapshots, artifact derived (recommended)

What it is: `design/components/<Name>/README.md` for every primitive, using the shared skeleton, with a short header naming the SwiftUI symbol and file, the Compose symbol and file, and the snapshot names on each platform (or "does not apply on Android: reason"). `design/components/README.md` is the index, the agent entry point and the source of the primitive list. Snapshot tests on both platforms render each primitive's variants and states with names that match the spec.

Who edits what, in order:
1. A change to a primitive starts with its spec (an agent drafts, Jon approves the words).
2. The agent changes the SwiftUI component, then the Compose one, then records the snapshots on both.
3. `just check` runs the component check; the pull request shows both pictures under "What it looks like" as today.
4. After merge, an agent republishes the changed component folders and screenshots to the Design System artifact.
5. In Claude Design, Jon mocks screens against the artifact; a new component found there comes back as a spec change in step 1, never as a direct edit.

What catches drift:
- A new check (a sibling of `check-snapshots.sh`, say `scripts/check-components.sh`): every name in the index has a spec; each spec's SwiftUI and Compose symbols exist; each named snapshot exists on both platforms or a reason is written; every spec is in the index; the primitive list in `.claude/rules/ios-ui.md` is replaced by a pointer to the index so there is one list.
- The existing snapshot tests catch a component that changes look; the parity test pattern could move into Kotlin later if the shell script grows awkward.
- The audit's hand-rolled copies are not caught by this check; that needs a separate lint per known copy, as `check-faint-ink.sh` and `check-haptics.sh` do today.

What agents read: the index, then one spec, both in the repo, with no login. Path scoped rules can point at them.

Effort: one session for the template and the check with its self test; one to two sessions for the 17 specs (agents draft from code, the audit and `design-principles.md`; Jon reads each); one session per platform for missing snapshots (iOS needs most, roughly 12 primitives; Android needs the six with no counterpart decided as build or "does not apply"); half a session to seed the artifact. Running cost: no cash; each primitive change adds a spec edit and two snapshot records to work that already happens; one republish per merge that touches `design/components/`.

### Option B: the Design System artifact as the catalogue, repo checks read a copy

What it is: component guidelines are written in the artifact (in Claude Design or by an agent), and a copy is pulled into the repo for checks and agents.

Workflow: Jon or an agent edits the guidelines in the artifact; an agent pulls the changed files into the repo; code changes follow; checks compare the pulled copy against code and snapshots.

What catches drift: the same check as A, but only against the last pulled copy; an edit made in Claude Design and not pulled is invisible to CI. The artifact cannot render native components, so the pictures still come from the repo and have to be pushed up, making the sync two-way.

What agents read: the artifact through the Artifact tool (needs a claude.ai session), or the pulled copy.

Effort: similar to A, plus a pull step that has to be remembered. Running cost: no cash; design work comes out of Jon's Claude usage. Risk: two places that each look like the truth, the same failure #2603 recorded on 13 October.

### Option C: a hosted documentation platform

What it is: zeroheight or Supernova holds the component pages; the repo links to them.

Workflow: pages are written in the platform; screenshots are uploaded or synced; agents read through the platform's MCP server.

What catches drift: the platforms' integrations target Figma and Storybook, neither of which intrada uses; nothing found checks SwiftUI or Compose code against the pages.

Effort: setup plus a manual screenshot upload per change. Running cost: zeroheight free (one editor, 500 MCP calls a month) or $49 to $59 an editor a month; Supernova free but capped at 20 pages, $399 a month for custom exporters. A third tool beside the repo and Claude Design, against the cost reasoning Jon gave on #2603.

## Recommendation and workflow

Option A. It is the only option where the truth sits where agents and checks can read it without a login, it follows the decisions already made (repo canonical, `Theme.swift` the token source, Claude Design kept), and it reuses machinery intrada already trusts (snapshot tests, a parity check, `just check`).

Step by step:
1. **Agree the template.** One page: header (purpose sentence, SwiftUI symbol and file, Compose symbol and file, snapshots per platform, status, related T decisions), then When to use, When not to use, Anatomy, Variants, States, Behaviour (motion, haptics), Content, Accessibility, Do and don't, Usage (Swift and Kotlin), Related, Changes. Folder layout mirrors the artifact's `components/<Comp>/README.md`, so publishing is a copy plus a generated preview card per component that shows the uploaded screenshots.
2. **Write the check first,** failing on today's gaps, with an allow list that shrinks as specs land (as `check-design-tokens.sh` did with the icon ink).
3. **Pilot three primitives** already in #2603's trial: card (`cardSurface` with `cardShadow`), segmented pills, and the primary button (`MarkerButton` on iOS, which is not in today's primitive list; the audit's I9 and A4 show why it should be). Specs, snapshots on both platforms, published to a Design System artifact with the Library screen on both platforms as a card (as #2603's trial asks), then a cold agent builds a screen from the artifact alone.
4. **Fill the rest of the 17,** folding in the audit: where the audit found several looks for one job (I10), the spec picks one and the fix issues reference it.
5. **Point everything at the index:** `.claude/rules/ios-ui.md` and `android-quality.md` link to it instead of listing names; `docs/design-workflow.md` gains the order above; a T decision in `docs/design-principles.md` records it.
6. **Move the design system page into the artifact** before 14 December: tokens from `Theme.swift`, components from `design/components/`, and retire the names-only table.
7. **Later, only with Jon's word on a dependency:** Prefire to turn the 112 iOS previews into tests, and `@Preview` plus ComposablePreviewScanner on Android once Roborazzi's preview support leaves experimental.

## Risks

- **Specs go stale in prose.** The check proves a spec exists and names real symbols and pictures, not that its words are right. Mitigation: the "Eyes before ready" step already puts pictures in front of Jon; add "spec still true" to the shipping self review.
- **Snapshot count and repo size.** One picture per state per platform adds perhaps 60 to 100 snapshot images; `check-snapshots.sh` already caps size.
- **Artifact republish depends on an agent remembering.** No CI route was found. Mitigation: a line in the `intrada-shipping` skill, and `lastChange` in the artifact index shows when it last moved.
- **Claude Design may draw components that do not exist.** Previews there are screenshots, so a mock can still invent; the spec rule "new component means spec first" is the guard.
- **Android gaps are real work:** six listed primitives have no same named Compose function; each needs a build or a written reason, which the check forces into the open.
- **Unconfirmed items:** `/design-sync` framework support; the type's `format.md` and `from-code.md`; whether CI can publish to an artifact; DocC's image handling; Nord's team size.

## Sources

- GOV.UK button: https://design-system.service.gov.uk/components/button/ and repo https://github.com/alphagov/govuk-design-system/tree/main/src/components/button
- Polaris button source: https://github.com/Shopify/polaris/blob/main/polaris.shopify.com/content/components/actions/button.mdx
- Atlassian button usage: https://atlassian.design/components/button/usage ; llms.txt: https://atlassian.design/llms.txt
- Apple HIG buttons: https://developer.apple.com/design/human-interface-guidelines/buttons
- Material Components Android: https://github.com/material-components/material-components-android/blob/master/docs/components/CommonButton.md
- Fluent 2 iOS button: https://fluent2.microsoft.design/components/ios/core/button/usage
- Nord: https://nordhealth.design/raw/components/button.md ; https://nordhealth.design/raw/docs/developer/working-with-ai/llms-txt.md
- Cloudscape llms.txt: https://cloudscape.design/gen-ai/ai-tools/llms-txt-files/ ; CMS: https://design.cms.gov/using-ai/llms-txt/
- Backpack: https://github.com/Skyscanner/backpack-android/blob/main/docs/compose/Badge/README.md ; https://github.com/Skyscanner/backpack-ios/blob/main/Backpack-SwiftUI/Badge/README.md ; https://github.com/Skyscanner/backpack-android/blob/main/CONTRIBUTING.md ; https://github.com/Skyscanner/backpack-ios/blob/main/CONTRIBUTING.md
- Tools: https://github.com/pointfreeco/swift-snapshot-testing ; https://takahirom.github.io/roborazzi/preview-support.html ; https://github.com/BarredEwe/Prefire ; https://github.com/getsentry/SnapshotPreviews ; https://github.com/sergio-sastre/ComposablePreviewScanner ; https://developer.android.com/studio/preview/compose-screenshot-testing ; https://github.com/airbnb/Showkase ; https://github.com/playbook-ui/playbook-ios ; https://github.com/aj-bartocci/Storybook-SwiftUI ; https://www.swift.org/documentation/docc/
- Platforms: https://zeroheight.com/pricing/ ; https://www.supernova.io/pricing ; https://www.figma.com/pricing/ ; https://developers.figma.com/docs/code-connect/
- Claude Design: https://support.claude.com/en/articles/14604416-get-started-with-claude-design ; https://claude.com/resources/articles/claude-design-stays-on-brand-for-daily-work ; https://wmedia.es/en/tips/claude-code-design-sync-your-components (third party)
- Design System artifact type instructions: read through the Artifact tool from https://claude.ai/artifact/5M7UeXXcx16TP3vzVFNDzd on 13 October 2026
