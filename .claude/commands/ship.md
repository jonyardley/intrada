---
description: Run the pre-push gates (fmt, clippy, tests, native-iOS) and the self-review checklist before opening/updating a PR. Before starting, read skill://intrada-shipping for the complete gate mechanics, Codecov expectations, and PR/issue body templates.
---

Prepare the current branch to ship. Do these in order and STOP at the first failure, reporting the actual output:

1. **Gates**: `just check` (fmt, then clippy, then tests; the recipes mirror CI's flags and crate exclusions exactly, so local green means CI green). If fmt fails, run `cargo fmt` and re-run. Do not suppress clippy warnings without justification.
2. **Native iOS** (only if files under `ios/` changed): `just ios-fmt-check` FIRST (seconds; CI's `native-ios` job fails on it and `just ios-test-full` won't catch it; fix with `just ios-fmt`), then `just ios-test-full` (builds the app + runs `IntradaTests` + `IntradaUITests` on the pinned iPhone 16 / iOS 26.5 sim; mirrors CI — the full tier, not the fast `just ios-test` inner-loop tier). Skip for Rust-only / non-iOS changes. If a snapshot reference legitimately changed, re-record + `just ios-snapshots-optimize` before committing. Skips outright if HEAD is clean and already stamped green at this tier (#1192) — don't re-run by hand on top of that.
3. **Self-review**: per CLAUDE.md §Workflow/Always(5): for non-trivial work, run the code-reviewer subagent via the `requesting-code-review` skill (include "comment-policy violations are Blockers, not Nits"); for small fixes use `/review`. Apply blockers inline.
4. **Deferred items**: open a GitHub issue for every deferred/out-of-scope item BEFORE posting the review (Always(7)). The self-review PR comment MUST end with `Deferred items tracked: #N, #M` or `none — all flagged items addressed inline`.
5. **Coverage** (Tier 2+): confirm the PR description has a `Coverage:` line; check the Codecov comment after CI.
6. **Status**: nothing to write. What's in flight is read from GitHub by `just status`, so opening the PR and labelling the issue *is* the status update. Update `docs/roadmap.md` only if a phase or direction changed, and `docs/where-we-are.md` only if the release or phase did.

Reminders: never push to `main` — always a feature branch + PR. Run fmt + clippy locally (not just commit-time) to avoid the ~3-min CI roundtrip; for `ios/` changes run `just ios-test-full` (or `just check-all`, which uses the fast `ios-test` tier for the inner loop) to avoid the slower ~5-min macOS CI roundtrip. If UI changed, drive the preview to verify or explicitly hand off verification steps.

$ARGUMENTS
