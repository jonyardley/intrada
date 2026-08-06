# Status — what's in flight now

*One screen: current phase, what's in flight, what just landed, what's next.
Updated in every PR that changes scope or state (see CLAUDE.md → After
completing work). Direction and phases live in [`roadmap.md`](roadmap.md);
scope and timing detail on the
[project board](https://github.com/users/jonyardley/projects/2). When this
doc and the issues disagree, the issues are right.*

> Last updated: 2026-08-06 (#1231, #1236, #1175, #1239 follow-ups)

## Where we are

The practice-coach pivot, **Phase 2a — prescribe and run** (in flight), with
**Phase 0 — the paper-teacher fortnight** running alongside (#1143). Machine
listening is deferred (decision 18); the loop runs on tap-verdicts against
countable criteria.

## In flight

- #1143 — Phase 0 fortnight practising from `content/`

## Recently landed

- #1223, #1224, #1225 — the loop's choreography, corrected from Phase 0 play:
  the click runs unbroken for a whole block (one count-in at block entry, and
  a restart only where a parameter actually changed), every block opens on an
  entry card with Start and Skip, a false start can be discarded without
  counting, the authored sparse click levels are honoured by the audio rather
  than only by the pill, and press-start shows the whole session rather than
  block one. Rationale: `docs/design-principles.md` T11, which supersedes half
  of T10. Design folded into `design/intrada-design-system.dc.html` first
  (#1232): Coach primitives `TapVerdict`, `BlockEntryCard`, `PlanBlockRow` and
  `CoachAction`.
- #1239 — agent teams retired: one agent per slice, worktree fan-out only for
  genuinely independent work. Evidence from building #1223 as a team. Its
  follow-up #1199 (speed edits to the team briefings) closed with it: the
  briefings are gone, and the one item that outlived them shipped as #1204.
  The `plan-slice` skill went too: its tier and fan-out rules are already in
  CLAUDE.md, so it was restating them for one reader.
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
- #1200, #1202–#1204 — local gate speed-ups: iOS suite tiering and the
  concurrency guard, `xcodegen --use-cache`, an XCUITest flake retry, and a
  green-stamp so an unchanged HEAD skips gates it has already passed
- #1231, #1175 — two conventions questions settled rather than re-opened per
  PR: ` — ` stays as a list-item label separator (the exception is written into
  CLAUDE.md § Conventions), and the BeatPosition current pip is `accent`, with
  the design system following the build. The block ceiling separates from the
  elapsed clock on weight, so `inkFaint` really is absent from the loop now.

## Next (once 2a closes)

- #1233 — the Claude Design project is behind the repo design system: sync the
  new `CoachAction` primitive up, then re-export through Share → Export. The
  4.9MB `design/intrada-design-system.html` is one BeatPosition paragraph behind
  the `.dc.html` as of #1175, deliberately: #1232 patched that bundle in place
  and the re-export should be a real export, done once, on this visit
- #1214 — the mastery store survives a restart, which is what makes the
  planner's overdue pull and the cold test real
- #1190 — delete the session-builder machinery (2a close-out)
- Phase 2b — steer and guard: declaration surfaces, back-chaining, gap read,
  circling check, grind trade (see the phase plan in `roadmap.md`)
