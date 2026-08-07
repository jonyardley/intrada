# Status — what's in flight now

*One screen: current phase, what's in flight, what just landed, what's next.
Updated in every PR that changes scope or state (see CLAUDE.md → After
completing work). Direction and phases live in [`roadmap.md`](roadmap.md);
scope and timing detail on the
[project board](https://github.com/users/jonyardley/projects/2). When this
doc and the issues disagree, the issues are right. For the last-updated date,
ask git: `git log -1 --format=%cs docs/status.md`.*

## Where we are

The practice-coach pivot, **Phase 2a — prescribe and run** (in flight), with
**Phase 0 — the paper-teacher fortnight** running alongside (#1143). Machine
listening is deferred (decision 18); the loop runs on tap-verdicts against
countable criteria.

## In flight

- #1143 — Phase 0 fortnight practising from `content/`

## Recently landed

- #1244 — the coach engine runs a clickless block (decision 20's engine half):
  `ClickLevel` gains l0 at the bottom of the ladder, a block at it starts
  playing rather than counting in, and the tap bounds the attempt (Jon's ruling
  on the issue, 7 Aug) since there is no phrase boundary to open a window on.
  Evidence is tagged `TapVerdictUntimed` and lands on l0's own mastery key, so
  knowing it out of time never vouches for the tempo; a tempo-down rung cannot
  act where there is no tempo. `DrillView::tempo_bpm` is now `Option<u16>`.
  The l0 drill screen is #1260 and the gates that use l0 are #1245, so nothing
  reaches it in the app yet
- #1190 — the session-builder machinery deleted (2a close-out): the Building
  phase and its screens (`SessionBuilderScreen`, `AddToSessionSheet`,
  `EntrySettingsSheet`, `AddRelatedExerciseSheet`) are gone from both
  `intrada-core` and `ios/Intrada`, along with the Building-coupled `Set`
  events (`SaveBuildingAsSet`, `LoadSetIntoSetlist`, `UpdateSetFromBuilding`).
  The session archive, the Active/Summary phases and their screens
  (`FocusPlayerScreen`, `SessionSummaryScreen`) stay — that's the session
  domain, not the builder. Recovery commit noted on the PR.
- #1248 — the Claude Design project caught up with the repo design system
  (`CoachAction` synced up, `design/intrada-design-system.html` re-exported
  through Share → Export), closing #1233
- #1249 — CLAUDE.md split into rules and `docs/reference.md`
- #1214 follow-up — an attempt carries the rung it was played at
  (`AttemptSummary::level`), so an escalated block replays truthfully: the
  ladder's tempo drop puts two rungs in one block, and without this the rebuild
  relocated the pre-drop evidence onto the post-drop rung. Also: a corrupt
  attempts blob fails the read instead of replaying as a block nobody played
  (invariant 5), and the spec gained the read-side contract (§7)
- #1214 — the mastery store survives a restart: a local-first launch reads the
  persisted block records back (`LoadCoachRecords`) and replays them through
  the mastery track in timestamp order, level-ups included. The planner's
  overdue pull and the cold test now fire for a real user, not only in tests
- Decision 20 — acquisition before the clock (design doc v8, T12): new material
  is learnt out of time before it is owned in time. The sparse-click ladder
  reaches l0 (no click, no tempo), material stages split into know-it and
  own-it-in-time gates, chunks follow authored musical structure (enter at the
  section, split on a fumble, merge on clean, joins their own rung), and tempo
  ramps up to a gate's target instead of starting there. Guiding, not
  prescriptive: Skip jumps the ramp, later gates retire earlier ones, pace is
  inferred from verdicts. Engine + planner work tracked in #1244, #1245
  (2b scope); until then the know-it rungs are practised by hand, Phase 0 style
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

- Phase 2b — steer and guard: declaration surfaces, back-chaining, gap read,
  circling check, grind trade, the rest of acquisition before the clock
  (#1245's gates and ramp, #1260's l0 screen; see the phase plan in
  `roadmap.md`)
