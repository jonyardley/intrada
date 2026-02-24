# Contracts: Weekly Practice Summary

No new API endpoints or contracts are needed for this feature.

All computation is performed client-side in the pure Crux core from existing
`PracticeSession` and `Item` data already fetched by the application. The results
flow through the existing `ViewModel.analytics` field to the web shell.

The only "contract" change is the shape of `AnalyticsView` (a Rust struct serialised
to the shell), which gains:
- Extended `WeeklySummary` fields (comparison data)
- New `neglected_items: Vec<NeglectedItem>` field
- New `score_changes: Vec<ScoreChange>` field

These are documented in `data-model.md`.
