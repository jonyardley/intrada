# Status — what's in flight now

*One screen: current phase, what's in flight, what just landed, what's next.
Updated in every PR that changes scope or state (see CLAUDE.md → After
completing work). Direction and phases live in [`roadmap.md`](roadmap.md);
scope and timing detail on the
[project board](https://github.com/users/jonyardley/projects/2). When this
doc and the issues disagree, the issues are right.*

> Last updated: 2026-08-05 (#1219)

## Where we are

The practice-coach pivot, **Phase 2a — prescribe and run** (in flight), with
**Phase 0 — the paper-teacher fortnight** running alongside (#1143). Machine
listening is deferred (decision 18); the loop runs on tap-verdicts against
countable criteria.

## In flight

- #1143 — Phase 0 fortnight practising from `content/`
- Tooling: gate speed-ups (#1202–#1204), team-briefing edits (#1199)

## Recently landed

- #1188 — the mastery track: Beta state per (node, level), fed by tap-verdicts
- #1189 — the five-stage planner as a pure function, retiring the seeded block
- #1182 — the drill loop reached from Practice: press-start, and the ladder's
  last two rungs
- #1122 — re-exported `design/intrada-design-system.html` (the Step ladder
  gallery entry, via Claude Design's Playwright-driven export)
- #1194 — the engine reads its authored content (`gates.toml` parser, planner
  + persistence groundwork); tap-verdict evidence survives a crash
- #1183 — the session state machine and the tap-verdict bridge
- #1178 — the drill screen (A2 during-play + A3 tap-verdict) and the seven
  coach primitives
- #1200 — iOS test gate tiering and the concurrency guard

## Next (once 2a closes)

- #1214 — the mastery store survives a restart, which is what makes the
  planner's overdue pull and the cold test real
- #1190 — delete the session-builder machinery (2a close-out)
- Phase 2b — steer and guard: declaration surfaces, back-chaining, gap read,
  circling check, grind trade (see the phase plan in `roadmap.md`)
