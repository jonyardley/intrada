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
- #1256 — built session, play-through altitudes, qualitative capture (2b).
  Phase A landed the spec (`specs/built-session.md`), the pulled A/B/C mockups
  (`specs/built-session/design/`), the voice spec graduated as T13, and the
  core scaffold: entities (`UserDrill`, `JournalItem`, `BuiltSession`,
  `PlayThroughRecord`, `Reflection`, `FeelEntry`), `BuiltSessionEvent`
  handlers (local-first only), GRDB v11 with an upgrade-path test, and
  real-bridge round-trips.
  **Phase B landed Journey A end-to-end**: the steer line under the intact
  hero, the compose sheet, decision 19's three-way resolution (proposed node
  match / user drill from one dictated sentence / journal), the composed
  session with declinable shape advice, and the canonical drill loop running a
  user drill with its evidence on its own node. `BlockOrigin` on the spec *and*
  the record enforces decision 17 on the live path and the launch replay
  (GRDB v12); #1269 is closed by the row-quarantine rule.
  **Phase C's core landed Journey B's three altitudes** (screens follow in
  their own PR, per the two-PR rule). Open question 1 is resolved: **v1 has no
  pipeline** — a run-through gates on the piece's own named chart sections,
  and a piece with none is offered the two lower altitudes instead. The
  run-through is a peer state of the drill loop (`SessionState::RunThrough`),
  one "Held / Broke down" tap per section, closing to a `PlayThroughRecord`
  whose verdicts land per section at l0; the launch replay reads them through
  the same function the live close does. Off-piste is now reachable from a
  piece and carries the tag (GRDB v13); unmonitored stays untagged on purpose.
  **#1285 gave unmonitored play somewhere to land**: its minutes were computed
  on close into a field nothing persisted, so decision 16's one output died
  with the process. They now write an `UnmonitoredRecord` (an id and two
  instants, nothing else) to its own `unmonitored_play` table (GRDB v14),
  rather than a discriminated `wander_record`: a table with no column for a
  piece cannot be made to name one, so "minutes only" is enforced by the
  schema rather than by every future writer remembering. `unmonitored_seconds`
  is gone from `EngineSession`, so the crash-recovery key is now v6.
  The superseded `RecordPlayThrough` /
  `SavePlayThrough` door is deleted — run-throughs ride the coach batch, so a
  failed write is retried rather than silently leaving the mastery track ahead
  of the store.
  **Phase C's screens landed Journey B end to end**: "Play it through" on a
  piece opens B0, the sheet asks the core which altitudes that piece can take
  (a piece with no labelled sections keeps the run-through on screen and says
  why), and the run itself is `RunThroughScreen` — one "Held / Broke down" tap
  per named section, the AltitudeChip in the orientation strip for the whole
  run, GateDots carrying per-section verdicts. Off-piste and off-the-record
  ship as `OpenPlayScreen`, drawing their clocks from the core's `started_at`.
  `PlayThroughHost` presents from RootView, because an altitude outlives the
  screen that opened it, and closes with `CloseSession` so the minutes are
  written. Two small read-only view projections were added to the core for it
  (`BuiltView.play_through`, `CoachView.open_play`) — no behaviour, no wire
  change. Deferred with issues: off-piste's mic and its keep-as-drill exit,
  which need an audio effect that does not exist yet (#1304), and resuming a
  crash mid-altitude without pressing start on the hero (#1305).
  **Phase D's core landed Journey C's rules** (screens follow in their own PR).
  The feel question is asked only where a block closed on the judgement track,
  never alongside GateDots, and never after two misses in that block; the
  close reflection is offered once per session and never after unmonitored
  play or off-piste, which have already spent what they may ask. Both prompts
  live on the `Model`, deliberately outside the crash-recovery blob. Open
  question 3 is resolved: **C3 ships rule-based** — the most recent unanswered
  reflection between six and twenty hours old, its first sentence naming
  exactly one thing the library can resolve, quoted back verbatim with one
  eight-minute offer. An ambiguous name proposes nothing, because a wrong
  quote-back is worse than none. Accepting writes `steer`/`steer_at` on the
  reflection (GRDB v15) and the block is re-derived into each plan from there,
  so a relaunch rebuilds it rather than losing it; declining leaves no trace
  beyond never asking again. A transcript that lands late reaches its
  reflection through `AttachTranscript`, so keeping the audio never waits on
  transcription.
  **Phase D's screens landed Journey C end to end**: `FeelScreen` is the feel
  moment (three chips on the `TapVerdict` key shape, only "It sang" tinted,
  Skip a first-class exit), `SessionReflectionScreen` is the question at close,
  and `ProposedSteerCard` is the morning card above the untouched hero. The
  drill loop's cover stays up until both questions have an answer, so the last
  block's feel is never taken down with the session; a gated run-through's
  close reaches the same host through the altitude cover. An accepted steer
  wears "you added this" where the minutes go in today's shape. Deferred with
  issues: C2's mic still waits on the audio effect (#1309), so the reflection
  arrives as text (keyboard dictation included) and keeps no audio; the feel
  screen shows no block position because the prompt does not carry one
  (#1315); the reflection is not linked to its session (#1314); the card
  is not offered on a day the user composed their own session (#1316); and the
  review found that accepting a steer can place nothing and still report
  success (#1317), which is core-side and wants fixing next.
  #1256 closes with it

## Recently landed

- #1205 — `just worktree-new <name>` warm-start bootstrap: creates a worktree
  off fresh `origin/main` and seeds `target/`, `ios/build/{spm,dd}` and
  `ios/generated` from the main checkout via APFS clonefile (`cp -Rc`),
  builds on the sccache work (#1206). Names are restricted to
  `[A-Za-z0-9_-]` — no slashes — so `.claude/worktrees/` stays flat and the
  sim-name collision check can see every worktree. Companion
  `just worktree-rm <name>` cleans up the sim then removes the worktree
- #1272 — crux upgraded to 0.20.0 (`crux_core`, `crux_http`; `crux_macros`
  0.10.1, `facet` pinned `=0.46.5`). The reason to take it: fire-and-forget
  effects no longer leak the bridge registry. Every `Render` and `AppEffect`
  took a slot that was never freed, because only a resolve removed one, so a
  long practice session grew it for the life of the process. It also stops a
  resolved `EffectId` being reissued to a later request, which could deliver
  one request's response to another and succeed silently (our `Store` resolves
  each request once, so we were not hit). No breaking change in 0.20 touches
  us; the generated Swift moves only in doc comments and a redundant explicit
  `Equatable` on 114 types (Swift's `Hashable` already refines it), with no
  type, field, variant or wire change. Riding along: every one of the 19 HTTP
  builders now routes its error arm through one `rejection_detail`, which
  reads the API's `{"error": …}` envelope via `HttpError::body_json`. A
  rejection surfaces the sentence the server wrote rather than
  "HTTP error 409: 409 Conflict", since `Display` carries the status alone.
  Pinned end-to-end through the live bridge, not just in the core.
  `Retry-After` back-off on a 429 is now reachable too, and deferred to #1273
- #1260 — the l0 drill screen: `DrillScreen` branches on `tempoBpm == nil`
  throughout. During `.playing` the tempo/click pill/beat position give way to
  `GateDots` alone, and the tap-verdict footer (`TapVerdict` + escapes) —
  normally only reachable from `.awaitingVerdict` — is reachable from
  `.playing` too, since l0 has no separate verdict phase to arrive at. The
  entry card names "no click" alongside section and minutes. Claude Design was
  skipped for this one (canonical project unreachable from this account, see
  #1244's note below) — built directly against `Theme.swift` and the existing
  coach primitives instead. The gates that use l0 are still #1245, so nothing
  reaches this screen in the seeded app yet
- #1244 — the coach engine runs a clickless block (decision 20's engine half):
  `ClickLevel` gains l0 at the bottom of the ladder, a block at it starts
  playing rather than counting in, and the tap bounds the attempt (Jon's ruling
  on the issue, 7 Aug) since there is no phrase boundary to open a window on.
  Evidence is tagged `TapVerdictUntimed` and lands on l0's own mastery key, so
  knowing it out of time never vouches for the tempo; a tempo-down rung cannot
  act where there is no tempo. `DrillView::tempo_bpm` is now `Option<u16>`.
  The l0 drill screen is #1260 (landed) and the gates that use l0 are #1245,
  so nothing reaches it in the app yet
- #1207 — the native-ios full-tier CI job fanned out: `native-ios-build`
  builds the test products once and uploads them as an artifact,
  `native-ios-test-unit` (IntradaTests) and `native-ios-test-ui`
  (IntradaUITests) consume it in parallel instead of each rebuilding, with the
  #1203 retry flags scoped to the UI job only. A thin `native-ios` fan-in job
  keeps branch protection's required-check name unchanged. `justfile`'s
  `_ios-test-run` was split into `_ios-build-for-testing` /
  `_ios-test-without-building` so CI and local dev share the same xcodebuild
  invocations. Deliberately parked since 2026-08-05 (doubles macOS runner
  minutes to save ~2min); picked up on request. xcodegen version-pin
  follow-up tracked in #1263
- #1206 — sccache adopted for cross-checkout Rust build caching: ~30% faster
  `just check` in a fresh worktree with `target/` cold and the sccache cache
  warm (58s → 40s, measured on the issue). Opt-in per developer via a
  gitignored `mise.local.toml` (`RUSTC_WRAPPER=sccache`), not the committed
  `mise.toml` — nobody's build behaviour changes by surprise. CI stays on
  Swatinem/rust-cache. Follow-up spot-check for `cargo swift package` /
  codegen builds tracked in #1258
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
