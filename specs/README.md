# specs/ what is live and what is not

The 2026-07 practice-coach pivot was reversed on 2026-08-13 (#1344): the coach
was removed and the session builder restored. Most specs here are therefore
records of shipped behaviour rather than current plans, and carry a banner
saying so. **This index is the fast answer to "can I implement from this
document?"** Where a spec's own banner and this table disagree, this table wins,
because a spec's header is written once and this table is reviewed.

Every top-level `.md` in this folder appears in exactly one section below. A
spec that does not appear here has not been classified, which is a gap to fix
rather than a licence to implement. Files inside a spec's own subfolder
(`<spec>/design/`, and `reflection-loop/spec.md`) travel with their parent and
are not listed separately.

Companion documents outside this folder:
[`docs/roadmap.md`](../docs/roadmap.md) (direction and phase order) and
`just status` (what is in flight, read from GitHub).

## Live, the current design

Accurate about how the app works or is being built now.

| Spec | Scope |
|---|---|
| [`session-builder-revert.md`](session-builder-revert.md) | **The 2026-08 revert.** Restores the session builder and removes the coach machinery (#1344). The decision that set the current product shape |
| [`native-ios.md`](native-ios.md) | The SwiftUI shell on the Crux core, the only shell. Its offline-first decisions are now enforced through `.claude/rules/offline-first.md`, which is where the invariants live; the paid-sync framing survives only here |
| [`ios-testflight-cicd.md`](ios-testflight-cicd.md) | Signing, match, the release lane (`.github/workflows/release-testflight.yml`) |
| [`key-modality.md`](key-modality.md) | Tonic plus major/minor instead of a free-text key (`Modality` in `domain/item.rs`) |
| [`library-sort.md`](library-sort.md) | Sorting the library list (`LibrarySort` in `domain/types.rs`) |
| [`exercise-relations.md`](exercise-relations.md) | The single "Used in" list replacing the related-to breadcrumb (`UsedInCard.swift`) |
| [`piece-related-exercises.md`](piece-related-exercises.md) | Writing your own exercise as the first action when adding exercises to a piece (`LibraryDetailScreen.swift`) |
| [`practice-instruments.md`](practice-instruments.md) | The Focus Player's timer, rep counter and honest click (`click_sounding` in `domain/session.rs`) |
| [`up-next-card.md`](up-next-card.md) | The "Up next" suggestion on the Practice tab (`compute_up_next` in `suggestion.rs`) |
| [`getting-cold-signal.md`](getting-cold-signal.md) | Weighting the "not practised in a while" signal by how well learned a piece is (`staleness.rs`) |
| [`api-removal.md`](api-removal.md) | The 2026-09-12 decision to remove the API and the sync, account and MCP-token client code (#1746, #1749); what went, what stays, what Jon tears down by hand |
| [`retire-shell-dead-session-fields.md`](retire-shell-dead-session-fields.md) | Removing the session intention, time target and the three reflection boxes nothing could set any more (#1766, #1374). Shipped in #1770 and #1780; the four columns (the intention and the three reflection boxes) stay, unread, in the on-device `session` table |
| [`profile.md`](profile.md) | The musician's name, instrument, icon and highlighter colour, held on the device (`domain/profile.rs`, `AppEffect::SaveProfile`). Shipped across #1691 and #1692 |
| [`one-pass-create.md`](one-pass-create.md) | Adding a piece with its chord chart and exercises in one save (`ItemEvent::AddPieceInFull`, sent from `LibraryAddScreen.swift`). Core in #1591, screens in #1596 |
| [`picker-core-sort.md`](picker-core-sort.md) | The linked-item picker sheet's sort and search calling into the core (`sort_and_filter_candidates` in `app.rs`, `sort_and_filter_picker_candidates` in `intrada-ffi`). Core in #1662, screens and the Swift copies deleted in #1664 |
| [`exercise-variations.md`](exercise-variations.md) | Variations defined on the exercise, and a session recording what was actually played (#1739). Phases A, B and C shipped; #1501 and #1107 follow outside it. Supersedes the current rung and the one-score session entry of #1083 (`exercise-linking-origins.md`) |
| [`notice-channel.md`](notice-channel.md) | A calm grey banner for an outcome that is true but not a failure (`ViewModel.notice`, #1325). Core in #2034, screens in #2038 |
| [`save-acknowledged.md`](save-acknowledged.md) | Save session waits for the store before clearing the summary and the crash-recovery copy (#974, shipped in #2018) |
| [`practice-weeks-in-core.md`](practice-weeks-in-core.md) | The Practice tab's week strip and the Progress headline numbers worked out in the core (#2046). Core in #2054, screens in #2056 |
| [`view-cache.md`](view-cache.md) | Building the library and Progress once per change rather than once per render, and sending the library across once (#1998, #1956). Shipped in #2071 and #2073 |
| [`list-reload-race.md`](list-reload-race.md) | A library or history reload never overwrites an edit made while it was in flight (#2067, shipped in #2072) |

## Planned, designed and not finished

| Spec | Scope |
|---|---|
| [`android-shell.md`](android-shell.md) | An Android app on the same core, a second native shell with no core changes. Specced under #1774; nothing is built |

## Shipped record, verify against the code before extending

Builder-era specs whose surfaces returned with the restored session builder, and specs shipped with their later phases parked.
Treat them as history of how the behaviour came to be, not as an implementation
contract.

| Spec | Scope |
|---|---|
| [`chart-to-scaffold.md`](chart-to-scaffold.md) | Chord-chart parsing and scaffold derivation (`domain/chart.rs`). Phase C shipped in PR #1111. The twelve-key ladder originally scoped inside it is still open as #1107, and that issue records that the steps mechanism it needs has since shipped, so the old #1083 blocker is gone |
| [`exercise-linking-origins.md`](exercise-linking-origins.md) | How exercises came to be linked to pieces and to have variations: the decisions that still hold from #1015, the #1026 to #1036 screen work and #1083, folded from four earlier specs (#1959). Read `exercise-variations.md`, `exercise-relations.md` and `piece-related-exercises.md` for current behaviour |
| [`session-block-grouping.md`](session-block-grouping.md) | Grouping and reordering blocks in the builder (`group_id`, `UngroupBlock`). Shipped via #1022, which the spec itself does not cite |
| [`track-exercises-per-piece/`](track-exercises-per-piece/) | Per-piece exercise tracking |
| [`native-player.md`](native-player.md) · [`native-ios-player.md`](native-ios-player.md) | The Focus Player, as two sequential phases (#932 spine, #948 persistence) |
| [`priority-items.md`](priority-items.md) | Priority items replacing Goals in the Plan layer. Landed across #739 and #769, with #981 as its slice 2 |
| [`piece-from-photo.md`](piece-from-photo.md) | Adding a piece from a photograph of the page. Phases A, B and C shipped (#1443, #1449, #1455, #1457, #1469, #1476). Phase D, a chord chart from a photo (#1387), is parked with every other chord chart issue in epic #2025 |

## Historical, do not implement from these

Kept as a design record. Every one of these targets a shell or a crate that no
longer exists, or a direction that was reversed, so the code they describe
cannot be found and should not be recreated from them.

| Spec | Why |
|---|---|
| [`intrada-practice-coach-design.md`](intrada-practice-coach-design.md) | The retired coach design, built through Phase 2b and removed 2026-08-13 (#1344) |
| [`intrada-coach-engine.md`](intrada-coach-engine.md) | The retired coach engine, same removal. Recover code from commit 071b85b. None of `CoachState`, `MasteryStore` or `JudgementStore` exist |
| [`design-system.md`](design-system.md) | **The Leptos web shell's dark glassmorphism system**, per its own first line. `crates/intrada-web` was deleted in #1133. The live system is `design/intrada-design-system.dc.html` with `Theme.swift` canonical, both outside `specs/` |
| [`design-refresh-2026.md`](design-refresh-2026.md) | A refresh of that same web theme, written before native iOS existed. None of `AccentRow`, `StatCard` or `DifficultyDots` exist anywhere in the tree |
| [`onboarding-welcome.md`](onboarding-welcome.md) | A first-run carousel for the deleted web shell; the surface did not return |
| [`background-audio-plugin.md`](background-audio-plugin.md) | Written as a Tauri plugin under `crates/intrada-mobile`, which no longer exists. Superseded by #1399: the native app keeps the metronome going in the background (`UIBackgroundModes: audio` in `Info.plist`) and the timer reads the wall clock, so there is nothing to port. The reference `BackgroundAudioPlugin.swift` was removed in #1745; recover it from `main`'s history (`git log --diff-filter=D -- ios/Reference/`) |
| [`live-activity-plugin.md`](live-activity-plugin.md) | Same: an ActivityKit design as a Tauri plugin in a deleted crate. Its reference Swift was removed in #1745 too; same recovery route. A native lock-screen spec does not exist yet |
| [`reflection-loop/`](reflection-loop/) | The three session-level reflection boxes and `UpdateSessionReflection`. Shipped, then retired by #1766 (see `retire-shell-dead-session-fields.md`); they survive only as unread columns in the on-device `session` table |

[`_archive/`](_archive/) holds the numbered SpecKit-era folders and retired
single-file specs (`seo-prerender.md`, `mcp-server.md`,
`account-settings-and-deletion.md`: the last two targeted the removed API,
#1746), and is excluded from the knowledge graph. Do not run `/speckit-*`
commands.
