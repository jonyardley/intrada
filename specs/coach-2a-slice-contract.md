# Coach 2a slice: mastery, planner, press-start — the bridge contract

The committed contract for the core+iOS slice covering #1188 (MasteryStore),
#1189 (the five-stage planner) and #1182 (press-start plus the ladder's third
and fourth rungs). Core owns every type below; the iOS shell reads them.

A message-only contract has gone missing mid-session before (#1199), so this
file is the source of truth. If a type here changes, this file changes in the
same commit and the team is told again.

Everything below is scoped by `specs/intrada-coach-engine.md` §2 (mastery), §5
(planner) and §6 (the bridge rules). Where this slice departs from the spec, the
departure is stated at the end.

## 1. What the shell has to change

Two things, both in `ios/Intrada/Views/Screens/`:

1. `CoachEvent::StartDrillLoop` is **gone**, replaced by two events (below). The
   `store.send(.coach(.startDrillLoop(now:)))` call in `DrillLoopHost` does not
   compile after the regen.
2. `CoachView` gains a second field, `plan`, which is what the press-start
   surface renders. `viewModel.coach.drill` is unchanged.

Nothing else on the wire moves. `EngineSession` grows fields, but the shell only
ever carries it as an opaque crash-recovery blob, so no Swift change follows.

## 2. The write half: starting a session

```rust
pub enum CoachEvent {
    /// Plan today's session without running it: the press-start surface reads
    /// the result from `CoachView::plan`. `available_minutes` is `None` until
    /// a surface can ask how long the user has, and falls back to the
    /// `[defaults]` session length.
    PlanSession {
        now: DateTime<Utc>,
        available_minutes: Option<u16>,
    },
    /// Run the plan already made. From `Idle` this plans today's session first,
    /// so a shell that wants to start without a preview can send this alone.
    StartPlannedSession {
        now: DateTime<Utc>,
    },
    // … the rest of the enum is unchanged …
}
```

In Swift:

```swift
store.send(.coach(.planSession(now: SessionClock.nowRFC3339(), availableMinutes: nil)))
store.send(.coach(.startPlannedSession(now: SessionClock.nowRFC3339())))
```

`availableMinutes` is `UInt16?`. Passing `nil` is the 2a path: the session
length comes from the authored `[defaults]`, because the declaration surfaces
(goal / campaign / steer / length) are Phase 2b.

Recovery is unchanged: `DrillLoopHost` still sends `.recoverSession` when a blob
is present, and only falls back to starting a session when there is none.

## 3. The read half: `CoachView`

```rust
pub struct CoachView {
    /// `None` unless a coach session is inside a block. Unchanged.
    pub drill: Option<DrillView>,
    /// `Some` while a session is planned but not yet running: the press-start
    /// surface. Goes to `None` the moment the first block opens.
    pub plan: Option<PlanView>,
}

pub struct PlanView {
    pub blocks: Vec<PlannedBlockView>,
    /// What the plan adds up to, which is what press-start promises.
    pub total_minutes: u16,
    /// What today could not take, in the plan's own words. Rendered as-is;
    /// silent dropping is a defect (spec §5 stage 5).
    pub deferred: Vec<String>,
}

pub struct PlannedBlockView {
    pub drill_title: String,
    /// Where in the material it sits — "A section", "one key, LH alone".
    pub section: Option<String>,
    pub kind: ItemKind,
    pub minutes: u16,
    /// One sentence, written by the core, saying why this block is here.
    /// The shell renders it; it never composes one.
    pub why: String,
}
```

`ItemKind` is the existing bridged enum (`Piece` / `Exercise`), so `TypeBadge`
works on a planned block unchanged.

The first block summary the press-start hero needs is `plan.blocks.first`:
title, section, minutes, and the why line.

## 4. New bridged types behind `EngineSession`

The shell does not read these — they reach the wire only inside the
crash-recovery blob — but they are on it, so they are part of the contract and
carry `Facet` derives plus round-trip tests.

```rust
pub struct Plan {
    pub blocks: Vec<PlannedBlock>,
    /// The seeded dealer, stored so any session replays in a test (spec §5).
    pub rng_seed: u64,
    pub deferred: Vec<String>,
}

pub struct PlannedBlock {
    pub spec: BlockSpec,
    pub why: Why,
    /// What the ladder's third and fourth rungs may draw on. Empty where the
    /// content offers no alternative, which is why a block can still end
    /// `Exit::Escalated`.
    pub alternatives: Vec<Alternative>,
    /// The dealer's new-key quota for this block, where the node authors a
    /// `[traversal.<node>]` entry. `None` where it does not.
    pub new_keys: Option<u8>,
}

pub struct Alternative {
    pub rung: Rung,
    pub spec: BlockSpec,
    pub why: Why,
}

pub struct Why {
    /// The declared destination this block serves, where one is declared.
    pub destination: Option<String>,
    pub node_state: NodeState,
    pub placed_by: Stage,
}

/// Integers, not floats: spec §6's rule, and a percentage is what a why line
/// says out loud anyway.
pub struct NodeState {
    pub estimate_pct: u8,
    /// Attempts beyond the prior (spec §2's `evidence`).
    pub evidence: u16,
    /// `100` is due. Above it, overdue.
    pub overdue_pct: u16,
    pub maturity: Maturity,
}

pub enum Maturity { New, Acquiring, Maintaining }

pub enum Stage { Intent, BackChain, Interleave, GrindCap, Template, Escalation }
```

`Plan::blocks` changes element type from `BlockSpec` to `PlannedBlock`.
`BlockSpec` itself is unchanged.

`BlockState` gains one field:

```rust
pub struct BlockState {
    // … unchanged …
    /// First rep of the day on returning material (spec §2). Decided by the
    /// mastery store, which is the only thing holding `last_attempt_at`.
    pub cold: bool,
}
```

## 5. Core-only types (not on the wire)

`MasteryStore` hangs off `CoachState`, which is `Model` state rather than a
bridge type, so none of this crosses:

```rust
pub struct CoachState {
    pub session: EngineSession,
    pub mastery: MasteryStore,
}

pub struct Mastery {
    pub alpha: f32,
    pub beta: f32,
    pub prior: (f32, f32),
    /// `None` on a prior with no attempt against it yet.
    pub last_attempt_at: Option<DateTime<Utc>>,
}

pub struct MasteryKey { pub node: String, pub level: ParameterLevel }
```

The three readings, defined once (spec §2):

```text
estimate   = alpha / (alpha + beta)
evidence   = (alpha + beta) - (alpha_0 + beta_0)
confidence = evidence / (evidence + k)                       // k = 8
interval_days = base_interval * (1 + estimate) ^ (evidence / e_scale)
overdue       = days_since_last_attempt / interval_days      // due at >= 1
```

The planner's entry point:

```rust
pub fn plan(state: &CoachState, ctx: PlanContext) -> Plan

pub struct PlanContext {
    pub now: DateTime<Utc>,
    pub available_minutes: u16,
    pub rng_seed: u64,
}
```

## 6. Departures from the spec, stated rather than discovered

- **`Plan::seed_drill_loop()` was already retired** by #1180; what #1189 lists
  as the thing to delete is now `Plan::for_today`, which this slice replaces
  with `plan(state, ctx)`. The `#[cfg(test)] Plan::fixture()` stays: the
  state-machine tests read it, so it has a reader.
- **`NodeState` carries integers, not `f32`.** Spec §6's integer rule, and it
  keeps a float off the typegen wire for no loss: the why line says "70%".
- **`Mastery::last_attempt_at` is `Option`**, where spec §2 writes a bare
  `Timestamp`. A prior seeded from content has no attempt behind it, and a
  sentinel date would be a lie the spacing read would then believe.
- **`seed_priors_from_archive()` is not built.** It has no caller until there
  is a migration to call it, and code with no reader gets deleted rather than
  parked (CLAUDE.md). Priors come from the authored content seeds
  (`MasteryStore::seeded_from(&ContentIndex)`), which is what the spec's
  `(estimate, band)` mapping describes. Tracked as a follow-up.
- **The mastery constants are hard-coded in one struct**, not read from
  `gates.toml`: the file authors none of them yet (#1188 permits this
  explicitly). `MasteryConstants` is the single place they live, and moving
  them into the file is a follow-up.
- **The mastery store is still not persisted, and never will be.** It is rebuilt
  at launch from the persisted `BlockRecord`s instead (§7, #1214): the records
  already hold every fact the store derives, so storing the store as well would
  be a second source of truth for the same evidence.

## 7. The read half of the evidence: rebuilding mastery at launch (#1214)

`SaveCoachRecords` (§4 of the engine spec) has written every attempt with its
verdict, level and timestamp since #1181, and nothing read them back. So
`last_attempt_at` only existed for attempts made since launch, which made
planner stage 3's overdue pull and the cold test dead paths for a real user.

The store itself is **not** persisted. It is a projection of the records, and
the records are the log:

```rust
pub enum PersistenceOperation {
    // … unchanged …
    /// Every closed block, tombstones excluded. Read once at a local-first
    /// launch; the core replays the attempts through the mastery track.
    LoadCoachRecords,
}

pub enum PersistenceOutput {
    // … unchanged …
    CoachRecords(Vec<BlockRecord>),
}

pub enum Event {
    // … unchanged …
    CoachStoreLoaded(PersistenceOutput),
}
```

Blocks only. A `WanderRecord` has no node and no level (§4), so it carries
nothing the mastery track is keyed by and nothing to replay.

`Event::StartApp { local_first: true }` requests it alongside `LoadItems` and
`LoadSessions`. `CoachStoreLoaded(CoachRecords(blocks))` calls:

```rust
impl CoachState {
    /// Re-seed from content, then replay every attempt in timestamp order.
    /// Idempotent by construction: the seed is the starting point each time,
    /// so calling it twice cannot double-count evidence.
    pub fn rebuild_mastery(&mut self, records: &[BlockRecord])
}
```

Replay is a projection, in one ordered pass:

- Every attempt of every block becomes `mastery.record(node, level, verdict, at)`.
- A block whose `exit` is `GatePassed` becomes a `level_up` at its `ended_at`.
- The whole stream sorts by timestamp before it runs, stably. Order matters —
  `level_up` declines to overwrite a rung that already has evidence — so a store
  that read the rows back in a different order would hold different priors.

Reconciliation stays in the core (offline-first invariant 4): the shell runs the
typed read and nothing else. `CoachStoreLoaded(Failed)` surfaces an error and
reloads nothing, like `StoreLoaded(Failed)` — a retry against a store that just
failed a read is a loop.

In Swift, the read is a decoder for rows the codec has only ever written:

```swift
protocol ItemStore {
  // … unchanged …
  func loadCoachRecords() throws -> [BlockRecord]
}
```

`SELECT … WHERE deleted_at IS NULL`. The stored enum spellings (`"clean"`,
`"two_and_four"`, `"gate_passed"`, …) that #1181 pinned as the format contract
are now read by production code, and an unrecognised one is reported through
`report(_:)` rather than silently defaulting (#949).
