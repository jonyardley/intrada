# Working with Claude Code on intrada — one-pager

> How to get the best out of the agent on this codebase. Read once; skim later.

## The mental model: match ceremony to scope

| Tier | When | What I do |
|------|------|-----------|
| **1 — just do it** | typos, copy, styles, renames, lint/clippy, dep bumps, single-file refactor | read → change → verify → ship. No plan, no spec. |
| **2 — Plan mode** | new view/endpoint/field following existing patterns | (UI: Claude Design first) → Plan → implement |
| **3 — light spec** | net-new feature, FFI/core change, auth/DB/migrations | `specs/<feature>.md` rides with Phase A → implement |

If unsure, I go one tier lighter and drift up. **Auth / the FFI bridge contract / DB schema / migrations always bump up one tier**, regardless of size.

Tiers set the ceremony; [`model-guide.md`](model-guide.md) sets the resourcing: which model and reasoning effort each kind of work (coding, reviewing, planning, design) should run on, and what a plan must say about model, effort, and parallel streams.

## How to brief me for best results

- **State the goal, not the keystrokes.** "Users should be able to X" beats "edit file Y" — I'll find Y and check it's the right place.
- **Name constraints up front** (must stay offline, no new deps, keep web working). I optimise to what you say matters.
- **Point at a reference** ("like the library list does it") — I reuse existing patterns over inventing new ones.
- **Tell me the tier if you have a strong view** ("just do it" / "plan this first"). Otherwise I'll pick and tell you.
- **For anything creative/architectural**, expect me to ask 1–2 clarifying questions first (the brainstorming flow). That's deliberate — it prevents building the wrong thing.

## Guardrails now in place (so you don't have to police them)

- **`rustfmt` runs automatically** on every `.rs` file I edit (PostToolUse hook) → fmt-related CI failures are now structurally impossible.
- **`/ship`** — one command runs the full pre-push gate: `cargo fmt --check` → `clippy -D warnings` → `cargo test` → self-review + deferred-items checklist. Run it before every PR.
- **Pre-push hook** refuses pushes to a merged-PR branch and flags comment-bloat.

## Skills worth invoking deliberately

| Say this | When |
|----------|------|
| "use TDD" (`test-driven-development`) | any `intrada-core` change, non-UI Tier 2, all Tier 3 — write the failing test first |
| "review this" (`requesting-code-review` / `/review`) | before any Tier 2+ PR; say *"comment-policy violations are Blockers"* |
| `/ship` | the standard funnel: gates, then self-review, then PR |
| `/code-review` | review the current diff for bugs at chosen depth (`ultra` = cloud multi-agent) |
| `/fewer-permission-prompts` | clean up permission prompts (already mostly covered) |

The Superpowers plugin is **disabled** — its "invoke a skill for anything" posture conflicts with the tier system. Four of its skills were kept as standalone globals and are invoked by name: `test-driven-development`, `requesting-code-review`, `receiving-code-review`, `using-git-worktrees`. And **never** `/speckit-*`.

## Verification — the standard I hold

- UI changes: I drive the preview and show proof (screenshot / logs), **or** I explicitly say "I can't reach the preview — here are the steps for you to verify X/Y/Z." I won't claim "all green" when that only means `cargo test` passed.
- I report failures faithfully with the actual output — no hedging, no "should work."

## Memory — how context survives

I keep a file-based memory (`~/.claude/projects/.../memory/`) of who you are, your preferences, project direction, and hard-won gotchas. It loads each session. If I learn something non-obvious worth keeping ("we decided X because Y"), I save it. You can say *"remember that …"* anytime.

## What slows us down (avoid)

- Vague approval of large scope ("do the rest") instead of finishing the current slice.
- Asking me to skip verification to "save time" — the rework costs more.
- Burying deferred work in PR descriptions — I open tracked issues instead.
- Pushing to `main` — never; always a feature branch + PR.

## The fastest loops

1. **Feature**: brief goal → I confirm tier → (Claude Design if UI) → implement → `/ship` → PR.
2. **Bug**: paste symptom + repro → failing test first → fix → verify.
3. **Big/parallel work**: say **"workflow"** to opt into multi-agent fan-out (research, broad review, migrations). Costs more tokens; you control when.
