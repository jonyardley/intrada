# specs/ — what's live and what isn't

The 2026-07 practice-coach pivot was reversed on 2026-08-13 (#1344): the coach
was removed and the session builder restored. Most specs here are therefore
records of shipped behaviour rather than current plans, and carry a banner
saying so. **This index is the fast answer to "can I implement from this
document?"**

## Live — the current design

| Spec | Scope |
|---|---|
| [`session-builder-revert.md`](session-builder-revert.md) | **The 2026-08 revert.** Restores the session builder and removes the coach machinery (#1344) |
| [`native-ios.md`](native-ios.md) | The SwiftUI shell on the Crux core — the only shell |
| [`design-system.md`](design-system.md) · [`design-refresh-2026.md`](design-refresh-2026.md) | The Paper & Score system; `Theme.swift` is canonical |
| [`ios-testflight-cicd.md`](ios-testflight-cicd.md) | Signing, match, the release lane |
| [`background-audio-plugin.md`](background-audio-plugin.md) | Background audio-session handling, preserved from the removed Tauri host |
| [`live-activity-plugin.md`](live-activity-plugin.md) | ActivityKit reference, preserved from the removed Tauri host |
| [`mcp-server.md`](mcp-server.md) | The API's MCP surface (still shipped) |
| [`account-settings-and-deletion.md`](account-settings-and-deletion.md) | Auth and account handling |
| [`key-modality.md`](key-modality.md) | Key/modality handling |
| [`library-sort.md`](library-sort.md) | Library sorting |

Companion documents outside this folder:
[`docs/roadmap.md`](../docs/roadmap.md) (direction and phase order) and
`just status` (what's in flight, read from GitHub).

## Planned — designed, not built

| Spec | Scope |
|---|---|
| [`piece-from-photo.md`](piece-from-photo.md) | Adding a piece from a photograph of the page: storage, then OCR, then on-device extraction (#1355, #1387) |

## Shipped record — verify against the code before extending

Builder-era specs whose surfaces returned with the restored session builder.
Each carries a banner saying so; treat them as history of how the behaviour
came to be, not as an implementation contract.

| Spec | Scope |
|---|---|
| [`chart-to-scaffold.md`](chart-to-scaffold.md) | Chord-chart parsing and scaffold derivation (`chart.rs`) |
| [`exercise-variants.md`](exercise-variants.md) | Exercise steps (variants) and the per-step ladder |
| [`session-block-grouping.md`](session-block-grouping.md) | Grouping and reordering blocks in the session builder |
| [`piece-linked-exercises.md`](piece-linked-exercises.md) · [`piece-linked-exercises-design-brief.md`](piece-linked-exercises-design-brief.md) | Piece-linked exercises |
| [`related-exercises-redesign.md`](related-exercises-redesign.md) | Related-exercises design reconciliation |
| [`reflection-loop/`](reflection-loop/) | The reflection loop's core model |
| [`track-exercises-per-piece/`](track-exercises-per-piece/) | Per-piece exercise tracking |
| [`native-player.md`](native-player.md) · [`native-ios-player.md`](native-ios-player.md) | The Focus Player |
| [`priority-items.md`](priority-items.md) | Priority items replacing Goals in the Plan layer |

## Historical — do not implement from these

| Spec | Why |
|---|---|
| [`intrada-practice-coach-design.md`](intrada-practice-coach-design.md) | The retired coach design (built through Phase 2b, removed 2026-08-13, #1344). Kept as the design record |
| [`intrada-coach-engine.md`](intrada-coach-engine.md) | The retired coach engine spec — same removal; recover code from commit 071b85b |
| [`onboarding-welcome.md`](onboarding-welcome.md) | Targets the deleted web shell; the surface did not return |
| [`seo-prerender.md`](seo-prerender.md) | **Obsolete:** the web shell it targets was deleted (#1133) |

[`_archive/`](_archive/) holds the numbered SpecKit-era folders and is excluded
from the knowledge graph. Do not run `/speckit-*` commands.
