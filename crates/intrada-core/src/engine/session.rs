//! The coach session state machine. `specs/intrada-coach-engine.md` §4 carries
//! the transition table and records where this scoping of it departs from it.

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use super::content::ContentIndex;
use super::gate::{EvidenceSource, GateProgress, Verdict};
use super::plan::{BlockSpec, Circle, Mode, ParameterLevel, Plan};

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
    /// Plan today's session from the authored content and run it. Becomes
    /// "run this plan" when the session-start screen can say how long the user
    /// has (#1182); until then the length comes from `[defaults]`.
    StartDrillLoop {
        now: DateTime<Utc>,
    },
    /// One count-in click, reported as it sounds; `remaining` is beats left
    /// *after* this one, counting down to 0 on the last click (#1184).
    CountInBeat {
        remaining: u8,
    },
    /// One click on or after bar 1 beat 1, 0-based. The phrase ends on the
    /// beat at index `DrillView::click_beats - 1`.
    Beat {
        beat_index: u32,
    },
    /// The user's verdict on the rep just played (decision 18).
    Tap {
        clean: bool,
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
    pub snapshot: SnapshotAction,
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
    /// Bumped whenever the shell should restart the click.
    pub rep_seq: u32,
    pub gate_opened_at_attempt: Option<u16>,
    pub reps_after_gate: u16,
    gate_open_since: Option<DateTime<Utc>>,
}

impl BlockState {
    fn open(spec: &BlockSpec, spec_index: usize, now: DateTime<Utc>) -> Self {
        Self {
            id: ulid::Ulid::generate().to_string(),
            spec_index,
            phase: Phase::CountIn {
                beats_remaining: spec.count_in_beats,
            },
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
            rep_seq: 1,
            gate_opened_at_attempt: None,
            reps_after_gate: 0,
            gate_open_since: None,
        }
    }

    /// A new repetition: back to bar 1, and a `rep_seq` the shell restarts the
    /// click on.
    fn start_rep(&mut self) {
        self.beat_index = 0;
        self.last_verdict = None;
        self.rep_seq = self.rep_seq.saturating_add(1);
    }

    /// Beats of phrase. The click the player aims at lands one beyond them, so
    /// [`Self::click_beats`] is what the shell schedules.
    pub fn body_beats(&self) -> u32 {
        u32::from(self.bars.max(1)) * u32::from(self.beats_per_bar.max(1))
    }

    pub fn click_beats(&self) -> u32 {
        self.body_beats() + 1
    }

    pub fn elapsed_seconds(&self) -> u32 {
        (self.now - self.started_at).num_seconds().max(0) as u32
    }

    /// The musician's 1-based counting.
    pub fn bar(&self) -> u16 {
        (self.beat_index / u32::from(self.beats_per_bar.max(1))) as u16 + 1
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
            SessionState::Running { plan, block, .. } => plan.blocks.get(block.spec_index),
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

    pub fn apply(&mut self, event: &CoachEvent) -> CoachWrites {
        let before = self.recovery_key();
        let was_closing = matches!(self.state, SessionState::Closing);
        let blocks_written = self.closed_blocks.len();
        let wanders_written = self.wanders.len();

        self.handle(event);

        let mut writes = CoachWrites {
            blocks: self.closed_blocks[blocks_written..].to_vec(),
            wanders: self.wanders[wanders_written..].to_vec(),
            snapshot: if matches!(self.state, SessionState::Closing) {
                if was_closing {
                    SnapshotAction::Unchanged
                } else {
                    SnapshotAction::Clear
                }
            } else if self.recovery_key() == before {
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

    fn handle(&mut self, event: &CoachEvent) {
        match event {
            CoachEvent::StartDrillLoop { now } => {
                let content = ContentIndex::shipped();
                self.start(Plan::for_today(content, content.session_minutes, 0), *now)
            }
            CoachEvent::CountInBeat { remaining } => self.count_in(*remaining),
            CoachEvent::Beat { beat_index } => self.beat(*beat_index),
            CoachEvent::Tap { clean, now } => self.tap(*clean, *now),
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
                block.rep_seq = block.rep_seq.saturating_add(1);
                if block.gate_progress.satisfied() && block.gate_opened_at_attempt.is_some() {
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

    fn start(&mut self, plan: Plan, now: DateTime<Utc>) {
        let Some(spec) = plan.blocks.first() else {
            self.state = SessionState::Planned { plan };
            return;
        };
        let block = BlockState::open(spec, 0, now);
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
            // The first count-in click ends the tap's glance (#1184).
            block.last_verdict = None;
            block.phase = Phase::CountIn {
                beats_remaining: remaining,
            };
        }
    }

    fn beat(&mut self, beat_index: u32) {
        let Some(block) = self.block_mut() else {
            return;
        };
        if !matches!(
            block.phase,
            Phase::CountIn { .. } | Phase::Listening | Phase::Escalating { .. }
        ) {
            return;
        }
        block.beat_index = beat_index;
        block.phase = if beat_index >= block.body_beats() {
            Phase::AwaitingVerdict
        } else {
            block.last_verdict = None;
            Phase::Listening
        };
    }

    fn tap(&mut self, clean: bool, now: DateTime<Utc>) {
        let trigger = self.config.consecutive_fail_trigger;
        let Some(block) = self.block_mut() else {
            return;
        };
        if block.phase != Phase::AwaitingVerdict {
            return;
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
            cold: block.attempts.is_empty(),
            self_predicted: None,
        });
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

        let verdict = block.last_verdict;
        block.start_rep();
        block.last_verdict = verdict;
        block.phase = Phase::CountIn {
            beats_remaining: block.count_in_beats,
        };
    }

    /// `false` = the ladder has no rung left the engine can act on, so the
    /// caller ends the block instead of pretending to escalate.
    fn escalate(&mut self) -> bool {
        loop {
            let (pct, floor, rung) = {
                let Some(block) = self.block() else {
                    return false;
                };
                let Some(rung) = self
                    .config
                    .ladder
                    .get(block.escalation_fired.len())
                    .copied()
                else {
                    return false;
                };
                (
                    self.config.tempo_down_pct.min(100),
                    self.config.tempo_floor_bpm,
                    rung,
                )
            };
            let Some(block) = self.block_mut() else {
                return false;
            };

            let acted = match rung {
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
                // Changing mode or swapping the drill needs alternatives only
                // §5's planner can supply (#1182).
                Rung::ChangeMode | Rung::SwapDrill => return false,
            };
            block.escalation_fired.push(rung);
            if !acted {
                // Already at the floor or down to one bar: record the rung as
                // spent and try the next, rather than restart an identical rep.
                continue;
            }
            block.consecutive_fails = 0;
            block.start_rep();
            block.phase = Phase::Escalating { rung };
            return true;
        }
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
        let over = ceiling.is_some_and(|ceiling| block.elapsed_seconds() >= ceiling);

        if held {
            self.close_block(Exit::GatePassed, true);
        } else if over {
            self.close_block(Exit::CeilingHit, true);
        }
    }

    fn leave_session(&mut self, now: DateTime<Utc>) {
        if let Some(block) = self.block_mut() {
            block.now = now;
        }
        self.close_block(Exit::SessionEnded, false);
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
        let Some(spec) = plan.blocks.get(block.spec_index) else {
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
            active_ms: (block.now - block.started_at).num_milliseconds().max(0) as u64,
            escalation_fired: block.escalation_fired.clone(),
            exit,
        });

        let next = block.spec_index + 1;
        let now = block.now;
        match plan.blocks.get(next).filter(|_| advance) {
            Some(spec) => *block = BlockState::open(spec, next, now),
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
        session.apply(&CoachEvent::Beat { beat_index: 0 });
        session
    }

    fn play_to_the_end(session: &mut EngineSession) {
        let last = session.block().expect("a running block").body_beats();
        session.apply(&CoachEvent::Beat { beat_index: last });
    }

    /// One whole repetition: play it out, answer, and let the count-in run into
    /// the next. Unconditional, so a broken transition fails where it broke.
    fn rep(session: &mut EngineSession, clean: bool, second: i64) {
        play_to_the_end(session);
        session.apply(&CoachEvent::Tap {
            clean,
            now: at(second),
        });
        session.apply(&CoachEvent::Beat { beat_index: 0 });
    }

    // ── Entering the loop ──

    #[test]
    fn starting_a_session_runs_todays_plan_from_the_content() {
        let mut session = EngineSession::default();
        session.apply(&CoachEvent::StartDrillLoop { now: at(0) });

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
    fn a_block_opens_on_the_count_in() {
        let mut session = EngineSession::default();
        assert_eq!(session.phase(), None, "nothing runs before it is started");

        session.start_fixture(at(0));
        assert_eq!(
            session.phase(),
            Some(&Phase::CountIn { beats_remaining: 4 })
        );
        assert_eq!(session.block().unwrap().rep_seq, 1);
    }

    #[test]
    fn the_count_in_ticks_down_and_the_first_body_beat_opens_the_window() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));

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
    fn the_first_count_in_beat_ends_the_glance() {
        let mut session = listening();
        play_to_the_end(&mut session);
        session.apply(&CoachEvent::Tap {
            clean: true,
            now: at(9),
        });
        assert_eq!(session.block().unwrap().last_verdict, Some(Verdict::Clean));

        session.apply(&CoachEvent::CountInBeat { remaining: 3 });
        assert_eq!(
            session.block().unwrap().last_verdict,
            None,
            "the glance belongs to the tap; the count-in belongs to the next rep (#1184)"
        );

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
        assert_eq!(block.rep_seq, 2, "the shell restarts the click on this");
        assert_eq!(
            session.phase(),
            Some(&Phase::CountIn { beats_remaining: 4 })
        );
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
    fn a_consecutive_gate_empties_its_dots_and_still_escalates() {
        let mut session = EngineSession::default();
        session.start_fixture(at(0));
        if let SessionState::Running { plan, block } = &mut session.state {
            plan.blocks[0].gate.requirement = Requirement::CleanPasses {
                count: 3,
                consecutive: true,
            };
            block.gate_progress = GateProgress::new(&plan.blocks[0].gate.requirement);
        }
        session.apply(&CoachEvent::Beat { beat_index: 0 });

        rep(&mut session, true, 9);
        rep(&mut session, true, 18);
        assert_eq!(session.block().unwrap().gate_progress.filled(), 2);

        rep(&mut session, false, 27);
        assert_eq!(session.block().unwrap().gate_progress.filled(), 0);
        rep(&mut session, false, 36);
        session.apply(&CoachEvent::Beat {
            beat_index: session.block().unwrap().body_beats(),
        });
        session.apply(&CoachEvent::Tap {
            clean: false,
            now: at(45),
        });
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
            second.drill_title = "Shells".to_string();
            second.node = "shells-ii-v-i".to_string();
            plan.blocks.push(second);
        }
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
        session.apply(&CoachEvent::Beat { beat_index: body });
        assert_eq!(
            session.phase(),
            Some(&Phase::AwaitingVerdict),
            "a late beat cannot reopen a rep that already ended"
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
            vec![Rung::TempoDown, Rung::ShrinkScope],
            "only the rungs that actually changed the plan are recorded"
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
            attempts[0].cold,
            "the first rep of the block on returning material is the cold test"
        );
        assert!(!attempts[1].cold);
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
