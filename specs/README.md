# specs/ — what's live and what isn't

The 2026-07 practice-coach pivot superseded most of the feature specs here. They
are kept for archaeology and carry a banner saying so. **This index is the fast
answer to "can I implement from this document?"**

## Live — the current design

| Spec | Scope |
|---|---|
| [`intrada-practice-coach-design.md`](intrada-practice-coach-design.md) | **The governing design.** Vision, decisions 1–17, pedagogy model, intent, session design, architecture, build plan. v5. |
| [`intrada-coach-engine.md`](intrada-coach-engine.md) | **The engine's technical design.** Mastery update, judgement track, session state machine, planner order, FFI contract, interruption arbitration, gate schema. Rides with Phase 1's first PR |
| [`native-ios.md`](native-ios.md) | The SwiftUI shell on the Crux core — the only shell |
| [`design-system.md`](design-system.md) · [`design-refresh-2026.md`](design-refresh-2026.md) | The Paper & Score system; `Theme.swift` is canonical |
| [`ios-testflight-cicd.md`](ios-testflight-cicd.md) | Signing, match, the release lane |
| [`background-audio-plugin.md`](background-audio-plugin.md) | **Phase 1 input** — the audio-session groundwork the click track needs |
| [`live-activity-plugin.md`](live-activity-plugin.md) | ActivityKit reference, preserved from the removed Tauri host |
| [`mcp-server.md`](mcp-server.md) | The API's MCP surface (still shipped) |
| [`account-settings-and-deletion.md`](account-settings-and-deletion.md) | Auth and account handling |
| [`key-modality.md`](key-modality.md) | Key/modality handling — a keep-column asset the coach relies on |
| [`library-sort.md`](library-sort.md) | Library sorting; the library survives the pivot |

Companion documents outside this folder:
[`docs/rebuild-review.md`](../docs/rebuild-review.md) (pivot strategy, keep/delete
columns, the FFI batching contract), [`docs/coach-user-journeys.md`](../docs/coach-user-journeys.md)
(the ten live scenarios), [`docs/roadmap.md`](../docs/roadmap.md) (phase order),
and [`content/`](../content/) (the Phase 0 authored content).

## Superseded — do not implement from these

Each carries a banner explaining what replaced it.

| Spec | Why |
|---|---|
| [`chart-to-scaffold.md`](chart-to-scaffold.md) | **Partial:** `chart.rs` theory is needed on day one of scoring; the commit-scaffold-to-library workflow is notebook-era |
| [`exercise-variants.md`](exercise-variants.md) | **Partial:** the per-step ladder *shape* informs per-parameter-level mastery; its self-rated data source does not |
| [`session-block-grouping.md`](session-block-grouping.md) | Extends the session builder, which is deleted at Phase 2 |
| [`piece-linked-exercises.md`](piece-linked-exercises.md) · [`piece-linked-exercises-design-brief.md`](piece-linked-exercises-design-brief.md) | Replaced by skill nodes × tune parameterisation |
| [`related-exercises-redesign.md`](related-exercises-redesign.md) | A notebook surface; the coach parameterises drills by tune |
| [`reflection-loop/`](reflection-loop/) | Self-report is no longer the progress signal |
| [`track-exercises-per-piece/`](track-exercises-per-piece/) | Context now comes from measured attempts against nodes |
| [`native-player.md`](native-player.md) · [`native-ios-player.md`](native-ios-player.md) | The focus player is replaced by the drill screen |
| [`onboarding-welcome.md`](onboarding-welcome.md) | Onboarding and placement deferred to Phase 4 |
| [`priority-items.md`](priority-items.md) | Prioritisation is the planner's job now |
| [`seo-prerender.md`](seo-prerender.md) | **Obsolete:** the web shell it targets was deleted (#1133) |

[`_archive/`](_archive/) holds the numbered SpecKit-era folders and is excluded
from the knowledge graph. Do not run `/speckit-*` commands.
