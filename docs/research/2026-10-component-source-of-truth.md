# One place of truth for intrada's components

Research for #2603 and #2599, 13 October 2026, trimmed to the answer and the
workflow. Delete when #2620 lands.

## The answer

Keep the component truth in the repo, one plain Markdown spec per component in a folder that mirrors the Claude Design "Design System" artifact's own layout (`design/components/<Name>/README.md`), with pictures produced by the snapshot tests both apps already run, and a check in `just check` that fails when a listed primitive lacks a spec, a SwiftUI symbol, a Compose symbol or a snapshot on either platform (or a recorded reason it does not apply). The Design System artifact then becomes a published view of those files, refreshed by an agent after a merge, which Claude Design and people browse; it is never edited as the source. This is how Skyscanner's Backpack runs two native libraries, and how GOV.UK and Polaris keep their docs (Markdown beside the code). It is the only option where what agents read, what checks read and what people browse cannot silently disagree. It needs no new dependency to start.

## Who edits what

1. A change to a primitive starts with its spec (an agent drafts, Jon approves the words).
2. The agent changes the SwiftUI component, then the Compose one, then records the snapshots on both.
3. `just check` runs the component check; the pull request shows both pictures under "What it looks like" as today.
4. After merge, an agent republishes the changed component folders and screenshots to the Design System artifact.
5. In Claude Design, Jon mocks screens against the artifact; a new component found there comes back as a spec change in step 1, never as a direct edit.

## Workflow

1. **Agree the template.** Header (purpose sentence, SwiftUI symbol and file, Compose symbol and file, snapshots per platform, status, related T decisions), then When to use, When not to use, Anatomy, Variants, States, Behaviour (motion, haptics), Content, Accessibility, Do and don't, Usage (Swift and Kotlin), Related, Changes. The folder layout mirrors the artifact's `components/<Comp>/README.md`, so publishing is a copy plus a generated preview card per component that shows the uploaded screenshots.
2. **Write the check first,** a sibling of `scripts/check-snapshots.sh`, failing on today's gaps, with an allow list that shrinks as specs land. It checks that every name in the index has a spec, each spec's symbols exist, and each named snapshot exists on both platforms or a reason is written.
3. **Pilot three primitives** from #2603's trial: card (`cardSurface` with `cardShadow`), segmented pills, and the primary button (`MarkerButton` on iOS). Publish them to a Design System artifact, then a cold agent builds a screen from the artifact alone.
4. **Fill the rest of the 17,** folding in the October design audit: where it found several looks for one job, the spec picks one and the fix issues reference it.
5. **Point everything at the index:** `.claude/rules/ios-ui.md` and `android-quality.md` link to it instead of listing names; `docs/design-workflow.md` gains the order above; a T decision in `docs/design-principles.md` records it.
6. **Move the design system page into the artifact** before 14 December 2026, when standalone claude.ai/design closes ([support article](https://support.claude.com/en/articles/14604416-get-started-with-claude-design), [Anthropic article](https://claude.com/resources/articles/claude-design-stays-on-brand-for-daily-work)): tokens from `Theme.swift`, components from `design/components/`, and retire the names-only table.
7. **Later, only with Jon's word on a dependency:** Prefire to turn the iOS previews into tests, and `@Preview` plus ComposablePreviewScanner on Android once Roborazzi's preview support leaves experimental.
