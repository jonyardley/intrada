# The coach engine

Technical design spec, Tier 3, 3 August 2026. Rides with Phase 1's first
implementation PR, not its own. Resolves [#1148](https://github.com/jonyardley/intrada/issues/1148)
and [#1147](https://github.com/jonyardley/intrada/issues/1147).

**Scope:** the seven mechanisms Phase 1 and 2a cannot be built without — mastery
update, judgement track, session state machine, planner resolution order, FFI
contract, interruption arbitration, gate schema. **Not here:** architecture
([`docs/rebuild-review.md`](../docs/rebuild-review.md)), pedagogy and decisions
1–17 ([the design doc](intrada-practice-coach-design.md), v5), scenarios
([`docs/coach-user-journeys.md`](../docs/coach-user-journeys.md)), UI. §9 lists
what this spec contradicts in those sources — fix them there.

## 1. Boundaries

One quarantined module, `intrada-core/src/engine/`. The gravity risk (review §4)
is held off structurally, not by discipline.

```rust
Model     { …existing…, pub coach: CoachState }
ViewModel { …existing…, pub coach: CoachView }
CoachState { mastery: MasteryStore, judgement: JudgementStore,
             session: EngineSession, ledger: InterruptionLedger, content: ContentIndex }
```

- `engine/` never imports the self-report path (`score: Option<u8>`,
  `ItemPracticeSummary`, `analytics.rs`). One test asserts it; a diff adding one
  is a blocker.
- The single permitted history read is `seed_priors_from_archive()` — one-way,
  once, at migration, never on the live path (review §5's priors note).
- No `local_first` branch in `engine/`: local-first only, dual-mode retired.
- Authored content is `include_str!`-embedded and parsed at startup, so the
  engine does no I/O and the planner stays pure.

## 2. Mastery update (#1148)

Inputs settled by decision 17: **measured attempts only**. Evidence unit is one
**scored attempt**, not one gate — a gate ("3 clean passes at 120") is composed
of attempts, and its pass is a derived event triggering a level-up. State is per
`(node, parameter_level)`; there is no node-level scalar (§9.2).

```rust
struct Mastery { alpha: f32, beta: f32, prior: (f32, f32), last_attempt_at: Timestamp }
```

**#1148.1 — Beta, confirmed, with three amendments.** Per-attempt pass/fail is
Bernoulli and Beta is its conjugate, so estimate and confidence fall out of one
state with no second mechanism to keep in sync. Rejected: Elo/Glicko (needs an
item-difficulty scale we lack), Bayesian Knowledge Tracing (slip/guess
parameters need population data — decision 4 forbids inventing it), EWMA (no
confidence for free). Amendments: discounted counts (#1148.2); an evidence cap
`alpha + beta <= 40` so a mastered node can still change its mind (velocity,
lever 4, needs that); and three readings defined once, so display, planner and
tests agree:

```
estimate   = alpha / (alpha + beta)
evidence   = (alpha + beta) - (alpha_0 + beta_0)   // attempts beyond the prior
confidence = evidence / (evidence + k)             // k = 8
```

Content's `(estimate, band)` seeds become priors: band strength `low 2 /
medium 5 / high 10` pseudo-counts, split by the seeded estimate.

**#1148.2 — Decay and spacing are one mechanism read twice.** Elapsed time pulls
counts toward the prior, never toward zero:

```
alpha ← alpha_0 + (alpha - alpha_0) · λ^Δdays          (same for beta)
λ = 0.5 ^ (1 / retention_half_life_days)               (per node family, authored)
interval_days = base_interval · g(estimate, evidence)  // expanding: both raise it
overdue = days_since_last_attempt / interval_days      // due at ≥ 1.0
```

Decay shrinks evidence and moves the estimate toward "we don't really know",
never toward failure — journey 7's requirement, and honest: absence of practice
is absence of evidence. **Decay never manufactures failure evidence.** Spacing is
derived from that same state rather than a second schedule, so the two cannot
disagree. Half-lives and `base_interval` are `gates.toml` data (lever 2: SM-2 is
a heuristic, not gospel).

**#1148.3 — Level-ups shift, never reset.** A new parameter level starts from a
discounted inheritance of the level below, `transfer ≈ 0.35`:

```
prior' = (1 + transfer · evidence_below · estimate_below,
          1 + transfer · evidence_below · (1 - estimate_below))
```

Reset discards real transfer *and* makes every level-up read as a regression —
"I got better and my score dropped" is a trust-ender (UX principle 8). Full
carry-over is equally wrong: a new tempo genuinely is less certain. The level
below is left untouched, so a level-down is a cursor move, not a rewrite.

**#1148.4 — Attempts-to-pass and time-share feed the circling check only.**
Attempts-to-pass *is* the attempt-level Bernoulli sequence seen from the other
end; feeding it in double-counts one piece of evidence. Time-share measures how
the **planner** allocated minutes, and the planner reads mastery — feeding it
back would make mastery partly a record of the app's own choices. Both stay
first-class recorded data (§4), read by the circling check against the user's own
distribution and by gate calibration (challenge 4) as authoring input.

**#1148's fourth listed question** — one mastery estimate or two — was closed by
decision 17 before the issue was read; close it as answered.

## 3. The judgement track

Deliberately smaller: no level, no distribution, no aggregation.

```rust
struct JudgementTarget {
    target: TargetRef,        // Node | PipelineStage | Opaque(String)
    completion: Completion,   // Open | RetiredByYou { at, return_after_days }
    feel: Vec<FeelRating>,    // { at, value: 1..=5, context: BlockId }
}
```

Asymmetric powers (decision 17). **May retire a target** — `RetiredByYou` stops
prescription and schedules a spaced return, which is a prompt, not a gate. **May
never satisfy a prerequisite** — enforced by signature, not convention:
`fn prerequisites_met(node: NodeId, mastery: &MasteryStore) -> bool` cannot see
`JudgementStore`. One test asserts a fully-retired, top-rated target leaves every
downstream node blocked. Opaque targets (decision 13) live here by construction:
no node, so no mastery; a count per normalised string makes the authoring queue
data rather than memory.

**Predict-then-reveal writes to neither track.** A mature-drill attempt carries
`self_predicted: Option<Verdict>` beside its measured verdict; the pair is the
divergence log automated and the evidence base for how much weight the track
eventually earns. It never updates a distribution.

## 4. The session state machine

New machine in `engine/`, beside the legacy `SessionStatus` that Phase 2a
deletes. Off-piste and unmonitored are **peers** of `Running`, not sub-states,
because `Running` implies a plan and gates: `Idle` → `Planned(Plan)` →
`Running { plan, cursor, block }` → `Closing(Summary)`, plus
`OffPiste { started_at }` (no plan, still capturing) and
`Unmonitored { started_at }` (nothing captured, nothing inferred — decision 16).

| From | Event | To | Note |
|---|---|---|---|
| `CountIn` | grid complete | `Listening` | Layer 0: no verdict during play |
| `Listening` | attempt segmented | `Verdict` | one glance, ~1s |
| `Verdict` | fails < trigger, gate unmet | `CountIn` | hands never leave the keys |
| `Verdict` | consecutive fails = trigger | `Escalating { rung }` | acts, doesn't narrate; **no budget spent** (§9.3) |
| `Verdict` | gate satisfied | `GateOpen` | stamps `gate_opened_at_attempt` |
| `GateOpen` | auto-advance / user continues | `Boundary` / `Listening` | continuing is what `reps_after_gate` counts |
| any | ceiling reached | `Boundary` | ceiling shown throughout (decision 15) |
| `Boundary` | arbitration (§7) | next `CountIn` / `Closing` | the only place an interruption may fire |

**Recorded from the first build.** Cheap now, impossible to reconstruct from an
aggregate row later; the circling check, horizons and gate calibration all
depend on them.

```rust
struct BlockRecord {
    id: Ulid, node: NodeId, drill: DrillId, gate: GateId, level: ParameterLevel,
    started_at: Timestamp, ended_at: Timestamp,
    attempts: Vec<AttemptSummary>,     // ordered: verdict, timing facts, self_predicted
    attempts_to_pass: Option<u16>,     // None if never passed
    gate_opened_at_attempt: Option<u16>, reps_after_gate: u16,
    active_ms: u64,                    // time attributed to this node
    escalation_fired: Vec<Rung>,
    exit: Exit,                        // GatePassed | CeilingHit | Skipped | Escalated | SessionEnded
    circle: Circle, mode: Mode,        // the Phase 2 time-by-circle tally, free at write time
}
```

Persistence per the invariants: client-minted ulid (3), append-only with
`updated_at` + `deleted_at` (2), `BlockRecord`s in GRDB, the in-progress
`EngineSession` in `crux_kv` for crash recovery (8).

## 5. Planner resolution order

```rust
fn plan(state: &CoachState, ctx: PlanContext) -> Plan
// ctx: available_minutes, now, steer: Option<Steer>, transport: TransportTier, rng_seed
```

Pure: the clock is a parameter, and the only randomness is a seeded dealer for
random-key gates whose seed is stored on the `Plan`, so any session replays in a
test. Five ordered stages, and **each may only remove or reorder candidates,
never add** — that monotonicity is what makes the why generable by construction.

1. **Resolve declared intent.** Goal → campaign target set → today's steer, each
   defaulted so a user who declared nothing still gets a session (decision 9).
   Opaque targets pass through carrying `scored: false`.
2. **Back-chain.** Through graph prerequisites and pipeline stage order to a
   frontier of `(node, level)` candidates. Ambition beyond the hands returns the
   prerequisite plus an honest distance from the user's own trend (decision 10)
   — never obedience, never refusal. The **structural gap read is this stage's
   byproduct**; the statistical read waits for Phase 4 (decision 4).
3. **Block or interleave, by node maturity.** Low `evidence` at the frontier =
   acquisition, so one longer uninterrupted block; matured = shorter interleaved
   passes (lever 1). `overdue >= 1` pulls maintenance ahead of new keys, which is
   how journey 7 falls out rather than being special-cased.
4. **Grind cap.** `grind`-tagged nodes capped by `grind_max_minutes_per_session`
   and `grind_max_blocks`, always parameterised by the in-flight tune, never
   practised in the abstract (principle 4). A grind trade is a logged debt the
   *next* plan reads, not a silent skip.
5. **Template constraints — last, and overriding.** Warm-up on owned material;
   at least one new-or-applied music block; close on music; in-flight caps (one
   phrase, one–two tunes) and time ceilings. **Caps and template outrank declared
   intent**: an over-full campaign is accepted and *sequenced*, and the plan must
   report what it queued. Silent dropping is a defect.

Each `PlannedBlock` carries `why { destination: Option<TargetRef>, node_state,
placed_by: Stage }`, written by the stage that placed it. Test: every block has a
non-empty why citing the destination whenever one is declared (principle 7).

## 6. The FFI contract

Note events cross **in batches, never per-note** (review §3.1): every event
rebuilds the whole `ViewModel`, so per-note crossing is architecture abuse at
playing speed. The entire engine bridge surface, and it does not grow:

```rust
struct NoteEvent { pitch: u8, velocity: u8, on: bool, t_us: u64 }  // from the click anchor
struct NoteBatch { session: Ulid, seq: u32, anchor_us: u64,
                   events: Vec<NoteEvent>, transport: TransportTier }
struct ClickGrid { bpm_milli: u32, beats_per_bar: u8, count_in_bars: u8,
                   click_level: ClickLevel, anchor_us: u64 }
enum  CoachOperation { ScheduleClick(ClickGrid), StopClick, StartCapture, StopCapture }
enum  EngineEvent    { NotesCaptured(NoteBatch), … }   // one ingest event
struct CoachView     { … }                             // one ViewModel field
```

Rules, from the #846 bincode hazard and review §3.2:

- **No JSON-only serde attributes** on any engine bridge type: no
  `deserialize_with` / `serialize_with`, no `skip_serializing_if`, no
  `Option<Option<T>>`. bincode has no "absent"; a three-state field is an
  explicit enum.
- **Integers where an integer will do** (`bpm_milli`, `t_us`), so timing is exact
  and reproducible rather than rounding-dependent.
- **Bounded batches.** The shell splits at `events.len() <= 512` and delivers per
  beat, per ~250ms, or per attempt window — whichever comes first. An unbounded
  `Vec` on the render path is a latency and allocation hazard.
- **Render coalescing.** `NotesCaptured` returns `Command::done()` unless the
  batch closed an attempt or moved gate progress, so the view rebuild stays as
  low-frequency as the bridge.
- **Round-trip before wiring.** `assert_round_trips` on every bridge type before
  it reaches a screen, plus one `LiveBridge` test pushing a real `NoteBatch`
  through the actual Swift↔Rust wire. A stub bridge cannot catch a wire break.

## 7. Interruption arbitration (#1147)

One budget, one order, one place: five per-feature "once per session" limits are
five interruptions.

```rust
fn arbitrate(candidates: &[Candidate], at: Boundary,
             budget: &mut InterruptionBudget, ledger: &mut InterruptionLedger)
    -> Option<Interruption>
```

- **Budget** 1 per session, 2 when planned minutes ≥ 30. `gates.toml` data.
- **Priority** `StuckWall` > `CirclingCheck` > `GrindTrade` > `CoachVoice` >
  `GapRead` — safety, cost, retention, value, then what can wait (#1147's order,
  adopted).
- **Called from the `Boundary` transition only** (§4), so "never mid-rep" is
  structural rather than a rule each feature has to remember.
- **Losers degrade to the summary** via `Summary.deferred`; nothing fires late.
- **Suppression is persisted**, because the anti-nag rules span sessions:
  `InterruptionLedger` keeps `(kind, subject, last_fired_at, times_fired,
  declined_at)`. Name-the-wall fires once ever per `(node, wall)`; a circling
  notice never repeats for the same node in a later session; a declined offer
  stands without a second ask.
- **Cold start** (journey 1) is one predicate here, not per-feature:
  `baseline_ready(kind)`, an `evidence` minimum, gates everything needing a
  personal baseline ("above your norm", session horizons, the circling check).
  *Which* mechanism switches on *when* is still open in the journeys doc; this
  predicate is where it gets answered.

The escalation ladder's **action** (tempo down, shrink scope, change mode, swap
drill) spends no budget — the doc's own rule is that it acts rather than
narrates, so it is the plan changing. Only the spoken name-the-wall competes
(§9.3).

## 8. The gate criteria schema

Formalised from [`content/gates.toml`](../content/gates.toml). The gain over its
flat bag of recognised-optional fields: the requirement becomes a closed enum, so
an unrepresentable gate fails to parse instead of silently losing a field.

```rust
struct GateCriteria {
    id: GateId, node: NodeId, criterion: String,     // prose, display only
    requirement: Requirement,
    tempo_bpm: u16, click_level: ClickLevel,
    judge: Judge,                                    // Machine | SelfConfirmed
    transport_min: TransportTier, time_ceiling_min: u8, self_rating_logged: bool,
}
enum Requirement {
    CleanPasses   { count: u8, consecutive: bool },
    KeyCoverage   { keys_required: u8, per_key_passes: u8, first_attempt: bool },
    Chained       { min_keys: u8 },
    SelfConfirmed { max_listens: Option<u8>, first_attempt: bool },
}
struct Clean { max_wrong_notes: u8, timing: TimingRule }
enum TimingRule { Consistent { max_cv: f32, max_drift: f32 } }   // decision 6
```

- **Timing is variance and drift only.** A stable 30ms lay-back passes; mean
  offset is *reported* beside swing ratio and never gated (decisions 6, 8).
  `TimingRule` has one variant on purpose — an absolute-deviation variant would
  be a design change, not a config change.
- **Transport gating is a capability check.** `requirement.capabilities_needed()
  ⊆ transport.capabilities()`, else the verdict is `Ask` — never a pass or a
  fail. Decision 7 and never-bluff, in one function.
- **Validation at load** (`validation.rs` idiom): unknown fields error rather
  than being ignored, `schema_version` must match, every `node`/`gate` reference
  must resolve. A dangling reference is a startup failure, because an unreachable
  gate is otherwise invisible.
- Feedback cadence, escalation ladder, circling thresholds, decay half-lives and
  the interruption budget are all data in the same file. Nothing here hard-codes
  a threshold.

## 9. What this contradicts in the source docs

Fix at the source rather than letting the documents diverge.

1. **The mastery-function paragraph** (design doc, Technical architecture) should
   become a pointer here. Its three claims survive; the *how* changed — decay
   applies to the pseudo-counts toward the prior rather than as a separate
   confidence term, and spacing derives from that same state.
2. **Node-level mastery.** "Three mechanisms" reads as a node-level `(estimate,
   confidence)`; the mastery-function line says per `(node, parameter level)`.
   This spec holds **no node-level scalar** — a node figure is derived, for
   display. So `content/nodes.md`'s seeds are *frontier-level* priors and
   `content/README.md`'s checklist should say so.
3. **The escalation ladder is not an interruption.** The design doc says it
   "acts, not narrates", yet the journeys doc and #1147 both give the stuck
   ladder a once-per-session slot. Under §7 only name-the-wall spends budget —
   amend the feedback-choreography section and #1147's list.
4. **The circling check has no free slot.** Layer 2 grants the coaching voice one
   slot "plus whenever the circling check has something to say" — an unbudgeted
   exemption contradicting the journeys doc's "every journey obeys the
   one-interruption budget". Under §7 it competes at its own priority.
5. **`rootless-traversal` is not a gate.** No pass/fail: it is a per-session
   scheduling quota (`new_keys_per_session_min/_max`), unrepresentable as a
   `Requirement`. Move it out of `[gates.*]` into a planner constraint.
6. **`transport = "paper"` conflates two axes** — paper is the absence of a
   transport plus a change of judge. §8 splits them: `transport: Wired |
   Bluetooth | Mic | None` and `judge: Machine | SelfConfirmed`, Phase 0 being
   `(None, SelfConfirmed)`. `[transport_tiers]` should follow.
7. **#1148's fourth question is stale** — decision 17 closed it before the issue
   was read.
