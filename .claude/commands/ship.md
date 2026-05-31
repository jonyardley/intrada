---
description: Run the pre-push gates (fmt, clippy, tests) and the self-review checklist before opening/updating a PR.
---

Prepare the current branch to ship. Do these in order and STOP at the first failure, reporting the actual output:

1. **Format check** — `cargo fmt --check`. If it fails, run `cargo fmt` and re-check.
2. **Lint** — `cargo clippy -- -D warnings` (and `cargo clippy -p intrada-api -- -D warnings` if API code changed). Fix warnings; do not suppress without justification.
3. **Tests** — `cargo test` (or the targeted crate if the change is scoped). Report pass/fail with the summary line.
4. **Self-review** — per CLAUDE.md §Workflow/Always(4): for non-trivial work, run the `superpowers:code-reviewer` agent (include "comment-policy violations are Blockers, not Nits"); for small fixes use `/review`. Apply blockers inline.
5. **Deferred items** — open a GitHub issue for every deferred/out-of-scope item BEFORE posting the review (Always(6)). The self-review PR comment MUST end with `Deferred items tracked: #N, #M` or `none — all flagged items addressed inline`.
6. **Coverage** (Tier 2+) — confirm the PR description has a `Coverage:` line; check the Codecov comment after CI.

Reminders: never push to `main` — always a feature branch + PR. Run fmt + clippy locally (not just commit-time) to avoid the ~3-min CI roundtrip. If UI changed, drive the preview to verify or explicitly hand off verification steps.

$ARGUMENTS
