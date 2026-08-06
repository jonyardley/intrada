//! The coach session state machine. `specs/intrada-coach-engine.md` §4 carries
//! the transition table and records where this scoping of it departs from it.

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use super::content::ContentIndex;
use super::gate::{EvidenceSource, GateProgress, Verdict};
use super::plan::{BlockSpec, Circle, Mode, ParameterLevel, Plan, PlannedBlock};

/// What the shell tells the engine. The whole write half of the bridge surface
/// for the tap-verdict loop (spec §6, as scoped) — the shell reports clicks,
/// taps and seconds, and decides none of them.
// `RecoverSession` carries a whole session and the rest carry a timestamp, so
// the variants differ wildly in size. Boxing it is the usual fix and not one
// available here: this crosses the FFI bridge, where `Box` is not a shape the
// typegen and the bincode wire agree on (same reason as app.rs's allow).
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum CoachEvent {
    /// Plan today's session without running it: the press-start surface reads
    /// the result from `CoachView::plan`. `available_minutes` is `None` until a
    /// surface can ask how long the user has, and falls back to the authored
    /// `[defaults]` length.
    PlanSession {
        now: DateTime<Utc>,
        available_minutes: Option<u16>,
    },
    /// Run the plan already made. From `Idle` this plans today's session first,
    /// so a shell that wants no preview can send this alone.
    StartPlannedSession {
        now: DateTime<Utc>,
    },
    /// One count-in click, reported as it sounds; `remaining` is beats left
    /// *after* this one, counting down to 0 on the last click (#1184).
    CountInBeat {
        remaining: u8,
    },
    /// One click on or after bar 1 beat 1, 0-based, climbing for as long as the
    /// pulse runs. A pass of the phrase ends every `DrillView::phrase_beats`.
    Beat {
        beat_index: u32,
    },
    /// The user's verdict on the rep just played (decision 18).
    Tap {
        clean: bool,
        now: DateTime<Utc>,
    },
    /// Leave the block-entry card and start the block. The ceiling starts
    /// here, not when the card went up.
    StartBlock {
        now: DateTime<Utc>,
    },
    /// Close this block now, with nothing held against the user for it.
    SkipBlock {
        now: DateTime<Utc>,
    },
    /// "Don't count that." A false start, or a pass the user does not want on
    /// the record: no attempt, no evidence, no gate progress, no miss run.
    DiscardAttempt {
        now: DateTime<Utc>,
    },
    /// The click stopped and the shell did not choose to stop it. Reports the
    /// fact only; what it costs the block is the core's call.
    ClickInterrupted {
        now: DateTime<Utc>,
    },
    /// "I'm stuck" — fires the next rung of the ladder now, without waiting
    /// for a run of misses.
    Stuck {
        now: DateTime<Utc>,
    },
    /// One second of wall clock. The ceiling and the gate-open hold are the
    /// core's decisions; this is what lets it make them.
    Tick {
        now: DateTime<Utc>,
    },
    /// The user left the drill screen. Ends the session, not just the block —
    /// the soft-landing exit is what replaces it (2a, #1182).
    LeaveSession {
        now: DateTime<Utc>,
    },
    /// The shell could not start the click, so the drill cannot run. Reported
    /// rather than swallowed, because a silent freeze is the #846 class one
    /// layer up.
    ClickUnavailable {
        now: DateTime<Utc>,
    },
    GoOffPiste {
        now: DateTime<Utc>,
    },
    GoUnmonitored {
        now: DateTime<Utc>,
    },
    /// Answer to the off-piste *keep this as a drill?* prompt.
    KeepWanderAsDrill {
        keep: bool,
    },
    CloseSession {
        now: DateTime<Utc>,
    },
    /// The crash-recovery blob, handed back by the shell at launch (#1181).
    /// The engine re-anchors its clock: the outage is never practice time, and
    /// the minutes already spent are not refunded either.
    RecoverSession {
        session: EngineSession,
        now: DateTime<Utc>,
    },
}

/// What one applied event leaves for the store. The session machine decides
/// what is worth writing; `app.rs` turns this into effects (spec §4).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CoachWrites {
    /// Records to append. Append-only: a record only leaves once, as it closes,
    /// so a crash costs at most the block in flight.
    pub blocks: Vec<BlockRecord>,
    pub wanders: Vec<WanderRecord>,
    /// Attempts the mastery track has yet to hear about (spec §2).
    pub evidence: Vec<ScoredAttempt>,
    pub snapshot: SnapshotAction,
}

/// One attempt, told to the mastery store in the terms it holds state in:
/// `(node, parameter_level)`.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredAttempt {
    pub node: String,
    pub level: ParameterLevel,
    pub verdict: Verdict,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapshotAction {
    #[default]
    Unchanged,
    /// Rewrite the crash-recovery blob: something a recovered session needs
    /// changed.
    Save,
    /// The session is over, so the blob would recover nothing.
    Clear,
}

/// What a recovered session would have to get right. Beats and ticks move the
/// clock and the beat cursor, which recovery restarts anyway, so they must not
/// cost a write per second.
#[derive(PartialEq)]
struct RecoveryKey {
    state: std::mem::Discriminant<SessionState>,
    block: Option<(String, usize, u16, u16)>,
    closed_blocks: usize,
    wanders: usize,
    keep_as_drill: Option<Option<bool>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Phase {
    /// The card a block opens on: silent, one glance, one tap (T1). Only
    /// `StartBlock` and `SkipBlock` leave it.
    BlockEntry,
    CountIn {
        beats_remaining: u8,
    },
    /// Layer 0: no verdict while the hands are still moving.
    Listening,
    AwaitingVerdict,
    /// The ladder acted. Held so the shell restarts the click on the new
    /// parameters before the next count-in.
    Escalating {
        rung: Rung,
    },
    GateOpen,
}

/// The escalation ladder (`content/gates.toml` `[escalation]`). It acts rather
/// than narrates, so firing a rung spends no interruption budget (spec §7).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Rung {
    TempoDown,
    ShrinkScope,
    ChangeMode,
    SwapDrill,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Exit {
    GatePassed,
    CeilingHit,
    Skipped,
    Escalated,
    SessionEnded,
}

/// One scored attempt — the evidence unit (spec §2). `self_predicted` stays
/// unused until the scoring path returns; a pre-play prediction against a
/// tap-verdict was considered and cut (§3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct AttemptSummary {
    pub at: DateTime<Utc>,
    pub verdict: Verdict,
    pub source: EvidenceSource,
    /// First rep of the block: the highest-information tap-verdict.
    pub cold: bool,
    pub self_predicted: Option<Verdict>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BlockRecord {
    pub id: String,
    pub node: String,
    pub drill: String,
    pub gate: String,
    pub level: ParameterLevel,
    pub circle: Circle,
    pub mode: Mode,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub attempts: Vec<AttemptSummary>,
    pub attempts_to_pass: Option<u16>,
    pub gate_opened_at_attempt: Option<u16>,
    pub reps_after_gate: u16,
    pub active_ms: u64,
    pub escalation_fired: Vec<Rung>,
    pub exit: Exit,
}

/// A wander has no node, drill, gate or level, so it gets its own type rather
/// than turning every id on `BlockRecord` into an `Option` (spec §4).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct WanderRecord {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub attempts: Vec<AttemptSummary>,
    /// `None` = not yet asked.
    pub keep_as_drill: Option<bool>,
}

/// Thresholds the engine must not hard-code: `[escalation]` in
/// `content/gates.toml` is where they live (spec §8's closing rule). Carried on
/// the session so a recovered one keeps the thresholds it ran with.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct EngineConfig {
    pub consecutive_fail_trigger: u8,
    pub ladder: Vec<Rung>,
    pub gate_open_hold_s: u32,
    pub tempo_down_pct: u16,
    pub tempo_floor_bpm: u16,
}

impl Default for EngineConfig {
    fn default() -> Self {
        let escalation = &ContentIndex::shipped().escalation;
        Self {
            consecutive_fail_trigger: escalation.consecutive_fail_trigger,
            ladder: escalation.ladder.clone(),
            gate_open_hold_s: escalation.gate_open_hold_s,
            tempo_down_pct: escalation.tempo_down_pct,
            tempo_floor_bpm: escalation.tempo_floor_bpm,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BlockState {
    pub id: String,
    pub spec_index: usize,
    pub phase: Phase,
    pub started_at: DateTime<Utc>,
    pub now: DateTime<Utc>,
    /// Mutable: the ladder's first rung drops the tempo.
    pub level: ParameterLevel,
    /// Mutable: the ladder's second rung shrinks the scope.
    pub bars: u16,
    pub beats_per_bar: u8,
    pub count_in_beats: u8,
    pub beat_index: u32,
    pub attempts: Vec<AttemptSummary>,
    pub consecutive_fails: u8,
    pub gate_progress: GateProgress,
    pub escalation_fired: Vec<Rung>,
    /// What the last tap said, so the glance can draw until the first
    /// count-in click.
    pub last_verdict: Option<Verdict>,
    /// Bumped when the shell must stop the click and start it again, count-in
    /// first. A tap, a discard and a phrase boundary all leave it alone.
    pub pulse_seq: u32,
    /// The pass in flight is a false start the user has already written off,
    /// so the boundary it reaches opens no verdict window.
    discarded: bool,
    /// What earlier stretches of this block took. Resuming after an
    /// interruption carries on from it rather than refunding it.
    spent_ms: u64,
    pub gate_opened_at_attempt: Option<u16>,
    pub reps_after_gate: u16,
    /// Whether this block's first rep is a cold test: decided by the mastery
    /// store, which is the only thing that knows when the material was last
    /// played (spec §2).
    pub cold: bool,
    gate_open_since: Option<DateTime<Utc>>,
}

impl BlockState {
    fn open(spec: &BlockSpec, spec_index: usize, now: DateTime<Utc>) -> Self {
        Self {
            id: ulid::Ulid::generate().to_string(),
            spec_index,
            phase: Phase::BlockEntry,
            started_at: now,
            now,
            level: spec.level,
            bars: spec.bars,
            beats_per_bar: spec.beats_per_bar,
            count_in_beats: spec.count_in_beats,
            beat_index: 0,
            attempts: Vec::new(),
            consecutive_fails: 0,
            gate_progress: GateProgress::new(&spec.gate.requirement),
            escalation_fired: Vec::new(),
            last_verdict: None,
            pulse_seq: 1,
            discarded: false,
            spent_ms: 0,
            gate_opened_at_attempt: None,
            reps_after_gate: 0,
            cold: false,
            gate_open_since: None,
        }
    }

    /// The click's parameters changed, so the pulse cannot carry on.
    fn restart_pulse(&mut self) {
        self.beat_index = 0;
        self.last_verdict = None;
        self.discarded = false;
        self.pulse_seq = self.pulse_seq.saturating_add(1);
    }

    /// Beats in one pass of the phrase. The landing beat the player aims at is
    /// the next pass's downbeat, since the pulse never stops between them.
    pub fn body_beats(&self) -> u32 {
        u32::from(self.bars.max(1)) * u32::from(self.beats_per_bar.max(1))
    }

    /// A card bills nothing while it is up, started or interrupted back to it.
    fn active_ms(&self) -> u64 {
        self.spent_ms + self.running_ms()
    }

    fn running_ms(&self) -> u64 {
        if self.phase == Phase::BlockEntry {
            return 0;
        }
        (self.now - self.started_at).num_milliseconds().max(0) as u64
    }

    pub fn elapsed_seconds(&self) -> u32 {
        (self.active_ms() / 1000) as u32
    }

    /// The musician's 1-based counting, within the phrase: the beat the shell
    /// reports climbs for the whole block, and nobody counts bar 41.
    pub fn bar(&self) -> u16 {
        ((self.beat_index % self.body_beats()) / u32::from(self.beats_per_bar.max(1))) as u16 + 1
    }

    pub fn beat(&self) -> u8 {
        (self.beat_index % u32::from(self.beats_per_bar.max(1))) as u8 + 1
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SessionState {
    #[default]
    Idle,
    Planned {
        plan: Plan,
    },
    Running {
        plan: Plan,
        block: BlockState,
    },
    /// A peer of `Running`, not a sub-state: no plan and no gates, still capturing.
    OffPiste {
        started_at: DateTime<Utc>,
    },
    /// Nothing captured, nothing inferred (decision 16).
    Unmonitored {
        started_at: DateTime<Utc>,
    },
    /// Spec §4 carries a `Summary` here; it arrives with the closing screen.
    Closing,
}

impl SessionState {
    /// Whether a crash here would lose something the engine cannot rebuild.
    /// Only these states earn a blob: a plan is remade from the content and the
    /// clock, and remade is better, since a plan carries the hour it was made
    /// at. Saving a `Planned` session dead-ended press-start, because the shell
    /// reads a blob as "recover" and recovering a plan hands back no drill
    /// (#1219).
    fn worth_recovering(&self) -> bool {
        matches!(
            self,
            SessionState::Running { .. }
                | SessionState::OffPiste { .. }
                | SessionState::Unmonitored { .. }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct EngineSession {
    pub state: SessionState,
    pub config: EngineConfig,
    pub closed_blocks: Vec<BlockRecord>,
    pub wanders: Vec<WanderRecord>,
    /// Banked but never interpreted — the whole point of decision 16.
    pub unmonitored_seconds: u32,
}

impl EngineSession {
    pub fn block(&self) -> Option<&BlockState> {
        match &self.state {
            SessionState::Running { block, .. } => Some(block),
            _ => None,
        }
    }

    pub fn phase(&self) -> Option<&Phase> {
        self.block().map(|block| &block.phase)
    }

    pub fn spec(&self) -> Option<&BlockSpec> {
        match &self.state {
            SessionState::Running { plan, block, .. } => plan
                .blocks
                .get(block.spec_index)
                .map(|planned| &planned.spec),
            _ => None,
        }
    }

    fn recovery_key(&self) -> RecoveryKey {
        RecoveryKey {
            state: std::mem::discriminant(&self.state),
            block: self.block().map(|block| {
                (
                    block.id.clone(),
                    block.attempts.len(),
                    block.level.tempo_bpm,
                    block.bars,
                )
            }),
            closed_blocks: self.closed_blocks.len(),
            wanders: self.wanders.len(),
            keep_as_drill: self.wanders.last().map(|wander| wander.keep_as_drill),
        }
    }

    /// The mastery store owns the cold decision, because it is the only thing
    /// holding `last_attempt_at` (spec §2). The session carries the answer onto
    /// the attempts of the block that has not started scoring yet.
    pub(crate) fn mark_cold(&mut self, cold: bool) {
        if let Some(block) = self.block_mut() {
            if block.attempts.is_empty() {
                block.cold = cold;
            }
        }
    }

    pub fn apply(&mut self, event: &CoachEvent) -> CoachWrites {
        self.apply_with_plan(event, None)
    }

    /// `plan` is today's session, which only `CoachState` can work out: the
    /// planner reads the mastery track, and the session is inside it.
    pub fn apply_with_plan(&mut self, event: &CoachEvent, plan: Option<Plan>) -> CoachWrites {
        let before = self.recovery_key();
        let was_closing = matches!(self.state, SessionState::Closing);
        let blocks_written = self.closed_blocks.len();
        let wanders_written = self.wanders.len();

        let evidence = self.handle(event, plan);

        // Recovery replaces the session wholesale, so it can hand back fewer
        // records than the live one had (a swallowed snapshot write leaves an
        // older blob on disk). It also offers every recovered record again: the
        // blob surviving the crash is no evidence that its records reached the
        // store, and the store upserts by id, so a repeat costs nothing.
        let (blocks_from, wanders_from) = if matches!(event, CoachEvent::RecoverSession { .. }) {
            (0, 0)
        } else {
            (
                blocks_written.min(self.closed_blocks.len()),
                wanders_written.min(self.wanders.len()),
            )
        };

        let mut writes = CoachWrites {
            blocks: self.closed_blocks[blocks_from..].to_vec(),
            wanders: self.wanders[wanders_from..].to_vec(),
            evidence: evidence.into_iter().collect(),
            snapshot: if matches!(self.state, SessionState::Closing) {
                if was_closing {
                    SnapshotAction::Unchanged
                } else {
                    SnapshotAction::Clear
                }
            } else if !self.state.worth_recovering() || self.recovery_key() == before {
                SnapshotAction::Unchanged
            } else {
                SnapshotAction::Save
            },
        };
        // The keep prompt answers a row the store already holds.
        if matches!(event, CoachEvent::KeepWanderAsDrill { .. }) {
            writes.wanders.extend(self.wanders.last().cloned());
        }
        writes
    }

    fn handle(&mut self, event: &CoachEvent, planned: Option<Plan>) -> Option<ScoredAttempt> {
        match event {
            CoachEvent::PlanSession { .. } => {
                if self.accepts_plan(event) {
                    if let Some(plan) = planned {
                        self.state = SessionState::Planned { plan };
                    }
                }
            }
            CoachEvent::StartPlannedSession { now } => match std::mem::take(&mut self.state) {
                SessionState::Planned { plan } => self.start(plan, *now),
                SessionState::Idle => {
                    if let Some(plan) = planned {
                        self.start(plan, *now);
                    }
                }
                running_or_closing => self.state = running_or_closing,
            },
            CoachEvent::CountInBeat { remaining } => self.count_in(*remaining),
            CoachEvent::Beat { beat_index } => self.beat(*beat_index),
            CoachEvent::Tap { clean, now } => return self.tap(*clean, *now),
            CoachEvent::StartBlock { now } => self.start_block(*now),
            CoachEvent::SkipBlock { now } => self.skip_block(*now),
            CoachEvent::DiscardAttempt { now } => self.discard(*now),
            CoachEvent::ClickInterrupted { now } => self.click_interrupted(*now),
            CoachEvent::Stuck { now } => self.stuck(*now),
            CoachEvent::Tick { now } => self.tick(*now),
            CoachEvent::LeaveSession { now } => self.leave_session(*now),
            CoachEvent::ClickUnavailable { now } => self.leave_session(*now),
            CoachEvent::GoOffPiste { now } => self.go_off_piste(*now),
            CoachEvent::GoUnmonitored { now } => self.go_unmonitored(*now),
            CoachEvent::KeepWanderAsDrill { keep } => {
                if let Some(wander) = self.wanders.last_mut() {
                    wander.keep_as_drill = Some(*keep);
                }
            }
            CoachEvent::CloseSession { now } => self.close_session(*now),
            CoachEvent::RecoverSession { session, now } => self.recover(session.clone(), *now),
        }
        None
    }

    /// When the mastery store's cold read applies: the events that carry a
    /// clock. A tap carries its own, which is the one that matters.
    pub fn now_of(event: &CoachEvent) -> Option<DateTime<Utc>> {
        match event {
            CoachEvent::PlanSession { now, .. }
            | CoachEvent::StartPlannedSession { now }
            | CoachEvent::Tap { now, .. }
            | CoachEvent::StartBlock { now }
            | CoachEvent::SkipBlock { now }
            | CoachEvent::DiscardAttempt { now }
            | CoachEvent::ClickInterrupted { now }
            | CoachEvent::Stuck { now }
            | CoachEvent::Tick { now }
            | CoachEvent::LeaveSession { now }
            | CoachEvent::ClickUnavailable { now }
            | CoachEvent::GoOffPiste { now }
            | CoachEvent::GoUnmonitored { now }
            | CoachEvent::CloseSession { now }
            | CoachEvent::RecoverSession { now, .. } => Some(*now),
            CoachEvent::CountInBeat { .. }
            | CoachEvent::Beat { .. }
            | CoachEvent::KeepWanderAsDrill { .. } => None,
        }
    }

    /// The clock stopped when the app died. A running block keeps the time it
    /// had already spent against its ceiling and comes back at a count-in; a
    /// wander banks none of the outage, because nothing knows what happened
    /// while the app was gone.
    fn recover(&mut self, mut session: EngineSession, now: DateTime<Utc>) {
        match &mut session.state {
            SessionState::Running { block, .. } => {
                let spent = (block.now - block.started_at).num_milliseconds().max(0);
                block.started_at = now - TimeDelta::milliseconds(spent);
                block.now = now;
                block.beat_index = 0;
                block.last_verdict = None;
                block.discarded = false;
                block.pulse_seq = block.pulse_seq.saturating_add(1);
                if block.phase == Phase::BlockEntry {
                    // The card is where a user pauses, so coming back must not
                    // start a block they never tapped Start on.
                    block.started_at = now;
                    block.gate_open_since = None;
                } else if block.gate_progress.satisfied() && block.gate_opened_at_attempt.is_some()
                {
                    block.gate_open_since = Some(now);
                    block.phase = Phase::GateOpen;
                } else {
                    block.gate_open_since = None;
                    block.phase = Phase::CountIn {
                        beats_remaining: block.count_in_beats,
                    };
                }
            }
            SessionState::OffPiste { started_at } | SessionState::Unmonitored { started_at } => {
                *started_at = now;
            }
            SessionState::Idle | SessionState::Planned { .. } | SessionState::Closing => {}
        }
        *self = session;
    }

    pub(crate) fn start(&mut self, plan: Plan, now: DateTime<Utc>) {
        let Some(planned) = plan.blocks.first() else {
            self.state = SessionState::Planned { plan };
            return;
        };
        let block = BlockState::open(&planned.spec, 0, now);
        self.state = SessionState::Running { plan, block };
    }

    fn count_in(&mut self, remaining: u8) {
        let Some(block) = self.block_mut() else {
            return;
        };
        if matches!(
            block.phase,
            Phase::CountIn { .. } | Phase::Escalating { .. }
        ) {
            block.phase = Phase::CountIn {
                beats_remaining: remaining,
            };
        }
    }

    fn beat(&mut self, beat_index: u32) {
        let Some(block) = self.block_mut() else {
            return;
        };
        let phrase = block.body_beats();
        match block.phase {
            // A pulse reports its first body beat as 0, so anything else here
            // is a straggler from the pulse just torn down (the click drains
            // asynchronously) and adopting it would cost the first pass.
            Phase::CountIn { .. } | Phase::Escalating { .. } => {
                if beat_index != 0 {
                    return;
                }
                block.beat_index = 0;
                block.last_verdict = None;
                block.phase = Phase::Listening;
            }
            Phase::Listening | Phase::AwaitingVerdict => {
                // A beat that does not advance is a repeat the click already
                // reported: obeying it would open a second window on one pass.
                if beat_index <= block.beat_index {
                    return;
                }
                let crossed = beat_index / phrase > block.beat_index / phrase;
                block.beat_index = beat_index;
                if crossed {
                    block.last_verdict = None;
                    block.phase = if std::mem::take(&mut block.discarded) {
                        Phase::Listening
                    } else {
                        Phase::AwaitingVerdict
                    };
                } else if block.phase != Phase::AwaitingVerdict {
                    block.last_verdict = None;
                    block.phase = Phase::Listening;
                }
            }
            Phase::BlockEntry | Phase::GateOpen => {}
        }
    }

    fn start_block(&mut self, now: DateTime<Utc>) {
        let Some(block) = self.block_mut() else {
            return;
        };
        if block.phase != Phase::BlockEntry {
            return;
        }
        block.started_at = now;
        block.now = now;
        block.phase = Phase::CountIn {
            beats_remaining: block.count_in_beats,
        };
    }

    fn skip_block(&mut self, now: DateTime<Utc>) {
        let Some(block) = self.block_mut() else {
            return;
        };
        block.now = now;
        self.close_block(Exit::Skipped, true);
    }

    fn click_interrupted(&mut self, now: DateTime<Utc>) {
        let Some(block) = self.block_mut() else {
            return;
        };
        if block.phase == Phase::BlockEntry {
            return;
        }
        block.now = now;
        // A pass earned before the interruption is still a pass, and the card
        // has no way to hold an open gate.
        if block.phase == Phase::GateOpen {
            self.close_block(Exit::GatePassed, true);
            return;
        }
        block.spent_ms = block.active_ms();
        block.started_at = now;
        block.beat_index = 0;
        block.last_verdict = None;
        block.discarded = false;
        block.phase = Phase::BlockEntry;
    }

    fn discard(&mut self, now: DateTime<Utc>) {
        let Some(block) = self.block_mut() else {
            return;
        };
        // With a window open this voids the pass it asks about, not the one in
        // flight. Voiding the pass in flight reads as well: open, see #1237.
        match block.phase {
            Phase::Listening => block.discarded = true,
            Phase::AwaitingVerdict => block.phase = Phase::Listening,
            _ => return,
        }
        block.now = now;
        block.last_verdict = None;
    }

    fn tap(&mut self, clean: bool, now: DateTime<Utc>) -> Option<ScoredAttempt> {
        let trigger = self.config.consecutive_fail_trigger;
        let node = self.spec()?.node.clone();
        let block = self.block_mut()?;
        if block.phase != Phase::AwaitingVerdict {
            return None;
        }

        let verdict = if clean {
            Verdict::Clean
        } else {
            Verdict::Missed
        };
        block.now = now;
        block.attempts.push(AttemptSummary {
            at: now,
            verdict,
            source: EvidenceSource::TapVerdict,
            cold: block.cold && block.attempts.is_empty(),
            self_predicted: None,
        });
        let scored = ScoredAttempt {
            node,
            level: block.level,
            verdict,
            at: now,
        };
        if block.gate_opened_at_attempt.is_some() {
            block.reps_after_gate += 1;
        }
        block.gate_progress.record(verdict);
        block.consecutive_fails = if clean {
            0
        } else {
            block.consecutive_fails.saturating_add(1)
        };
        block.last_verdict = Some(verdict);

        self.resolve_tap(trigger);
        Some(scored)
    }

    /// The spec's three `Verdict | … |` rows, in one place.
    fn resolve_tap(&mut self, trigger: u8) {
        let Some(block) = self.block_mut() else {
            return;
        };

        if block.gate_progress.satisfied() && block.gate_opened_at_attempt.is_none() {
            let attempt = block.attempts.len() as u16;
            block.gate_opened_at_attempt = Some(attempt);
            block.gate_open_since = Some(block.now);
            block.phase = Phase::GateOpen;
            return;
        }

        if trigger > 0 && block.consecutive_fails >= trigger {
            if !self.escalate() {
                self.close_block(Exit::Escalated, true);
            }
            return;
        }

        // `last_verdict` is left set: it is what the glance draws until the
        // next beat clears it.
        block.phase = Phase::Listening;
    }

    /// `false` = the ladder has no rung left the engine can act on, so the
    /// caller ends the block instead of pretending to escalate.
    fn escalate(&mut self) -> bool {
        loop {
            let Some(rung) = self.next_rung() else {
                return false;
            };
            // A rung is spent whether or not it could act: restarting an
            // identical rep is not an escalation, so the ladder moves on.
            match rung {
                // These two change the material rather than the parameters, so
                // they close this block and open the alternative the plan
                // carried (#1182).
                Rung::ChangeMode | Rung::SwapDrill => {
                    self.spend_rung(rung);
                    if self.draw_alternative(rung) {
                        self.close_block(Exit::Escalated, true);
                        return true;
                    }
                }
                Rung::TempoDown | Rung::ShrinkScope => {
                    let acted = self.change_parameter(rung);
                    self.spend_rung(rung);
                    if acted {
                        let Some(block) = self.block_mut() else {
                            return false;
                        };
                        block.consecutive_fails = 0;
                        block.restart_pulse();
                        block.phase = Phase::Escalating { rung };
                        return true;
                    }
                }
            }
        }
    }

    /// Whether a fresh plan would be used if one were handed over. The shell
    /// plans on every arrival at Practice, so planning has to be inert
    /// mid-session: replacing the state would drop the block in flight with no
    /// `BlockRecord` behind it (#1219). `CoachState` reads this too, so it does
    /// not run the planner for an answer that would be discarded.
    pub(crate) fn accepts_plan(&self, event: &CoachEvent) -> bool {
        match event {
            CoachEvent::PlanSession { .. } => matches!(
                self.state,
                SessionState::Idle | SessionState::Planned { .. }
            ),
            // A plan already made wins; from anywhere else the event is ignored.
            CoachEvent::StartPlannedSession { .. } => matches!(self.state, SessionState::Idle),
            _ => false,
        }
    }

    fn next_rung(&self) -> Option<Rung> {
        let block = self.block()?;
        self.config
            .ladder
            .get(block.escalation_fired.len())
            .copied()
    }

    fn spend_rung(&mut self, rung: Rung) {
        if let Some(block) = self.block_mut() {
            block.escalation_fired.push(rung);
        }
    }

    /// `false` = already at the tempo floor or down to a single bar, so this
    /// rung would change nothing.
    fn change_parameter(&mut self, rung: Rung) -> bool {
        let pct = self.config.tempo_down_pct.min(100);
        let floor = self.config.tempo_floor_bpm;
        let Some(block) = self.block_mut() else {
            return false;
        };
        match rung {
            Rung::TempoDown => {
                let dropped = block
                    .level
                    .tempo_bpm
                    .saturating_sub(
                        (u32::from(block.level.tempo_bpm) * u32::from(pct) / 100) as u16,
                    )
                    .max(floor);
                let moved = dropped < block.level.tempo_bpm;
                block.level.tempo_bpm = dropped;
                moved
            }
            Rung::ShrinkScope => {
                let shrunk = (block.bars / 2).max(1);
                let moved = shrunk < block.bars;
                block.bars = shrunk;
                moved
            }
            Rung::ChangeMode | Rung::SwapDrill => false,
        }
    }

    /// The plan supplies the alternative; this only moves the cursor onto it, by
    /// putting it next in the plan so the closing block hands straight over.
    fn draw_alternative(&mut self, rung: Rung) -> bool {
        let SessionState::Running { plan, block } = &mut self.state else {
            return false;
        };
        let index = block.spec_index;
        let Some(planned) = plan.blocks.get_mut(index) else {
            return false;
        };
        let Some(offer) = planned
            .alternatives
            .iter()
            .position(|alternative| alternative.rung == rung)
        else {
            return false;
        };
        let drawn = planned.alternatives.remove(offer);
        plan.blocks.insert(
            index + 1,
            PlannedBlock {
                spec: drawn.spec,
                why: drawn.why,
                alternatives: Vec::new(),
                new_keys: None,
            },
        );
        true
    }

    fn stuck(&mut self, now: DateTime<Utc>) {
        {
            let Some(block) = self.block_mut() else {
                return;
            };
            if !matches!(block.phase, Phase::Listening | Phase::AwaitingVerdict) {
                return;
            }
            block.now = now;
        }
        if !self.escalate() {
            self.close_block(Exit::Escalated, true);
        }
    }

    fn tick(&mut self, now: DateTime<Utc>) {
        let ceiling = self.spec().map(|spec| u32::from(spec.minutes) * 60);
        let hold = self.config.gate_open_hold_s;
        let Some(block) = self.block_mut() else {
            return;
        };
        block.now = now;

        let held = block.phase == Phase::GateOpen
            && block
                .gate_open_since
                .is_some_and(|since| (now - since).num_seconds() >= i64::from(hold));
        // A card bills nothing, so it cannot run out of time while it is up.
        let over = block.phase != Phase::BlockEntry
            && ceiling.is_some_and(|ceiling| block.elapsed_seconds() >= ceiling);

        if held {
            self.close_block(Exit::GatePassed, true);
        } else if over {
            self.close_block(Exit::CeilingHit, true);
        }
    }

    fn leave_session(&mut self, now: DateTime<Utc>) {
        match self.block_mut() {
            Some(block) => {
                block.now = now;
                self.close_block(Exit::SessionEnded, false);
            }
            // Nothing was running, so there is no record to write, but the
            // session still has to close: an open one leaves a blob that would
            // recover a session the user has already left.
            None => self.state = SessionState::Closing,
        }
    }

    fn go_off_piste(&mut self, now: DateTime<Utc>) {
        match &mut self.state {
            SessionState::Running { block, .. } => {
                block.now = now;
                self.close_block(Exit::Skipped, false);
                self.state = SessionState::OffPiste { started_at: now };
            }
            SessionState::Planned { .. } => {
                self.state = SessionState::OffPiste { started_at: now };
            }
            _ => {}
        }
    }

    fn go_unmonitored(&mut self, now: DateTime<Utc>) {
        // Never from mid-`Running`: switching part-way would make the
        // already-captured half of the session retrospectively unconsented.
        if matches!(
            self.state,
            SessionState::Idle | SessionState::Planned { .. }
        ) {
            self.state = SessionState::Unmonitored { started_at: now };
        }
    }

    fn close_session(&mut self, now: DateTime<Utc>) {
        match self.state {
            SessionState::OffPiste { started_at } => {
                self.wanders.push(WanderRecord {
                    id: ulid::Ulid::generate().to_string(),
                    started_at,
                    ended_at: now,
                    attempts: Vec::new(),
                    keep_as_drill: None,
                });
                self.state = SessionState::Closing;
            }
            SessionState::Unmonitored { started_at } => {
                self.unmonitored_seconds = (now - started_at).num_seconds().max(0) as u32;
                self.state = SessionState::Closing;
            }
            SessionState::Running { .. } => self.leave_session(now),
            _ => self.state = SessionState::Closing,
        }
    }

    /// Abandoning must never be what the app teaches, so a closed block either
    /// hands on to the next or closes the session (spec §4).
    fn close_block(&mut self, exit: Exit, advance: bool) {
        let SessionState::Running { plan, block } = &mut self.state else {
            return;
        };
        let Some(spec) = plan
            .blocks
            .get(block.spec_index)
            .map(|planned| &planned.spec)
        else {
            return;
        };

        self.closed_blocks.push(BlockRecord {
            id: block.id.clone(),
            node: spec.node.clone(),
            drill: spec.drill.clone(),
            gate: spec.gate.id.clone(),
            level: block.level,
            circle: spec.circle,
            mode: spec.mode,
            started_at: block.started_at,
            ended_at: block.now,
            attempts: block.attempts.clone(),
            attempts_to_pass: block.gate_opened_at_attempt,
            gate_opened_at_attempt: block.gate_opened_at_attempt,
            reps_after_gate: block.reps_after_gate,
            active_ms: block.active_ms(),
            escalation_fired: block.escalation_fired.clone(),
            exit,
        });

        let next = block.spec_index + 1;
        let now = block.now;
        match plan.blocks.get(next).filter(|_| advance) {
            Some(planned) => *block = BlockState::open(&planned.spec, next, now),
            None => self.state = SessionState::Closing,
        }
    }

    /// The state-machine tests run a fixed plan rather than today's, so what
    /// they assert is the machine and not whatever the content now says.
    #[cfg(test)]
    pub(crate) fn start_fixture(&mut self, now: DateTime<Utc>) {
        self.start(Plan::fixture(), now);
    }

    fn block_mut(&mut self) -> Option<&mut BlockState> {
        match &mut self.state {
            SessionState::Running { block, .. } => Some(block),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::gate::Requirement;
    use chrono::TimeZone;

    fn at(second: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_754_300_000 + second, 0).unwrap()
    }

    /// Started, count-in done, one body beat played — the state every rep runs in.
    fn listening() -> EngineSession {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));
        session.apply(&CoachEvent::StartBlock { now: at(0) });
        session.apply(&CoachEvent::Beat { beat_index: 0 });
        session
    }

    /// A fixture whose plan has somewhere to go, so a close lands on a card
    /// rather than ending the session. Left on block 1's card.
    fn two_block_session_carded() -> EngineSession {
        let mut plan = Plan::fixture();
        let mut second = plan.blocks[0].clone();
        second.spec.drill_title = "Shells".to_string();
        second.spec.node = "shells-ii-v-i".to_string();
        plan.blocks.push(second);

        let mut session = EngineSession::default();
        session.start(plan, at(0));
        session
    }

    fn two_block_session() -> EngineSession {
        let mut session = two_block_session_carded();
        session.apply(&CoachEvent::StartBlock { now: at(0) });
        session.apply(&CoachEvent::Beat { beat_index: 0 });
        session
    }

    /// On to the next phrase boundary, wherever the climbing beat has got to.
    fn play_to_the_end(session: &mut EngineSession) {
        let block = session.block().expect("a running block");
        let phrase = block.body_beats();
        let boundary = (block.beat_index / phrase + 1) * phrase;
        session.apply(&CoachEvent::Beat {
            beat_index: boundary,
        });
    }

    /// One whole repetition: play it out and answer. Unconditional, so a broken
    /// transition fails where it broke.
    fn rep(session: &mut EngineSession, clean: bool, second: i64) {
        play_to_the_end(session);
        session.apply(&CoachEvent::Tap {
            clean,
            now: at(second),
        });
    }

    // ── The block-entry card ──

    #[test]
    fn a_session_opens_on_the_first_block_card_rather_than_a_count_in() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));

        assert_eq!(
            session.phase(),
            Some(&Phase::BlockEntry),
            "one glance and one tap before the hands move (T1)"
        );
    }

    #[test]
    fn the_card_is_silent_until_it_is_started() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));

        session.apply(&CoachEvent::CountInBeat { remaining: 3 });
        session.apply(&CoachEvent::Beat { beat_index: 0 });
        assert_eq!(
            session.phase(),
            Some(&Phase::BlockEntry),
            "nothing is sounding, so nothing can be reported"
        );
    }

    #[test]
    fn starting_a_block_begins_its_count_in_and_its_ceiling() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));

        session.apply(&CoachEvent::Tick { now: at(120) });
        session.apply(&CoachEvent::StartBlock { now: at(120) });

        assert_eq!(
            session.phase(),
            Some(&Phase::CountIn { beats_remaining: 4 })
        );
        assert_eq!(
            session.block().unwrap().elapsed_seconds(),
            0,
            "reading the card is not practice time"
        );
    }

    #[test]
    fn a_stray_start_cannot_hand_a_running_block_a_fresh_ceiling() {
        let mut session = listening();
        session.apply(&CoachEvent::Tick { now: at(300) });

        session.apply(&CoachEvent::StartBlock { now: at(300) });

        assert_eq!(
            session.block().unwrap().elapsed_seconds(),
            300,
            "a second Start must not refund the minutes already spent"
        );
        assert_eq!(session.phase(), Some(&Phase::Listening));
    }

    #[test]
    fn the_ceiling_cannot_run_out_while_the_card_is_still_up() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));

        session.apply(&CoachEvent::Tick { now: at(3600) });
        assert!(
            session.closed_blocks.is_empty(),
            "a block nobody has started cannot hit its ceiling"
        );
    }

    #[test]
    fn a_closed_block_lands_on_the_next_card_rather_than_auto_advancing() {
        let mut session = two_block_session();
        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        rep(&mut session, true, 27);
        session.apply(&CoachEvent::Tick { now: at(30) });

        assert_eq!(session.block().map(|block| block.spec_index), Some(1));
        assert_eq!(
            session.phase(),
            Some(&Phase::BlockEntry),
            "a breath between blocks, not a count-in the hands did not ask for"
        );
    }

    #[test]
    fn a_recovered_card_is_still_a_card() {
        let mut crashed = EngineSession::default();
        crashed.start_fixture(at(0));
        crashed.apply(&CoachEvent::Tick { now: at(30) });

        let mut restored = EngineSession::default();
        restored.apply(&CoachEvent::RecoverSession {
            session: crashed,
            now: at(3600),
        });

        assert_eq!(
            restored.phase(),
            Some(&Phase::BlockEntry),
            "backgrounding on the card is where a user pauses; coming back to a \
             count-in starts a block they never tapped Start on"
        );
        assert_eq!(restored.block().unwrap().elapsed_seconds(), 0);
    }

    // ── Skip ──

    #[test]
    fn skipping_from_the_card_closes_the_block_and_moves_on() {
        let mut session = two_block_session_carded();

        session.apply(&CoachEvent::SkipBlock { now: at(5) });

        let record = session.closed_blocks.last().expect("a record for the skip");
        assert_eq!(record.exit, Exit::Skipped);
        assert!(
            record.attempts.is_empty(),
            "a skip is a fact worth keeping, not an attempt"
        );
        assert_eq!(record.active_ms, 0, "and no practice time against it");
        assert_eq!(session.block().map(|block| block.spec_index), Some(1));
        assert_eq!(session.phase(), Some(&Phase::BlockEntry));
    }

    #[test]
    fn a_block_skipped_from_its_card_bills_no_practice_time() {
        let mut session = two_block_session_carded();
        session.apply(&CoachEvent::Tick { now: at(600) });

        session.apply(&CoachEvent::SkipBlock { now: at(600) });

        let record = session.closed_blocks.last().expect("a record for the skip");
        assert_eq!(
            record.active_ms, 0,
            "ten minutes of a card sitting there is not ten minutes of practice"
        );
    }

    #[test]
    fn leaving_from_a_card_bills_no_practice_time_either() {
        let mut session = two_block_session_carded();
        session.apply(&CoachEvent::Tick { now: at(600) });

        session.apply(&CoachEvent::LeaveSession { now: at(600) });

        assert_eq!(session.closed_blocks.last().expect("a record").active_ms, 0);
    }

    #[test]
    fn skipping_mid_block_keeps_the_evidence_already_earned() {
        let mut session = two_block_session();
        rep(&mut session, true, 9);

        session.apply(&CoachEvent::SkipBlock { now: at(20) });

        let record = session.closed_blocks.last().expect("a record");
        assert_eq!(record.exit, Exit::Skipped);
        assert_eq!(
            record.attempts.len(),
            1,
            "the tap already given still counts"
        );
    }

    #[test]
    fn skipping_the_last_block_closes_the_session() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));

        session.apply(&CoachEvent::SkipBlock { now: at(5) });

        assert_eq!(session.state, SessionState::Closing);
        assert_eq!(session.closed_blocks[0].exit, Exit::Skipped);
    }

    // ── Interruption ──

    #[test]
    fn an_interruption_parks_the_block_on_its_card() {
        let mut session = listening();

        session.apply(&CoachEvent::ClickInterrupted { now: at(30) });

        assert_eq!(
            session.phase(),
            Some(&Phase::BlockEntry),
            "the card is the re-entry point: Start to carry on, Skip to move on"
        );
    }

    #[test]
    fn an_interruption_keeps_every_pass_already_banked() {
        let mut session = listening();
        rep(&mut session, true, 9);
        rep(&mut session, true, 18);

        session.apply(&CoachEvent::ClickInterrupted { now: at(30) });

        let block = session.block().expect("the block is still here");
        assert_eq!(block.attempts.len(), 2, "a phone call costs no evidence");
        assert_eq!(block.gate_progress.filled(), 2);
        assert!(session.closed_blocks.is_empty(), "and closes nothing");
    }

    #[test]
    fn an_interruption_closes_an_open_window_without_recording_it() {
        let mut session = listening();
        play_to_the_end(&mut session);
        assert_eq!(session.phase(), Some(&Phase::AwaitingVerdict));

        let writes = session.apply(&CoachEvent::ClickInterrupted { now: at(30) });

        assert!(
            session.block().unwrap().attempts.is_empty(),
            "the pass was interrupted, so a verdict on it would be false evidence"
        );
        assert!(writes.evidence.is_empty());
    }

    #[test]
    fn an_interruption_does_not_restart_the_click() {
        let mut session = listening();
        let pulse = session.block().unwrap().pulse_seq;

        session.apply(&CoachEvent::ClickInterrupted { now: at(30) });

        assert_eq!(
            session.block().unwrap().pulse_seq,
            pulse,
            "nothing to restart yet; Start from the card is what restarts it"
        );
    }

    #[test]
    fn an_interruption_on_a_card_is_nothing_to_report() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));

        session.apply(&CoachEvent::ClickInterrupted { now: at(30) });

        assert_eq!(
            session.phase(),
            Some(&Phase::BlockEntry),
            "nothing was sounding, so a route change between blocks is not an event"
        );
    }

    #[test]
    fn the_minutes_already_practised_survive_an_interruption() {
        let mut session = listening();
        session.apply(&CoachEvent::Tick { now: at(180) });
        assert_eq!(session.block().unwrap().elapsed_seconds(), 180);

        session.apply(&CoachEvent::ClickInterrupted { now: at(180) });
        assert_eq!(
            session.block().unwrap().elapsed_seconds(),
            180,
            "three minutes were practised; parking on the card does not unspend them"
        );

        session.apply(&CoachEvent::Tick { now: at(400) });
        assert_eq!(
            session.block().unwrap().elapsed_seconds(),
            180,
            "and the interruption itself is not practice either"
        );

        session.apply(&CoachEvent::StartBlock { now: at(400) });
        session.apply(&CoachEvent::Tick { now: at(430) });
        assert_eq!(
            session.block().unwrap().elapsed_seconds(),
            210,
            "resuming carries on from what was spent rather than refunding it"
        );
    }

    #[test]
    fn a_block_skipped_after_an_interruption_records_the_time_it_took() {
        let mut session = two_block_session();
        rep(&mut session, true, 9);
        session.apply(&CoachEvent::Tick { now: at(180) });
        session.apply(&CoachEvent::ClickInterrupted { now: at(180) });

        session.apply(&CoachEvent::SkipBlock { now: at(400) });

        let record = session.closed_blocks.last().expect("a record");
        assert_eq!(record.attempts.len(), 1);
        assert_eq!(
            record.active_ms, 180_000,
            "a record carrying attempts must carry the time they took"
        );
    }

    #[test]
    fn an_interruption_after_the_gate_opened_still_banks_the_pass() {
        let mut session = two_block_session();
        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        rep(&mut session, true, 27);
        assert_eq!(session.phase(), Some(&Phase::GateOpen));

        session.apply(&CoachEvent::ClickInterrupted { now: at(30) });

        let record = session.closed_blocks.last().expect("the block closed");
        assert_eq!(
            record.exit,
            Exit::GatePassed,
            "the criterion was met before the phone rang, and that stands"
        );
        assert_eq!(session.block().map(|block| block.spec_index), Some(1));
    }

    #[test]
    fn leaving_from_an_interrupted_card_still_records_what_was_practised() {
        let mut session = listening();
        rep(&mut session, true, 9);
        session.apply(&CoachEvent::Tick { now: at(180) });
        session.apply(&CoachEvent::ClickInterrupted { now: at(180) });

        session.apply(&CoachEvent::LeaveSession { now: at(400) });

        let record = session.closed_blocks.last().expect("a record");
        assert_eq!(record.exit, Exit::SessionEnded);
        assert_eq!(record.active_ms, 180_000);
    }

    #[test]
    fn an_interrupted_block_keeps_its_minutes_across_a_crash() {
        let mut crashed = listening();
        crashed.apply(&CoachEvent::Tick { now: at(180) });
        crashed.apply(&CoachEvent::ClickInterrupted { now: at(180) });
        crashed.apply(&CoachEvent::Tick { now: at(300) });

        let mut restored = EngineSession::default();
        restored.apply(&CoachEvent::RecoverSession {
            session: crashed,
            now: at(9000),
        });

        let block = restored.block().expect("the block came back");
        assert_eq!(block.phase, Phase::BlockEntry);
        assert_eq!(
            block.elapsed_seconds(),
            180,
            "the blob carries what was practised, and the outage is not more of it"
        );
    }

    #[test]
    fn an_interruption_on_the_last_block_closes_the_session_on_its_pass() {
        let mut session = listening();
        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        rep(&mut session, true, 27);
        assert_eq!(session.phase(), Some(&Phase::GateOpen));

        let writes = session.apply(&CoachEvent::ClickInterrupted { now: at(30) });

        assert_eq!(session.state, SessionState::Closing);
        assert_eq!(
            writes.blocks.len(),
            1,
            "the record still leaves for the store"
        );
        assert_eq!(writes.blocks[0].exit, Exit::GatePassed);
        assert_eq!(writes.snapshot, SnapshotAction::Clear);
    }

    // ── The continuous pulse ──

    #[test]
    fn a_tap_does_not_restart_the_click() {
        let mut session = listening();
        let pulse = session.block().unwrap().pulse_seq;
        play_to_the_end(&mut session);

        session.apply(&CoachEvent::Tap {
            clean: true,
            now: at(9),
        });

        let block = session.block().unwrap();
        assert_eq!(
            block.pulse_seq, pulse,
            "the pulse runs unbroken for the whole block"
        );
        assert_eq!(
            session.phase(),
            Some(&Phase::Listening),
            "the hands are already playing the next pass"
        );
        assert_eq!(
            session.block().unwrap().last_verdict,
            Some(Verdict::Clean),
            "the glance still draws until the next beat clears it (T10)"
        );
    }

    #[test]
    fn the_next_beat_ends_the_glance() {
        let mut session = listening();
        play_to_the_end(&mut session);
        session.apply(&CoachEvent::Tap {
            clean: true,
            now: at(9),
        });
        let landed = session.block().unwrap().beat_index;

        session.apply(&CoachEvent::Beat {
            beat_index: landed + 1,
        });

        assert_eq!(
            session.block().unwrap().last_verdict,
            None,
            "the glance belongs to the tap, and the pulse has moved on (#1184)"
        );
    }

    #[test]
    fn a_new_block_starts_a_new_pulse_and_a_new_count_in() {
        let mut session = two_block_session();
        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        rep(&mut session, true, 27);
        session.apply(&CoachEvent::Tick { now: at(30) });

        session.apply(&CoachEvent::StartBlock { now: at(31) });
        assert_eq!(
            session.phase(),
            Some(&Phase::CountIn { beats_remaining: 4 }),
            "new material, so the click starts again from a count-in"
        );
    }

    #[test]
    fn a_straggler_from_the_torn_down_pulse_does_not_silence_the_new_one() {
        let mut session = listening();
        let phrase = session.block().unwrap().body_beats();
        session.apply(&CoachEvent::Beat { beat_index: 20 });
        session.apply(&CoachEvent::Stuck { now: at(5) });

        session.apply(&CoachEvent::Beat { beat_index: 47 });
        assert_eq!(
            session.block().unwrap().beat_index,
            0,
            "a beat from a pulse that no longer exists is not this pulse's position"
        );

        session.apply(&CoachEvent::CountInBeat { remaining: 0 });
        session.apply(&CoachEvent::Beat { beat_index: 0 });
        let shrunk = session.block().unwrap().body_beats();
        session.apply(&CoachEvent::Beat { beat_index: shrunk });

        assert_eq!(
            session.phase(),
            Some(&Phase::AwaitingVerdict),
            "the first pass after an escalation is judged like any other"
        );
        let _ = phrase;
    }

    #[test]
    fn the_ladder_acting_is_what_restarts_the_click() {
        let mut session = listening();
        let pulse = session.block().unwrap().pulse_seq;

        session.apply(&CoachEvent::Stuck { now: at(5) });

        assert_ne!(
            session.block().unwrap().pulse_seq,
            pulse,
            "a new tempo is a new pulse, so the shell has to restart it"
        );
    }

    #[test]
    fn the_verdict_window_opens_on_each_pass_and_the_beats_keep_coming() {
        let mut session = listening();
        let phrase = session.block().unwrap().body_beats();

        session.apply(&CoachEvent::Beat { beat_index: phrase });
        assert_eq!(session.phase(), Some(&Phase::AwaitingVerdict));

        session.apply(&CoachEvent::Beat {
            beat_index: phrase + 1,
        });
        assert_eq!(
            session.phase(),
            Some(&Phase::AwaitingVerdict),
            "the window rests for the whole pass; the hands play on under it"
        );
        assert_eq!(session.block().unwrap().beat_index, phrase + 1);
    }

    #[test]
    fn an_untapped_pass_is_dropped_rather_than_freezing_the_loop() {
        let mut session = listening();
        let phrase = session.block().unwrap().body_beats();
        session.apply(&CoachEvent::Beat { beat_index: phrase });

        session.apply(&CoachEvent::Beat {
            beat_index: phrase * 2,
        });

        assert_eq!(
            session.phase(),
            Some(&Phase::AwaitingVerdict),
            "the window now asks about the pass that just finished"
        );
        assert!(
            session.block().unwrap().attempts.is_empty(),
            "the pass nobody judged is not evidence"
        );
    }

    #[test]
    fn the_beat_position_reads_the_phrase_and_not_the_whole_block() {
        let mut session = listening();
        let phrase = session.block().unwrap().body_beats();

        session.apply(&CoachEvent::Beat {
            beat_index: phrase + 5,
        });

        let block = session.block().unwrap();
        assert_eq!(
            (block.bar(), block.beat()),
            (2, 2),
            "the second pass reads as bar 2 beat 2, not bar 10 of a block"
        );
    }

    // ── Discard ──

    #[test]
    fn discarding_an_open_verdict_records_nothing() {
        let mut session = listening();
        play_to_the_end(&mut session);

        session.apply(&CoachEvent::DiscardAttempt { now: at(9) });

        let block = session.block().unwrap();
        assert!(block.attempts.is_empty(), "a false start is not an attempt");
        assert_eq!(block.gate_progress.filled(), 0);
        assert_eq!(block.consecutive_fails, 0);
        assert_eq!(block.last_verdict, None, "there is no verdict to glance at");
        assert_eq!(session.phase(), Some(&Phase::Listening));
    }

    #[test]
    fn a_discard_never_reaches_the_mastery_track() {
        let mut session = listening();
        play_to_the_end(&mut session);

        let writes = session.apply(&CoachEvent::DiscardAttempt { now: at(9) });

        assert!(
            writes.evidence.is_empty(),
            "nothing that would bias the Beta estimate down"
        );
    }

    #[test]
    fn a_discard_leaves_a_run_of_misses_where_it_was() {
        let mut session = listening();
        rep(&mut session, false, 9);
        rep(&mut session, false, 18);
        assert_eq!(session.block().unwrap().consecutive_fails, 2);

        play_to_the_end(&mut session);
        session.apply(&CoachEvent::DiscardAttempt { now: at(27) });

        assert_eq!(
            session.block().unwrap().consecutive_fails,
            2,
            "discarding is not a pass, and it is not a fail either"
        );
        assert_eq!(session.phase(), Some(&Phase::Listening));
    }

    #[test]
    fn discarding_mid_pass_suppresses_the_window_that_pass_would_have_opened() {
        let mut session = listening();
        let phrase = session.block().unwrap().body_beats();
        session.apply(&CoachEvent::Beat { beat_index: 2 });

        session.apply(&CoachEvent::DiscardAttempt { now: at(4) });
        session.apply(&CoachEvent::Beat { beat_index: phrase });

        assert_eq!(
            session.phase(),
            Some(&Phase::Listening),
            "the fumbled pass goes round again with nothing asked of it"
        );

        session.apply(&CoachEvent::Beat {
            beat_index: phrase * 2,
        });
        assert_eq!(
            session.phase(),
            Some(&Phase::AwaitingVerdict),
            "and the pass after it is judged as usual"
        );
    }

    #[test]
    fn a_discard_with_nothing_to_discard_is_ignored() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));

        session.apply(&CoachEvent::DiscardAttempt { now: at(5) });

        assert_eq!(
            session.phase(),
            Some(&Phase::BlockEntry),
            "there is nothing to not count before the block has started"
        );
    }

    // ── Entering the loop ──

    #[test]
    fn starting_a_session_runs_the_plan_it_was_handed() {
        let mut session = EngineSession::default();
        let content = ContentIndex::shipped();
        session.apply_with_plan(
            &CoachEvent::StartPlannedSession { now: at(0) },
            Some(crate::engine::plan::plan(
                &crate::engine::coach::CoachState::default(),
                crate::engine::plan::PlanContext {
                    now: at(0),
                    available_minutes: content.session_minutes,
                    rng_seed: 0,
                },
            )),
        );

        let SessionState::Running { plan, .. } = &session.state else {
            panic!("a session should be running");
        };
        assert!(
            plan.blocks.len() > 1,
            "a session has a shape, not one seeded block (#1180)"
        );
        assert_eq!(
            session.spec().expect("a first block").drill,
            "shells-cycle",
            "planned from content/gates.toml, not from a Rust constant"
        );
    }

    #[test]
    fn the_thresholds_are_the_files_and_not_rust_constants() {
        let config = EngineConfig::default();
        let authored = &ContentIndex::shipped().escalation;

        assert_eq!(
            config.consecutive_fail_trigger,
            authored.consecutive_fail_trigger
        );
        assert_eq!(config.ladder, authored.ladder);
        assert_eq!(config.tempo_down_pct, authored.tempo_down_pct);
        assert_eq!(config.tempo_floor_bpm, authored.tempo_floor_bpm);
        assert_eq!(config.gate_open_hold_s, authored.gate_open_hold_s);
    }

    #[test]
    fn a_block_opens_on_its_card() {
        let mut session = EngineSession::default();
        assert_eq!(session.phase(), None, "nothing runs before it is started");

        session.start_fixture(at(0));
        assert_eq!(session.phase(), Some(&Phase::BlockEntry));
    }

    #[test]
    fn the_count_in_ticks_down_and_the_first_body_beat_opens_the_window() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));
        session.apply(&CoachEvent::StartBlock { now: at(0) });

        session.apply(&CoachEvent::CountInBeat { remaining: 2 });
        assert_eq!(
            session.phase(),
            Some(&Phase::CountIn { beats_remaining: 2 })
        );

        session.apply(&CoachEvent::Beat { beat_index: 0 });
        assert_eq!(session.phase(), Some(&Phase::Listening));
    }

    #[test]
    fn the_last_body_beat_asks_the_question() {
        let mut session = listening();
        let body = session.block().unwrap().body_beats();

        session.apply(&CoachEvent::Beat {
            beat_index: body - 1,
        });
        assert_eq!(
            session.phase(),
            Some(&Phase::Listening),
            "still playing on the final beat of the phrase"
        );

        session.apply(&CoachEvent::Beat { beat_index: body });
        assert_eq!(session.phase(), Some(&Phase::AwaitingVerdict));
    }

    #[test]
    fn the_count_in_runs_out_to_a_brief_rest_before_the_first_beat() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));
        session.apply(&CoachEvent::StartBlock { now: at(0) });

        session.apply(&CoachEvent::CountInBeat { remaining: 0 });
        assert_eq!(
            session.phase(),
            Some(&Phase::CountIn { beats_remaining: 0 }),
            "the last count-in click leaves a brief zero-remaining rest"
        );
    }

    #[test]
    fn a_tap_during_play_is_not_a_verdict() {
        let mut session = listening();
        session.apply(&CoachEvent::Tap {
            clean: true,
            now: at(4),
        });

        assert_eq!(
            session.phase(),
            Some(&Phase::Listening),
            "Layer 0: no verdict exists while the hands are still moving"
        );
        assert_eq!(session.block().unwrap().attempts.len(), 0);
    }

    // ── The tap, and what it decides (spec §4's three `Verdict` rows) ──

    #[test]
    fn a_tap_fills_a_dot_and_sends_the_hands_straight_back() {
        let mut session = listening();
        play_to_the_end(&mut session);
        session.apply(&CoachEvent::Tap {
            clean: true,
            now: at(9),
        });

        let block = session.block().unwrap();
        assert_eq!(block.gate_progress.filled(), 1);
        assert_eq!(block.last_verdict, Some(Verdict::Clean));
        assert_eq!(session.phase(), Some(&Phase::Listening));
    }

    #[test]
    fn the_gate_opens_when_the_last_dot_fills() {
        let mut session = listening();
        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        rep(&mut session, true, 27);

        assert_eq!(session.phase(), Some(&Phase::GateOpen));
        let block = session.block().unwrap();
        assert!(block.gate_progress.satisfied());
        assert_eq!(block.gate_opened_at_attempt, Some(3));
    }

    #[test]
    fn a_miss_run_at_the_trigger_escalates_without_being_asked() {
        let mut session = listening();
        rep(&mut session, false, 9);
        rep(&mut session, false, 18);
        assert_eq!(
            session.phase(),
            Some(&Phase::Listening),
            "two misses is a bad patch, not a wall — straight back to playing"
        );

        play_to_the_end(&mut session);
        session.apply(&CoachEvent::Tap {
            clean: false,
            now: at(27),
        });
        assert_eq!(
            session.phase(),
            Some(&Phase::Escalating {
                rung: Rung::TempoDown
            })
        );
        let block = session.block().unwrap();
        assert_eq!(block.level.tempo_bpm, 96, "120 less a fifth");
        assert_eq!(block.escalation_fired, vec![Rung::TempoDown]);
        assert_eq!(
            block.consecutive_fails, 0,
            "the ladder acted, so the run starts again from the new tempo"
        );
    }

    #[test]
    fn escalating_puts_the_hands_back_at_bar_one() {
        let mut session = listening();
        session.apply(&CoachEvent::Beat { beat_index: 19 });
        assert_eq!(session.block().unwrap().bar(), 5);

        session.apply(&CoachEvent::Stuck { now: at(5) });
        let block = session.block().unwrap();
        assert_eq!(
            (block.bar(), block.beat()),
            (1, 1),
            "the position must never read past the phrase it just shrank"
        );
        assert_eq!(
            block.last_verdict, None,
            "the ladder acted; the glance is stale"
        );
    }

    #[test]
    fn a_rung_that_would_change_nothing_spends_itself_and_moves_on() {
        let mut session = EngineSession {
            config: EngineConfig {
                tempo_floor_bpm: 120,
                ..EngineConfig::default()
            },
            ..EngineSession::default()
        };
        session.start_fixture(at(0));
        session.apply(&CoachEvent::StartBlock { now: at(0) });
        session.apply(&CoachEvent::Beat { beat_index: 0 });

        session.apply(&CoachEvent::Stuck { now: at(5) });
        let block = session.block().unwrap();
        assert_eq!(
            block.level.tempo_bpm, 120,
            "already at the floor, so the tempo cannot drop"
        );
        assert_eq!(
            session.phase(),
            Some(&Phase::Escalating {
                rung: Rung::ShrinkScope
            }),
            "an identical rep is not an escalation — fall through to the next rung"
        );
        assert_eq!(
            block.escalation_fired,
            vec![Rung::TempoDown, Rung::ShrinkScope],
            "the unusable rung is still spent"
        );
    }

    #[test]
    fn a_plan_handed_over_mid_block_is_refused_by_the_machine_itself() {
        let mut session = listening();
        let running = session.spec().expect("a running block").drill.clone();

        let writes = session.apply_with_plan(
            &CoachEvent::PlanSession {
                now: at(5),
                available_minutes: Some(20),
            },
            Some(Plan::fixture()),
        );

        assert_eq!(
            session.spec().map(|spec| spec.drill.clone()),
            Some(running),
            "the state owner refuses it too, not only the caller that declines to \
             plan: a dropped block leaves no record behind it (#1219)"
        );
        assert!(writes.blocks.is_empty());
        assert_eq!(writes.snapshot, SnapshotAction::Unchanged);
    }

    /// A fixture plan whose one block has somewhere to go when the ladder gets
    /// past dropping the tempo and shrinking the phrase.
    fn with_alternative(rung: Rung) -> EngineSession {
        let mut plan = Plan::fixture();
        let mut spec = plan.blocks[0].spec.clone();
        spec.drill = "rootless-one-key".to_string();
        spec.section = Some("one key, LH alone".to_string());
        let why = plan.blocks[0].why.clone();
        plan.blocks[0]
            .alternatives
            .push(crate::engine::plan::Alternative { rung, spec, why });

        let mut session = EngineSession::default();
        session.start(plan, at(0));
        session.apply(&CoachEvent::StartBlock { now: at(0) });
        session.apply(&CoachEvent::Beat { beat_index: 0 });
        session
    }

    /// Every rung of the ladder, fired by asking rather than by missing.
    fn get_stuck(session: &mut EngineSession, times: usize) {
        for time in 0..times {
            session.apply(&CoachEvent::Stuck {
                now: at(5 + time as i64),
            });
            session.apply(&CoachEvent::Beat { beat_index: 0 });
        }
    }

    #[test]
    fn swapping_the_drill_draws_the_alternative_the_plan_supplied() {
        let mut session = with_alternative(Rung::SwapDrill);

        get_stuck(&mut session, 4);

        assert_eq!(
            session.spec().map(|spec| spec.drill.as_str()),
            Some("rootless-one-key"),
            "the fourth rung swaps the drill for one the plan carried"
        );
        let closed = session
            .closed_blocks
            .last()
            .expect("the block it gave up on");
        assert_eq!(closed.exit, Exit::Escalated);
        assert_eq!(
            closed.escalation_fired,
            vec![
                Rung::TempoDown,
                Rung::ShrinkScope,
                Rung::ChangeMode,
                Rung::SwapDrill
            ],
            "every rung it went through is on the record, including the one that \
             had nothing to draw"
        );
        assert!(
            matches!(session.state, SessionState::Running { .. }),
            "and the session carries on: abandoning is not what the app teaches"
        );
    }

    #[test]
    fn changing_mode_draws_its_own_alternative_before_the_drill_is_swapped() {
        let mut session = with_alternative(Rung::ChangeMode);

        get_stuck(&mut session, 3);

        assert_eq!(
            session.spec().map(|spec| spec.drill.as_str()),
            Some("rootless-one-key"),
            "the third rung acts when the plan can supply a different way in"
        );
        assert_eq!(
            session
                .closed_blocks
                .last()
                .map(|closed| closed.escalation_fired.clone()),
            Some(vec![Rung::TempoDown, Rung::ShrinkScope, Rung::ChangeMode]),
            "and the ladder stops there rather than swapping the drill too"
        );
    }

    #[test]
    fn a_block_that_escalates_past_what_the_plan_can_supply_still_ends_escalated() {
        let mut session = listening();

        get_stuck(&mut session, 4);

        assert_eq!(
            session.state,
            SessionState::Closing,
            "one block, no alternatives, so the session closes rather than pretending"
        );
        let closed = session.closed_blocks.last().expect("the block");
        assert_eq!(closed.exit, Exit::Escalated);
        assert_eq!(
            closed.escalation_fired,
            vec![
                Rung::TempoDown,
                Rung::ShrinkScope,
                Rung::ChangeMode,
                Rung::SwapDrill
            ]
        );
    }

    #[test]
    fn an_alternative_is_drawn_once_and_not_dealt_twice() {
        let mut session = with_alternative(Rung::SwapDrill);

        get_stuck(&mut session, 4);
        session.apply(&CoachEvent::StartBlock { now: at(60) });
        get_stuck(&mut session, 4);

        assert_eq!(
            session.closed_blocks.len(),
            2,
            "the swapped-in block escalates too, and has nothing left to draw"
        );
        assert_eq!(session.state, SessionState::Closing);
    }

    #[test]
    fn a_consecutive_gate_empties_its_dots_and_still_escalates() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));
        if let SessionState::Running { plan, block } = &mut session.state {
            plan.blocks[0].spec.gate.requirement = Requirement::CleanPasses {
                count: 3,
                consecutive: true,
            };
            block.gate_progress = GateProgress::new(&plan.blocks[0].spec.gate.requirement);
        }
        session.apply(&CoachEvent::StartBlock { now: at(0) });
        session.apply(&CoachEvent::Beat { beat_index: 0 });

        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        assert_eq!(session.block().unwrap().gate_progress.filled(), 2);

        rep(&mut session, false, 27);
        assert_eq!(session.block().unwrap().gate_progress.filled(), 0);
        rep(&mut session, false, 36);
        rep(&mut session, false, 45);
        assert_eq!(
            session.phase(),
            Some(&Phase::Escalating {
                rung: Rung::TempoDown
            }),
            "three misses is three misses, whatever the dots did"
        );
    }

    #[test]
    fn a_second_block_opens_where_the_first_left_off() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));
        if let SessionState::Running { plan, .. } = &mut session.state {
            let mut second = plan.blocks[0].clone();
            second.spec.drill_title = "Shells".to_string();
            second.spec.node = "shells-ii-v-i".to_string();
            plan.blocks.push(second);
        }
        session.apply(&CoachEvent::StartBlock { now: at(0) });
        session.apply(&CoachEvent::Beat { beat_index: 0 });

        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        rep(&mut session, true, 27);
        session.apply(&CoachEvent::Tick { now: at(30) });

        assert_eq!(session.closed_blocks.len(), 1);
        assert_eq!(session.closed_blocks[0].node, "rootless-a-b");
        let block = session.block().expect("the next block opened");
        assert_eq!(block.spec_index, 1);
        assert_eq!(
            block.gate_progress.filled(),
            0,
            "a fresh gate, not the old one"
        );
        assert_eq!(block.attempts.len(), 0);
        assert_eq!(session.spec().unwrap().node, "shells-ii-v-i");
    }

    #[test]
    fn events_arriving_out_of_order_are_ignored_rather_than_obeyed() {
        let mut session = listening();
        play_to_the_end(&mut session);
        session.apply(&CoachEvent::Tap {
            clean: true,
            now: at(9),
        });
        let after_first = session.block().unwrap().attempts.len();

        session.apply(&CoachEvent::Tap {
            clean: true,
            now: at(10),
        });
        assert_eq!(
            session.block().unwrap().attempts.len(),
            after_first,
            "a second tap on one rep is not a second attempt"
        );

        let body = session.block().unwrap().body_beats();
        session.apply(&CoachEvent::Beat { beat_index: body });
        assert_eq!(
            session.phase(),
            Some(&Phase::Listening),
            "a beat the click already reported cannot reopen the pass it ended"
        );
    }

    #[test]
    fn the_gate_wins_over_a_ceiling_landing_in_the_same_second() {
        let mut session = listening();
        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        play_to_the_end(&mut session);
        session.apply(&CoachEvent::Tap {
            clean: true,
            now: at(359),
        });

        session.apply(&CoachEvent::Tick { now: at(362) });
        assert_eq!(
            session.closed_blocks[0].exit,
            Exit::GatePassed,
            "the criterion was met; the clock running out doesn't undo it"
        );
    }

    #[test]
    fn being_stuck_fires_the_next_rung_without_waiting_for_three_misses() {
        let mut session = listening();
        session.apply(&CoachEvent::Stuck { now: at(5) });

        assert_eq!(
            session.phase(),
            Some(&Phase::Escalating {
                rung: Rung::TempoDown
            })
        );
        assert_eq!(session.block().unwrap().level.tempo_bpm, 96);
    }

    #[test]
    fn the_second_rung_shrinks_the_scope() {
        let mut session = listening();
        let bars = session.block().unwrap().bars;
        session.apply(&CoachEvent::Stuck { now: at(5) });
        session.apply(&CoachEvent::Beat { beat_index: 0 });
        session.apply(&CoachEvent::Stuck { now: at(30) });

        let block = session.block().unwrap();
        assert_eq!(
            session.phase(),
            Some(&Phase::Escalating {
                rung: Rung::ShrinkScope
            })
        );
        assert_eq!(block.bars, bars / 2);
        assert_eq!(block.level.tempo_bpm, 96, "the tempo drop stands");
    }

    #[test]
    fn a_ladder_the_engine_cannot_act_on_ends_the_block_rather_than_pretending() {
        let mut session = listening();
        for second in [5, 30, 60] {
            session.apply(&CoachEvent::Stuck { now: at(second) });
            session.apply(&CoachEvent::Beat { beat_index: 0 });
        }

        assert_eq!(session.state, SessionState::Closing);
        assert_eq!(session.closed_blocks.len(), 1);
        assert_eq!(session.closed_blocks[0].exit, Exit::Escalated);
        assert_eq!(
            session.closed_blocks[0].escalation_fired,
            vec![
                Rung::TempoDown,
                Rung::ShrinkScope,
                Rung::ChangeMode,
                Rung::SwapDrill
            ],
            "a rung with nothing to draw on is still spent, the same way a rung              that would change nothing is (#1182)"
        );
    }

    // ── Leaving a block ──

    #[test]
    fn the_ceiling_ends_the_block_wherever_it_has_got_to() {
        let mut session = listening();
        session.apply(&CoachEvent::Tick { now: at(359) });
        assert_eq!(session.phase(), Some(&Phase::Listening));

        session.apply(&CoachEvent::Tick { now: at(361) });
        assert_eq!(session.closed_blocks[0].exit, Exit::CeilingHit);
    }

    #[test]
    fn the_gate_open_moment_holds_then_moves_on() {
        let mut session = listening();
        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        rep(&mut session, true, 27);
        assert_eq!(session.phase(), Some(&Phase::GateOpen));

        session.apply(&CoachEvent::Tick { now: at(28) });
        assert_eq!(session.phase(), Some(&Phase::GateOpen), "still reading it");

        session.apply(&CoachEvent::Tick { now: at(30) });
        let record = &session.closed_blocks[0];
        assert_eq!(record.exit, Exit::GatePassed);
        assert_eq!(record.attempts_to_pass, Some(3));
    }

    #[test]
    fn ending_early_still_closes_the_block() {
        let mut session = listening();
        rep(&mut session, true, 9);
        session.apply(&CoachEvent::LeaveSession { now: at(20) });

        assert_eq!(session.closed_blocks[0].exit, Exit::SessionEnded);
        assert_eq!(session.closed_blocks[0].attempts_to_pass, None);
        assert_eq!(session.state, SessionState::Closing);
    }

    // ── What gets written down (spec §4: recorded from the first build) ──

    #[test]
    fn every_attempt_records_where_its_verdict_came_from() {
        let mut session = listening();
        rep(&mut session, true, 9);
        rep(&mut session, false, 18);

        let attempts = &session.block().unwrap().attempts;
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0].verdict, Verdict::Clean);
        assert_eq!(attempts[1].verdict, Verdict::Missed);
        assert!(attempts
            .iter()
            .all(|a| a.source == EvidenceSource::TapVerdict));
        assert!(
            attempts.iter().all(|a| !a.cold),
            "the session does not decide cold: whether this is returning material \
             is the mastery store's read (#1188), and nothing set it here"
        );
        assert!(
            attempts.iter().all(|a| a.self_predicted.is_none()),
            "predict-then-reveal is deferred with machine listening (§3)"
        );
    }

    #[test]
    fn the_block_record_keeps_what_an_aggregate_row_could_never_rebuild() {
        let mut session = listening();
        rep(&mut session, false, 9);
        rep(&mut session, true, 18);
        rep(&mut session, true, 27);
        rep(&mut session, true, 36);
        session.apply(&CoachEvent::Tick { now: at(40) });

        let record = &session.closed_blocks[0];
        assert_eq!(record.node, "rootless-a-b");
        assert_eq!(record.attempts.len(), 4);
        assert_eq!(record.attempts_to_pass, Some(4));
        assert_eq!(record.gate_opened_at_attempt, Some(4));
        assert_eq!(record.reps_after_gate, 0);
        assert!(record.active_ms > 0);
        assert!(!record.id.is_empty(), "client-minted ulid (invariant 3)");
    }

    #[test]
    fn the_record_carries_the_fluency_frame_tags_of_what_was_practised() {
        let mut session = listening();
        session.apply(&CoachEvent::LeaveSession { now: at(20) });

        let record = &session.closed_blocks[0];
        assert_eq!(
            (record.circle, record.mode),
            (Circle::Hands, Mode::Keys),
            "rootless voicings are hands-circle work at the keys (content/nodes.md)"
        );
    }

    // ── What leaves for the store (spec §4's persistence paragraph, #1181) ──

    #[test]
    fn a_closed_block_is_handed_over_for_persistence() {
        let mut session = listening();
        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        rep(&mut session, true, 27);

        let writes = session.apply(&CoachEvent::Tick { now: at(30) });
        assert_eq!(writes.blocks.len(), 1, "the record leaves as it closes");
        assert_eq!(writes.blocks[0].exit, Exit::GatePassed);
        assert_eq!(
            writes.snapshot,
            SnapshotAction::Clear,
            "the session is over, so the crash-recovery blob would recover nothing"
        );
    }

    #[test]
    fn a_tap_is_worth_a_snapshot_and_a_beat_is_not() {
        let mut session = listening();
        play_to_the_end(&mut session);

        let writes = session.apply(&CoachEvent::Tap {
            clean: true,
            now: at(9),
        });
        assert_eq!(writes.snapshot, SnapshotAction::Save);
        assert!(writes.blocks.is_empty(), "the block is still running");

        assert_eq!(
            session.apply(&CoachEvent::Beat { beat_index: 0 }).snapshot,
            SnapshotAction::Unchanged,
            "the click reports several beats a second; recovery restarts the rep anyway"
        );
        assert_eq!(
            session.apply(&CoachEvent::Tick { now: at(10) }).snapshot,
            SnapshotAction::Unchanged
        );
    }

    #[test]
    fn the_ladder_acting_is_worth_a_snapshot() {
        let mut session = listening();
        assert_eq!(
            session.apply(&CoachEvent::Stuck { now: at(5) }).snapshot,
            SnapshotAction::Save,
            "a recovered session must come back at the tempo the ladder dropped it to"
        );
    }

    #[test]
    fn recovery_keeps_the_evidence_and_the_time_already_spent() {
        let mut crashed = listening();
        rep(&mut crashed, true, 9);
        rep(&mut crashed, false, 18);
        let spent = crashed.block().unwrap().elapsed_seconds();

        let mut restored = EngineSession::default();
        restored.apply(&CoachEvent::RecoverSession {
            session: crashed,
            now: at(3600),
        });

        let block = restored.block().expect("the block came back");
        assert_eq!(block.attempts.len(), 2, "the taps already banked survive");
        assert_eq!(
            block.elapsed_seconds(),
            spent,
            "the outage is not practice time, and the minutes already spent are not refunded"
        );
        assert_eq!(
            restored.phase(),
            Some(&Phase::CountIn { beats_remaining: 4 }),
            "the hands come back to a count-in, not mid-phrase"
        );
    }

    #[test]
    fn a_recovered_block_tells_the_shell_to_start_the_click_again() {
        let mut crashed = listening();
        crashed.apply(&CoachEvent::DiscardAttempt { now: at(5) });
        let pulse = crashed.block().unwrap().pulse_seq;

        let mut restored = EngineSession::default();
        restored.apply(&CoachEvent::RecoverSession {
            session: crashed,
            now: at(3600),
        });

        let block = restored.block().expect("the block came back");
        assert_ne!(
            block.pulse_seq, pulse,
            "nothing is sounding after a crash, so the shell has to be told to \
             start the click rather than left thinking it already runs"
        );
        assert_eq!(
            block.beat_index, 0,
            "and the new pulse counts from its own first beat"
        );
    }

    #[test]
    fn a_discard_does_not_survive_the_crash_that_interrupted_it() {
        let mut crashed = listening();
        crashed.apply(&CoachEvent::DiscardAttempt { now: at(5) });

        let mut restored = EngineSession::default();
        restored.apply(&CoachEvent::RecoverSession {
            session: crashed,
            now: at(3600),
        });
        restored.apply(&CoachEvent::CountInBeat { remaining: 0 });
        restored.apply(&CoachEvent::Beat { beat_index: 0 });
        let phrase = restored.block().unwrap().body_beats();
        restored.apply(&CoachEvent::Beat { beat_index: phrase });

        assert_eq!(
            restored.phase(),
            Some(&Phase::AwaitingVerdict),
            "the discarded pass died with the crash; the first pass back is judged"
        );
    }

    #[test]
    fn a_gate_that_opened_before_the_crash_still_closes_its_block() {
        let mut crashed = listening();
        rep(&mut crashed, true, 9);
        rep(&mut crashed, true, 18);
        rep(&mut crashed, true, 27);
        assert_eq!(crashed.phase(), Some(&Phase::GateOpen));

        let mut restored = EngineSession::default();
        restored.apply(&CoachEvent::RecoverSession {
            session: crashed,
            now: at(3600),
        });
        assert_eq!(restored.phase(), Some(&Phase::GateOpen));

        restored.apply(&CoachEvent::Tick { now: at(3603) });
        assert_eq!(
            restored.closed_blocks[0].exit,
            Exit::GatePassed,
            "a pass earned before the crash is still a pass"
        );
    }

    #[test]
    fn a_recovered_wander_never_counts_the_outage_as_practice() {
        let mut crashed = EngineSession::default();
        crashed.start_fixture(at(0));
        crashed.apply(&CoachEvent::GoOffPiste { now: at(30) });

        let mut restored = EngineSession::default();
        restored.apply(&CoachEvent::RecoverSession {
            session: crashed,
            now: at(4000),
        });
        restored.apply(&CoachEvent::CloseSession { now: at(4100) });

        let wander = &restored.wanders[0];
        assert_eq!(
            (wander.ended_at - wander.started_at).num_seconds(),
            100,
            "the app cannot know what happened while it was dead, so it banks none of it"
        );
    }

    #[test]
    fn recovering_a_blob_older_than_the_session_does_not_slice_past_its_records() {
        // A snapshot write the shell swallowed leaves an older blob on disk, so
        // recovery can hand back a session with fewer records than the live one.
        let mut live = EngineSession::default();
        live.start_fixture(at(0));
        live.apply(&CoachEvent::LeaveSession { now: at(20) });
        assert_eq!(live.closed_blocks.len(), 1);

        let writes = live.apply(&CoachEvent::RecoverSession {
            session: EngineSession::default(),
            now: at(3600),
        });
        assert!(writes.blocks.is_empty(), "an empty blob closed nothing");
    }

    #[test]
    fn a_recovered_session_offers_its_records_again() {
        let mut crashed = EngineSession::default();
        crashed.start_fixture(at(0));
        crashed.apply(&CoachEvent::LeaveSession { now: at(20) });

        let mut restored = EngineSession::default();
        let writes = restored.apply(&CoachEvent::RecoverSession {
            session: crashed,
            now: at(3600),
        });
        assert_eq!(
            writes.blocks.len(),
            1,
            "the snapshot survived the crash, so its records may never have \
             landed; the store upserts, so offering them again is free"
        );
    }

    #[test]
    fn leaving_a_planned_session_closes_it_rather_than_leaving_it_open() {
        let mut session = EngineSession {
            state: SessionState::Planned {
                plan: Plan::fixture(),
            },
            ..EngineSession::default()
        };

        let writes = session.apply(&CoachEvent::LeaveSession { now: at(20) });
        assert_eq!(session.state, SessionState::Closing);
        assert_eq!(
            writes.snapshot,
            SnapshotAction::Clear,
            "a session nobody is in must not leave a blob to recover"
        );
    }

    #[test]
    fn a_closed_wander_is_handed_over_for_persistence() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));
        session.apply(&CoachEvent::GoOffPiste { now: at(0) });

        let writes = session.apply(&CoachEvent::CloseSession { now: at(600) });
        assert_eq!(writes.wanders.len(), 1);
        assert_eq!(writes.snapshot, SnapshotAction::Clear);
    }

    #[test]
    fn answering_the_keep_prompt_rewrites_the_record() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));
        session.apply(&CoachEvent::GoOffPiste { now: at(0) });
        session.apply(&CoachEvent::CloseSession { now: at(600) });

        let writes = session.apply(&CoachEvent::KeepWanderAsDrill { keep: true });
        assert_eq!(
            writes.wanders.len(),
            1,
            "the answer changes a row already written, so it goes back to the store"
        );
        assert_eq!(writes.wanders[0].keep_as_drill, Some(true));
    }

    // ── The peer states (spec §4: peers of Running, not sub-states) ──

    #[test]
    fn off_piste_banks_its_time_and_asks_about_keeping_it() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));
        session.apply(&CoachEvent::GoOffPiste { now: at(30) });
        assert!(matches!(session.state, SessionState::OffPiste { .. }));

        session.apply(&CoachEvent::CloseSession { now: at(400) });
        assert_eq!(session.state, SessionState::Closing);
        let wander = &session.wanders[0];
        assert_eq!(wander.keep_as_drill, None, "None = not yet asked");
        assert!(wander.attempts.is_empty(), "no verdicts without a gate");

        session.apply(&CoachEvent::KeepWanderAsDrill { keep: true });
        assert_eq!(session.wanders[0].keep_as_drill, Some(true));
    }

    #[test]
    fn going_off_piste_closes_the_block_it_left() {
        let mut session = listening();
        rep(&mut session, true, 9);
        session.apply(&CoachEvent::GoOffPiste { now: at(30) });

        assert_eq!(session.closed_blocks.len(), 1);
        assert_eq!(session.closed_blocks[0].exit, Exit::Skipped);
    }

    #[test]
    fn unmonitored_is_never_reachable_from_mid_block() {
        let mut session = listening();
        session.apply(&CoachEvent::GoUnmonitored { now: at(30) });

        assert!(
            matches!(session.state, SessionState::Running { .. }),
            "switching part-way would make the captured half retrospectively unconsented"
        );
        assert!(session.closed_blocks.is_empty());
    }

    #[test]
    fn unmonitored_from_the_start_captures_nothing_but_a_duration() {
        let mut session = EngineSession::default();
        session.apply(&CoachEvent::GoUnmonitored { now: at(0) });
        assert!(matches!(session.state, SessionState::Unmonitored { .. }));

        session.apply(&CoachEvent::CloseSession { now: at(600) });
        assert_eq!(session.state, SessionState::Closing);
        assert_eq!(session.unmonitored_seconds, 600);
        assert!(session.closed_blocks.is_empty());
        assert!(session.wanders.is_empty(), "decision 16: nothing inferred");
    }
}
