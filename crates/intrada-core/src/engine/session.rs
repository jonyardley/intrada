//! The coach session state machine (engine spec §4), scoped by decision 18 to
//! tap-verdicts: the transitions are the spec's, what fires them is a tap
//! rather than a segmented attempt.
//!
//! Two departures from the table, both forced by there being no machine
//! verdict yet:
//!
//! - `Listening` splits. The spec's one `Listening | attempt segmented |
//!   Verdict` row becomes two steps — the grid finishing the phrase body
//!   (`AwaitingVerdict`), then the user's tap. When the scoring path returns,
//!   `AwaitingVerdict` collapses back out.
//! - `Verdict` is the tap, not a state. With nothing to wait for, the three
//!   `Verdict | … |` rows are three branches of [`EngineSession::resolve_tap`],
//!   and the one-second glance is what `CountIn` draws while `last_verdict` is
//!   set. A phase no event can rest in would be a phase the shell can't render.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::gate::{EvidenceSource, GateProgress, Verdict};
use super::plan::{BlockSpec, ParameterLevel, Plan};

/// What the shell tells the engine. The whole write half of the bridge surface
/// for the tap-verdict loop (spec §6, as scoped) — the shell reports clicks,
/// taps and seconds, and decides none of them.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum CoachEvent {
    /// Run the seeded drill loop. Becomes "run this plan" when §5's planner lands.
    StartDrillLoop {
        now: DateTime<Utc>,
    },
    /// One count-in click; `remaining` counts down to 1.
    CountInBeat {
        remaining: u8,
    },
    /// One click on or after bar 1 beat 1, 0-based. `body_beats` is the
    /// landing beat that ends the phrase.
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
    EndBlock {
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
}

/// Where a block is within one repetition.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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
pub enum Rung {
    TempoDown,
    ShrinkScope,
    ChangeMode,
    SwapDrill,
}

/// How a block ended.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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
pub struct AttemptSummary {
    pub at: DateTime<Utc>,
    pub verdict: Verdict,
    pub source: EvidenceSource,
    /// First rep of the block: the highest-information tap-verdict.
    pub cold: bool,
    pub self_predicted: Option<Verdict>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockRecord {
    pub id: String,
    pub node: String,
    pub drill: String,
    pub gate: String,
    pub level: ParameterLevel,
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
pub struct WanderRecord {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub attempts: Vec<AttemptSummary>,
    /// `None` = not yet asked.
    pub keep_as_drill: Option<bool>,
}

/// Thresholds the engine must not hard-code. `content/gates.toml` becomes their
/// source when its parser lands (§8); these are the values in the file today.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EngineConfig {
    pub consecutive_fail_trigger: u8,
    pub ladder: Vec<Rung>,
    pub gate_open_hold_s: i64,
    pub tempo_down_pct: u16,
    pub tempo_floor_bpm: u16,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            consecutive_fail_trigger: 3,
            ladder: vec![
                Rung::TempoDown,
                Rung::ShrinkScope,
                Rung::ChangeMode,
                Rung::SwapDrill,
            ],
            gate_open_hold_s: 2,
            tempo_down_pct: 20,
            tempo_floor_bpm: 40,
        }
    }
}

impl Rung {
    /// Changing mode or swapping the drill needs alternatives only §5's planner
    /// can supply, so a block that reaches those rungs ends rather than
    /// pretending to escalate.
    fn acts(&self) -> bool {
        matches!(self, Rung::TempoDown | Rung::ShrinkScope)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    /// What the last tap said, so `CountIn` can draw the glance.
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

    /// Beats of phrase. `body_beats` itself is the landing beat that ends it —
    /// the click the player aims at, so the shell schedules one more than this.
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
pub enum SessionState {
    #[default]
    Idle,
    Planned {
        plan: Plan,
    },
    Running {
        plan: Plan,
        cursor: usize,
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

    pub fn apply(&mut self, event: &CoachEvent) {
        match event {
            CoachEvent::StartDrillLoop { now } => self.start(Plan::seed_drill_loop(), *now),
            CoachEvent::CountInBeat { remaining } => self.count_in(*remaining),
            CoachEvent::Beat { beat_index } => self.beat(*beat_index),
            CoachEvent::Tap { clean, now } => self.tap(*clean, *now),
            CoachEvent::Stuck { now } => self.stuck(*now),
            CoachEvent::Tick { now } => self.tick(*now),
            CoachEvent::EndBlock { now } => self.end_block(*now),
            CoachEvent::GoOffPiste { now } => self.go_off_piste(*now),
            CoachEvent::GoUnmonitored { now } => self.go_unmonitored(*now),
            CoachEvent::KeepWanderAsDrill { keep } => {
                if let Some(wander) = self.wanders.last_mut() {
                    wander.keep_as_drill = Some(*keep);
                }
            }
            CoachEvent::CloseSession { now } => self.close_session(*now),
        }
    }

    fn start(&mut self, plan: Plan, now: DateTime<Utc>) {
        let Some(spec) = plan.blocks.first() else {
            self.state = SessionState::Planned { plan };
            return;
        };
        let block = BlockState::open(spec, 0, now);
        self.state = SessionState::Running {
            plan,
            cursor: 0,
            block,
        };
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
            block.consecutive_fails + 1
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

        if block.consecutive_fails >= trigger {
            if !self.escalate() {
                self.close_block(Exit::Escalated, true);
            }
            return;
        }

        let count_in_beats = block.count_in_beats;
        block.rep_seq += 1;
        block.phase = Phase::CountIn {
            beats_remaining: count_in_beats,
        };
    }

    /// `false` = the ladder has no rung left the engine can act on.
    fn escalate(&mut self) -> bool {
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
                self.config.tempo_down_pct,
                self.config.tempo_floor_bpm,
                rung,
            )
        };
        if !rung.acts() {
            return false;
        }

        let Some(block) = self.block_mut() else {
            return false;
        };
        match rung {
            Rung::TempoDown => {
                let dropped = block.level.tempo_bpm.saturating_sub(
                    (u32::from(block.level.tempo_bpm) * u32::from(pct) / 100) as u16,
                );
                block.level.tempo_bpm = dropped.max(floor);
            }
            Rung::ShrinkScope => block.bars = (block.bars / 2).max(1),
            Rung::ChangeMode | Rung::SwapDrill => return false,
        }
        block.escalation_fired.push(rung);
        block.consecutive_fails = 0;
        block.last_verdict = None;
        block.rep_seq += 1;
        block.phase = Phase::Escalating { rung };
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
        let ceiling = self.spec().and_then(|spec| spec.gate.time_ceiling_s);
        let hold = self.config.gate_open_hold_s;
        let Some(block) = self.block_mut() else {
            return;
        };
        block.now = now;

        if ceiling.is_some_and(|ceiling| block.elapsed_seconds() >= ceiling) {
            self.close_block(Exit::CeilingHit, true);
            return;
        }
        if block.phase == Phase::GateOpen
            && block
                .gate_open_since
                .is_some_and(|since| (now - since).num_seconds() >= hold)
        {
            self.close_block(Exit::GatePassed, true);
        }
    }

    fn end_block(&mut self, now: DateTime<Utc>) {
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
            SessionState::Running { .. } => self.end_block(now),
            _ => self.state = SessionState::Closing,
        }
    }

    /// Bank the block, then either open the next one or close. Abandoning must
    /// never be what the app teaches (spec §4).
    fn close_block(&mut self, exit: Exit, advance: bool) {
        let SessionState::Running {
            plan,
            cursor,
            block,
        } = &self.state
        else {
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

        let next = cursor + 1;
        let now = block.now;
        match plan.blocks.get(next).filter(|_| advance) {
            Some(spec) => {
                let block = BlockState::open(spec, next, now);
                let SessionState::Running { plan, .. } = std::mem::take(&mut self.state) else {
                    return;
                };
                self.state = SessionState::Running {
                    plan,
                    cursor: next,
                    block,
                };
            }
            None => self.state = SessionState::Closing,
        }
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
    use chrono::TimeZone;

    fn at(second: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_754_300_000 + second, 0).unwrap()
    }

    /// Started, count-in done, one body beat played — the state every rep runs in.
    fn listening() -> EngineSession {
        let mut session = EngineSession::default();
        session.apply(&CoachEvent::StartDrillLoop { now: at(0) });
        session.apply(&CoachEvent::Beat { beat_index: 0 });
        session
    }

    fn play_to_the_end(session: &mut EngineSession) {
        let last = session.block().expect("a running block").body_beats();
        session.apply(&CoachEvent::Beat { beat_index: last });
    }

    fn rep(session: &mut EngineSession, clean: bool, second: i64) {
        play_to_the_end(session);
        session.apply(&CoachEvent::Tap {
            clean,
            now: at(second),
        });
        if session.phase() == Some(&Phase::CountIn { beats_remaining: 4 }) {
            session.apply(&CoachEvent::Beat { beat_index: 0 });
        }
    }

    // ── Entering the loop ──

    #[test]
    fn a_block_opens_on_the_count_in() {
        let mut session = EngineSession::default();
        assert_eq!(session.phase(), None, "nothing runs before it is started");

        session.apply(&CoachEvent::StartDrillLoop { now: at(0) });
        assert_eq!(
            session.phase(),
            Some(&Phase::CountIn { beats_remaining: 4 })
        );
        assert_eq!(session.block().unwrap().rep_seq, 1);
    }

    #[test]
    fn the_count_in_ticks_down_and_the_first_body_beat_opens_the_window() {
        let mut session = EngineSession::default();
        session.apply(&CoachEvent::StartDrillLoop { now: at(0) });

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

        rep(&mut session, false, 27);
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
        session.apply(&CoachEvent::EndBlock { now: at(20) });

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

    // ── The peer states (spec §4: peers of Running, not sub-states) ──

    #[test]
    fn off_piste_banks_its_time_and_asks_about_keeping_it() {
        let mut session = EngineSession::default();
        session.apply(&CoachEvent::StartDrillLoop { now: at(0) });
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
