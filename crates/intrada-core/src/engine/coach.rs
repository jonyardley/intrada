//! The engine's read half of the bridge (spec §6, as scoped by decision 18):
//! one `ViewModel` field, built from the session machine. The capture types
//! (`NoteBatch` and friends) belong to the deferred scoring path and are not
//! here yet.
//!
//! Everything the drill screen draws comes from this module, so counting,
//! gating and what-comes-next stay in Rust.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::content::ContentIndex;
use super::gate::{Requirement, Verdict};
use super::mastery::MasteryStore;
use super::plan::{plan, BlockOrigin, ParameterLevel, Plan, PlanContext, PlannedBlock, Stage};
use super::session::{
    section_evidence, Altitude, BlockRecord, CoachEvent, CoachWrites, EngineSession, Exit, Phase,
    SessionState, SnapshotAction, SoftLanding,
};
use crate::domain::built_session::PlayThroughRecord;
use crate::domain::item::ItemKind;

/// Spec §1 gives this five fields; the judgement track and the interruption
/// ledger arrive with Phase 2b.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CoachState {
    pub session: EngineSession,
    pub mastery: MasteryStore,
    /// Last reported by the shell on `PlanSession` (#1221). Off
    /// `EngineSession` so it costs no blob: the next plan re-supplies it.
    pub utc_offset_minutes: i32,
    /// A crash-recovery blob the shell found at launch, waiting on the user's
    /// answer (#1193, #1305). Off `EngineSession` for the same reason as the
    /// offset: it is the shell's blob, not the running session's, and holding
    /// it here keeps it off the wire that writes the blob.
    pending_recovery: Option<EngineSession>,
}

impl Default for CoachState {
    fn default() -> Self {
        Self {
            session: EngineSession::default(),
            mastery: MasteryStore::seeded_from(ContentIndex::shipped()),
            utc_offset_minutes: 0,
            pending_recovery: None,
        }
    }
}

/// What [`CoachState::place_steer`] did with an accepted steer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteerPlacement {
    Placed,
    AlreadyPlanned,
    /// No plan to place into yet. Not a refusal: an accepted steer is
    /// re-derived into every later plan (`app::refresh_steer`).
    Deferred,
    /// A session is running at another altitude: no plan to join, and none
    /// coming until it closes.
    SessionInFlight,
}

impl CoachState {
    pub fn apply(&mut self, event: &CoachEvent) -> CoachWrites {
        match event {
            CoachEvent::OfferRecovery { session } => return self.offer_recovery(session),
            CoachEvent::DeclineRecovery => return self.decline_recovery(),
            CoachEvent::AcceptRecovery {
                now,
                utc_offset_minutes,
            } => {
                // No `accepts_something_new` guard: `settle_offer` is where
                // that invariant lives, so nothing is holdable once one runs.
                let Some(session) = self.pending_recovery.take() else {
                    return CoachWrites::default();
                };
                return self.apply(&CoachEvent::RecoverSession {
                    session,
                    now: *now,
                    utc_offset_minutes: *utc_offset_minutes,
                });
            }
            _ => {}
        }
        // Every door that can open a session reports it, so skipping the
        // preview cannot silently put the cold test back on UTC (#1221).
        if let Some(offset) = reported_offset(event) {
            self.utc_offset_minutes = offset;
        }
        self.mark_cold(event);
        let planned = self.plan_for(event);
        let writes = self.session.apply_with_plan(event, planned);
        for attempt in &writes.evidence {
            self.mastery
                .record(&attempt.node, attempt.level, attempt.verdict, attempt.at);
        }
        for record in banked_passes(&writes.blocks) {
            if let Some(next) = next_rung(record) {
                self.mastery.level_up(&record.node, record.level, next);
            }
        }
        self.settle_offer(&writes);
        writes
    }

    /// An offer is live only while nothing is running *and* its blob is still on
    /// disk. One enforcement point rather than one per door, because the doors
    /// are not all `CoachEvent`s: `adopt_plan` is the built session's (#1256),
    /// and it is the one that made this bite — with the offer still up, the
    /// drill loop ran the crashed session over the composition Start was
    /// pressed on.
    fn settle_offer(&mut self, writes: &CoachWrites) {
        let running = !self.session.state.accepts_something_new();
        // A `Save` rewrote the blob and a clear removed it; either way the bytes
        // the offer was made from have gone, so it can no longer be taken.
        let blob_moved = writes.snapshot != SnapshotAction::Unchanged;
        if running || blob_moved {
            self.pending_recovery = None;
        }
    }

    /// Two refusals, and they must not share an action: a blob holding a state
    /// no crash could strand recovers nothing and goes, but a session in flight
    /// owns the bytes on disk, so clearing them would strip its own protection.
    fn offer_recovery(&mut self, session: &EngineSession) -> CoachWrites {
        if !self.session.state.accepts_something_new() {
            return CoachWrites::default();
        }
        if !session.state.worth_recovering() {
            return CoachWrites {
                snapshot: SnapshotAction::ClearOffer,
                ..CoachWrites::default()
            };
        }
        self.pending_recovery = Some(session.clone());
        CoachWrites::default()
    }

    /// Declining is the whole point of the prompt (#1193), so it has to reach
    /// the blob: dropping it here alone would offer the same session again on
    /// the next launch.
    fn decline_recovery(&mut self) -> CoachWrites {
        self.pending_recovery = None;
        CoachWrites {
            snapshot: SnapshotAction::ClearOffer,
            ..CoachWrites::default()
        }
    }

    /// Run a plan the planner did not make: the built session (#1256). The
    /// composition is the user's, so `CoachState` cannot work it out — but the
    /// session machine must still be the one that runs it, or a steer would be
    /// a second drill loop.
    pub fn adopt_plan(&mut self, plan: Plan, now: DateTime<Utc>) -> CoachWrites {
        if !self.session.state.accepts_something_new() {
            // A session already running is not something a steer may replace:
            // the blocks in flight have evidence riding on them.
            return CoachWrites::default();
        }
        // Clearing to `Idle` is what makes the machine take the plan it is
        // handed rather than the prescribed one it made for itself.
        self.session.state = SessionState::Idle;
        let writes = self.session.apply_with_plan(
            &CoachEvent::StartPlannedSession {
                now,
                utc_offset_minutes: self.utc_offset_minutes,
            },
            Some(plan),
        );
        self.settle_offer(&writes);
        writes
    }

    /// Place C3's accepted steer in today's plan (#1256 Phase D): one block
    /// added to the planner's plan rather than replacing it, because decision 12
    /// is propose, confirm, never plan. The extra minutes run over the session
    /// length rather than costing one of the planner's blocks.
    ///
    /// Second in a plan not yet started, so a steer is the first real thing the
    /// session does rather than what it runs out of time for. In a running one
    /// it goes after the block in flight, never at or below
    /// `BlockState::spec_index`, which would silently re-aim it.
    ///
    /// The duplicate refusal is live, not theoretical: the accept places it on
    /// screen, and starting that plan carries it into the running one.
    ///
    /// Told apart rather than reported as one failure, because the caller
    /// spends the user's offer on the answer (#1317).
    pub fn place_steer(&mut self, block: PlannedBlock) -> SteerPlacement {
        let (plan, floor) = match &mut self.session.state {
            SessionState::Planned { plan } => (plan, 1),
            SessionState::Running {
                plan,
                block: running,
            } => (plan, running.spec_index + 1),
            state if !state.accepts_something_new() => return SteerPlacement::SessionInFlight,
            _ => return SteerPlacement::Deferred,
        };
        // Section too: `node_id` is target-only, so a whole-piece block and
        // "the bridge of it" share a node without being the same offer.
        if plan.blocks.iter().any(|placed| {
            placed.spec.node == block.spec.node && placed.spec.section == block.spec.section
        }) {
            return SteerPlacement::AlreadyPlanned;
        }
        let at = floor.min(plan.blocks.len());
        plan.blocks.insert(at, block);
        SteerPlacement::Placed
    }

    /// Rebuild the mastery track from the persisted evidence at launch (#1214):
    /// the same one-way replay as the live path in [`Self::apply`]. Replaces
    /// rather than adds, like every other load handler, so a repeated load can
    /// never double-count.
    pub fn rebuild_mastery(&mut self, records: Vec<BlockRecord>, runs: Vec<PlayThroughRecord>) {
        self.mastery = MasteryStore::seeded_from(ContentIndex::shipped());
        let mut replay = Vec::new();
        // A run-through's section verdicts are evidence the live close already
        // recorded, so the rebuild has to know about them too — the same rule
        // in both paths, through the same function (key decision 7).
        let run_evidence: Vec<_> = runs.iter().flat_map(section_evidence).collect();
        for record in &records {
            // Decision 17, on the replay path too: a judgement-track block's
            // taps were never evidence, so they must not become evidence at
            // launch (the #1214 class of bug, one rule along).
            if !record.origin.feeds_mastery() {
                continue;
            }
            for attempt in &record.attempts {
                replay.push((
                    attempt.at,
                    record.node.clone(),
                    Replay::Attempt(attempt.verdict, attempt.level),
                ));
            }
            // `level_up` declines a rung that already has evidence, so a banked
            // pass has to reach the store in the order it happened.
            if let Some(next) = banked_passes(std::slice::from_ref(record))
                .next()
                .and_then(next_rung)
            {
                replay.push((
                    record.ended_at,
                    record.node.clone(),
                    Replay::LevelUp {
                        from: record.level,
                        to: next,
                    },
                ));
            }
        }
        for attempt in run_evidence {
            replay.push((
                attempt.at,
                attempt.node,
                Replay::Attempt(attempt.verdict, attempt.level),
            ));
        }
        replay.sort_by_key(|(at, _, _)| *at);
        for (at, node, step) in replay {
            match step {
                Replay::Attempt(verdict, level) => self.mastery.record(&node, level, verdict, at),
                // The gate was passed at the level the block closed on.
                Replay::LevelUp { from, to } => self.mastery.level_up(&node, from, to),
            }
        }
    }

    /// Only the two press-start events need a plan, and only `CoachState` can
    /// make one: the planner reads the mastery track, which the session does not
    /// hold. The clock seeds the dealer and is stored on the `Plan`, so the
    /// session still replays.
    fn plan_for(&self, event: &CoachEvent) -> Option<Plan> {
        if !self.session.accepts_plan(event) {
            return None;
        }
        let (now, minutes) = match event {
            CoachEvent::PlanSession {
                now,
                available_minutes,
                ..
            } => (*now, *available_minutes),
            CoachEvent::StartPlannedSession { now, .. } => (*now, None),
            _ => return None,
        };
        let content = ContentIndex::shipped();
        Some(plan(
            self,
            PlanContext {
                now,
                available_minutes: minutes.unwrap_or(content.session_minutes),
                rng_seed: now.timestamp().unsigned_abs(),
            },
        ))
    }

    fn mark_cold(&mut self, event: &CoachEvent) {
        let Some(now) = EngineSession::now_of(event) else {
            return;
        };
        let Some((node, level)) = self
            .session
            .spec()
            .map(|spec| spec.node.clone())
            .zip(self.session.block().map(|block| block.level))
        else {
            return;
        };
        let cold = self
            .mastery
            .is_cold(&node, level, now, self.utc_offset_minutes);
        self.session.mark_cold(cold);
    }

    pub fn view(&self) -> CoachView {
        CoachView {
            drill: self.drill_view(),
            plan: self.plan_view(),
            altitude: self.session.state.altitude(),
            run_through: self.run_through_view(),
            open_play: self.open_play_view(),
            recovery: self.recovery_view(),
            landing: self.landing_view(),
        }
    }

    /// The launch prompt for a blob that survived a crash (#1193, #1305). The
    /// wording is the core's, per altitude: the shell renders two strings and
    /// works out nothing about what it was the user was doing.
    fn recovery_view(&self) -> Option<RecoveryView> {
        let session = self.pending_recovery.as_ref()?;
        let (headline, detail, started_at) = match &session.state {
            SessionState::Running { plan, block } => (
                "Pick up where you left off?",
                format!(
                    "{} · block {} of {}",
                    plan.blocks
                        .get(block.spec_index)
                        .map(|planned| planned.spec.drill_title.as_str())
                        .unwrap_or("Your session"),
                    block.spec_index + 1,
                    plan.blocks.len()
                ),
                block.started_at,
            ),
            SessionState::RunThrough(run) => (
                "Carry on the run?",
                format!(
                    "{} · {} of {} sections judged",
                    run.title,
                    run.verdicts.len(),
                    run.sections.len()
                ),
                run.started_at,
            ),
            SessionState::OffPiste { started_at, .. } => (
                "Back to exploring?",
                "Time logged, nothing scored.".to_string(),
                *started_at,
            ),
            SessionState::Unmonitored { started_at, .. } => (
                "Back to playing?",
                "Minutes only, nothing recorded.".to_string(),
                *started_at,
            ),
            // `offer_recovery` refuses these, so reaching one would mean the
            // guard moved rather than that the prompt needs a fifth wording.
            SessionState::Idle | SessionState::Planned { .. } | SessionState::Closing { .. } => {
                return None
            }
        };
        Some(RecoveryView {
            altitude: session.state.altitude(),
            headline: headline.to_string(),
            detail,
            started_at,
        })
    }

    /// The soft landing (#1323). Every sentence is the core's, because a shell
    /// composing its own would be a shell deciding how the user should feel
    /// about their own practice (design doc §"Streak mechanics, defanged").
    fn landing_view(&self) -> Option<LandingView> {
        let SessionState::Closing {
            landing: Some(landing),
        } = &self.session.state
        else {
            return None;
        };
        // Read the same whether or not the plan was walked through: "0 blocks"
        // is a scoreboard for a session that did not happen.
        if landing.blocks_played == 0 {
            return Some(LandingView {
                headline: "Another time.".to_string(),
                detail: None,
                note: Some(
                    "Nothing played, so nothing's on the record. Today's plan is still there."
                        .to_string(),
                ),
            });
        }
        let (headline, note) = if landing.plan_finished {
            ("That's the session.", None)
        } else {
            (
                "That's banked.",
                Some("Short is still practice, and it's on the record.".to_string()),
            )
        };
        Some(LandingView {
            headline: headline.to_string(),
            detail: Some(landing_detail(landing)),
            note,
        })
    }

    fn open_play_view(&self) -> Option<OpenPlayView> {
        match &self.session.state {
            SessionState::OffPiste {
                item_id,
                started_at,
                ..
            } => Some(OpenPlayView {
                altitude: Altitude::OffPiste,
                item_id: item_id.clone(),
                title: None,
                started_at: *started_at,
            }),
            SessionState::Unmonitored { started_at, .. } => Some(OpenPlayView {
                altitude: Altitude::Unmonitored,
                item_id: None,
                title: None,
                started_at: *started_at,
            }),
            _ => None,
        }
    }

    fn run_through_view(&self) -> Option<RunThroughView> {
        let run = self.session.run_through()?;
        Some(RunThroughView {
            title: run.title.clone(),
            sections: run.sections.clone(),
            // Held / broke down, in the order the sections were judged. The
            // shell draws the dots from this and counts nothing itself.
            held: run.verdicts.iter().map(|verdict| verdict.held).collect(),
            current_section: run.current_section().cloned(),
            complete: run.complete(),
            elapsed_seconds: run.elapsed_seconds(),
        })
    }

    /// The press-start surface: what today's session is, before it runs. Gone
    /// the moment the first block opens, because the drill view takes over.
    fn plan_view(&self) -> Option<PlanView> {
        let SessionState::Planned { plan } = &self.session.state else {
            return None;
        };
        Some(PlanView {
            total_minutes: plan.blocks.iter().map(|block| block.spec.minutes).sum(),
            blocks: plan
                .blocks
                .iter()
                .map(|block| PlannedBlockView {
                    drill_title: block.spec.drill_title.clone(),
                    section: block.spec.section.clone(),
                    kind: block.spec.kind.clone(),
                    minutes: block.spec.minutes,
                    why: block.why_line(),
                    added_by_you: block.why.placed_by == Stage::Steer,
                })
                .collect(),
            deferred: plan.deferred.clone(),
        })
    }

    fn drill_view(&self) -> Option<DrillView> {
        let SessionState::Running { plan, .. } = &self.session.state else {
            return None;
        };
        let block = self.session.block()?;
        let spec = self.session.spec()?;
        let tempo_bpm = (!block.level.is_untimed()).then_some(block.level.tempo_bpm);

        Some(DrillView {
            phase: match block.phase {
                Phase::BlockEntry => DrillPhase::BlockEntry,
                Phase::AwaitingVerdict => DrillPhase::AwaitingVerdict,
                Phase::GateOpen => DrillPhase::GateOpen,
                Phase::CountIn { beats_remaining } => DrillPhase::CountIn {
                    remaining: beats_remaining,
                },
                // The glance draws over a pulse that never stopped, so the
                // beat after the tap turns the page, not a count-in click (T11).
                Phase::Listening => match block.last_verdict {
                    Some(verdict) => DrillPhase::Acknowledged {
                        clean: verdict == Verdict::Clean,
                    },
                    None => DrillPhase::Playing,
                },
                Phase::Escalating { .. } => DrillPhase::Playing,
            },
            drill_title: spec.drill_title.clone(),
            section: spec.section.clone(),
            destination: spec.destination.clone(),
            kind: spec.kind.clone(),
            tempo_bpm,
            click_level: block.level.click_level.spoken().to_string(),
            beat: block.beat(),
            beats_per_bar: block.beats_per_bar,
            bar: block.bar(),
            bars: block.bars,
            count_in_beats: block.count_in_beats,
            phrase_beats: block.body_beats(),
            pulse_seq: block.pulse_seq,
            // Never at l0: the metronome there is absent, not silenced, so the
            // shell has no pulse to schedule and no key to hold.
            pulse_running: block.phase != Phase::BlockEntry && !block.level.is_untimed(),
            click_pattern: block.level.click_level.pattern(block.beats_per_bar),
            elapsed_seconds: block.elapsed_seconds(),
            minutes: spec.minutes,
            why: plan
                .blocks
                .get(block.spec_index)
                .map(|planned| planned.why_line())
                .unwrap_or_default(),
            ceiling_seconds: Some(u32::from(spec.minutes) * 60),
            block_kinds: plan
                .blocks
                .iter()
                .map(|block| block.spec.kind.clone())
                .collect(),
            block_index: block.spec_index,
            // A judgement-track block counts nothing, so it asks nothing a
            // count could answer: "you decide when it's done" (A5).
            gate_question: match spec.origin {
                BlockOrigin::Judgement => "Done for now?".to_string(),
                _ => gate_question(&spec.gate.requirement, tempo_bpm),
            },
            gate_summary: match spec.origin {
                BlockOrigin::Judgement => "your call".to_string(),
                _ => gate_summary(&spec.gate.requirement, tempo_bpm),
            },
            gate_filled: block.gate_progress.filled(),
            gate_target: block.gate_progress.target(),
            origin: spec.origin,
            serves: spec.serves.clone(),
        })
    }
}

enum Replay {
    Attempt(Verdict, ParameterLevel),
    LevelUp {
        from: ParameterLevel,
        to: ParameterLevel,
    },
}

/// The gate passes allowed to move a cursor. One definition, called by both the
/// live path and the launch replay, so decision 17 cannot hold in one and not
/// the other — and so it rests on `BlockOrigin` rather than on a judgement
/// node's id happening to be unknown to the content.
fn reported_offset(event: &CoachEvent) -> Option<i32> {
    match event {
        CoachEvent::PlanSession {
            utc_offset_minutes, ..
        }
        | CoachEvent::StartPlannedSession {
            utc_offset_minutes, ..
        }
        | CoachEvent::RecoverSession {
            utc_offset_minutes, ..
        } => Some(*utc_offset_minutes),
        _ => None,
    }
}

/// "2 blocks, 12 minutes. 1 gate passed.": stated, nothing inferred from it.
fn landing_detail(landing: &SoftLanding) -> String {
    let mut detail = format!(
        "{}, {}",
        plural(landing.blocks_played, "block"),
        // A block can close inside a minute, and "0 minutes" reads as none.
        if landing.minutes == 0 {
            "under a minute".to_string()
        } else {
            plural(landing.minutes, "minute")
        }
    );
    if landing.gates_passed > 0 {
        detail.push_str(&format!(
            ". {} passed",
            plural(landing.gates_passed, "gate")
        ));
    }
    detail.push('.');
    detail
}

fn plural(count: u16, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

fn banked_passes(records: &[BlockRecord]) -> impl Iterator<Item = &BlockRecord> {
    records
        .iter()
        .filter(|record| record.exit == Exit::GatePassed && record.origin.feeds_mastery())
}

/// A gate pass moves the cursor to the next rung of the node's ladder the loop
/// can run. The top rung has nowhere to go.
fn next_rung(record: &BlockRecord) -> Option<ParameterLevel> {
    let content = ContentIndex::shipped();
    let node = content.node(&record.node)?;
    let passed = node.drills.iter().position(|id| *id == record.drill)?;
    node.drills[passed + 1..]
        .iter()
        .filter_map(|id| content.drill(id))
        .find_map(|drill| drill.level)
}

/// " at 120", where there is a tempo to be at: a gate must not ask for one
/// the rung cannot name (decision 20).
fn at_tempo(tempo_bpm: Option<u16>) -> String {
    tempo_bpm
        .map(|bpm| format!(" at {bpm}"))
        .unwrap_or_default()
}

fn gate_question(requirement: &Requirement, tempo_bpm: Option<u16>) -> String {
    let at = at_tempo(tempo_bpm);
    match requirement {
        Requirement::CleanPasses { .. } | Requirement::KeyCoverage { .. } => {
            format!("Clean{at}?")
        }
        Requirement::Chained { .. } => format!("Clean{at}, no stops?"),
        Requirement::SelfConfirmed { .. } => "Did that match?".to_string(),
    }
}

fn gate_summary(requirement: &Requirement, tempo_bpm: Option<u16>) -> String {
    let at = at_tempo(tempo_bpm);
    match requirement {
        Requirement::CleanPasses { count, .. } => format!("{count} clean{at}"),
        Requirement::KeyCoverage {
            keys_required,
            per_key_passes,
            first_attempt,
        } => {
            if *first_attempt {
                format!("clean first time, in {keys_required} keys")
            } else {
                format!("{per_key_passes} clean{at}, in {keys_required} keys")
            }
        }
        Requirement::Chained { min_keys } => format!("{min_keys} keys chained, no stops"),
        Requirement::SelfConfirmed { max_listens, .. } => match max_listens {
            Some(listens) => format!("matched within {listens} listens"),
            None => "your call".to_string(),
        },
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CoachView {
    /// `None` unless a coach session is inside a block.
    pub drill: Option<DrillView>,
    /// `Some` while a session is planned but not yet running.
    pub plan: Option<PlanView>,
    /// What the AltitudeChip shows, for the whole run. `None` for a prescribed
    /// session, which is not one of the three altitudes (decision 16).
    pub altitude: Option<Altitude>,
    /// `Some` only while a gated run-through is in flight.
    pub run_through: Option<RunThroughView>,
    /// `Some` while one of the two lower altitudes is running (B2, B3).
    pub open_play: Option<OpenPlayView>,
    /// `Some` while a crash-recovery blob is waiting on Resume or Discard.
    pub recovery: Option<RecoveryView>,
    /// `Some` while a prescribed session's soft landing is owed (#1323).
    pub landing: Option<LandingView>,
}

/// The launch prompt for a session a crash cut off (#1193, #1305). Nothing is
/// running while this is up: it is an offer, and the alternative to accepting
/// it is discarding it, which is the affordance the legacy session already had.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct RecoveryView {
    /// `None` for a prescribed session, as everywhere else.
    pub altitude: Option<Altitude>,
    pub headline: String,
    pub detail: String,
    /// When the interrupted session started, on its own clock — the rebase to
    /// "now" happens on accept, so this is still the instant the user left.
    pub started_at: DateTime<Utc>,
}

/// The last frame of a prescribed session (#1323). Sentences and no numbers, so
/// the shell cannot round, pluralise or reframe what the session was worth.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LandingView {
    pub headline: String,
    /// `None` where nothing was played.
    pub detail: Option<String>,
    /// `None` where the plan was finished, and consolation would be for nothing.
    pub note: Option<String>,
}

/// What off-piste and unmonitored draw: a clock and, off-piste only, the piece.
/// The elapsed number is rendered from `started_at` rather than sent as a count,
/// because these two states have no ceiling and no tick — the core re-anchors
/// the instant on recovery, so an outage still never becomes playing time.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct OpenPlayView {
    pub altitude: Altitude,
    /// Always `None` at unmonitored, and `None` for an off-piste reached
    /// mid-session. Decision 7's asymmetry, read straight off the state: the
    /// screen can name the piece exactly where a record could carry it.
    pub item_id: Option<String>,
    /// Filled from the library where there is an `item_id` — the join is the
    /// core's, so the shell never resolves a domain id itself.
    pub title: Option<String>,
    pub started_at: DateTime<Utc>,
}

/// What the run-through screen draws: which section the next tap judges, and
/// what the ones behind it said.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct RunThroughView {
    pub title: String,
    pub sections: Vec<String>,
    /// One entry per verdict given so far, in order.
    pub held: Vec<bool>,
    /// `None` once every section has a verdict.
    pub current_section: Option<String>,
    /// Every section judged. The exit is still an explicit tap: "Don't count
    /// this run" has to stay reachable after the last verdict (B1).
    pub complete: bool,
    pub elapsed_seconds: u32,
}

/// What press-start shows: today's session before it starts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlanView {
    pub blocks: Vec<PlannedBlockView>,
    pub total_minutes: u16,
    /// What today could not take, in the plan's own words. Rendered as it
    /// stands; silent dropping is a defect (spec §5 stage 5).
    pub deferred: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlannedBlockView {
    pub drill_title: String,
    pub section: Option<String>,
    pub kind: ItemKind,
    pub minutes: u16,
    /// One sentence, written by the core. The shell renders it and never
    /// composes one.
    pub why: String,
    /// C3's "you added this": provenance stays legible in the shape, so an
    /// accepted steer never reads as something the app decided (decision 12).
    pub added_by_you: bool,
}

/// What the drill screen shows. Deliberately presentational: `Escalating`
/// reads as `Playing`, and the tap's glance holds only until the first
/// count-in click turns the page (#1184, T10).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum DrillPhase {
    /// The card a block opens on. Silent: nothing is scheduled until Start.
    BlockEntry,
    Playing,
    /// The during-play page, count-in dots in the stuck target's place.
    /// `remaining` is beats left after the sounding click, down to 0; the
    /// full count before the first click, so no dot filled.
    CountIn {
        remaining: u8,
    },
    AwaitingVerdict,
    Acknowledged {
        clean: bool,
    },
    GateOpen,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct DrillView {
    pub phase: DrillPhase,
    pub drill_title: String,
    pub section: Option<String>,
    pub destination: Option<String>,
    pub kind: ItemKind,
    /// `None` at l0, where the rung has no tempo to state (decision 20). It is
    /// the same fact as having no beat to count, so a view with no tempo draws
    /// no beat position either.
    pub tempo_bpm: Option<u16>,
    /// The running click level in the musician's words — "beats 2 & 4".
    pub click_level: String,
    pub beat: u8,
    pub beats_per_bar: u8,
    pub bar: u16,
    pub bars: u16,
    pub count_in_beats: u8,
    /// Beats in one pass of the phrase. The pulse itself is unbounded: the
    /// shell keeps a rolling schedule going rather than one rep's worth.
    pub phrase_beats: u32,
    /// The restart key, block-scoped, so the shell keys on
    /// `(block_index, pulse_seq)`. Contract: [`BlockState::pulse_seq`].
    pub pulse_seq: u32,
    /// `false` while a block-entry card is up: stop the click and forget the
    /// key.
    pub pulse_running: bool,
    /// One cycle of the click placement, from the pulse's first body beat:
    /// `click_pattern[beat_index % click_pattern.len()]` sounds, and at least
    /// one beat of it always does. The count-in clicks every beat regardless.
    pub click_pattern: Vec<bool>,
    pub elapsed_seconds: u32,
    pub minutes: u16,
    pub why: String,
    pub ceiling_seconds: Option<u32>,
    pub block_kinds: Vec<ItemKind>,
    pub block_index: usize,
    pub gate_question: String,
    pub gate_summary: String,
    pub gate_filled: u8,
    pub gate_target: u8,
    /// Whose block this is (#1256): what the boundary card's kind chip says,
    /// and whether the taps count for anything beyond the time.
    pub origin: BlockOrigin,
    /// Where a user drill's evidence shows in the ability picture, in the
    /// player's words (A7). `None` where the drill claims nothing.
    pub serves: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::assert_round_trips;
    use crate::engine::gate::ClickLevel;
    use crate::engine::plan::ParameterLevel;
    use crate::engine::session::SnapshotAction;
    use chrono::{DateTime, TimeDelta, TimeZone, Utc};

    fn at(second: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_754_300_000 + second, 0).unwrap()
    }

    fn playing() -> CoachState {
        let mut coach = CoachState::default();
        coach.session.start_fixture(at(0));
        coach.apply(&CoachEvent::StartBlock { now: at(0) });
        coach.apply(&CoachEvent::Beat { beat_index: 0 });
        coach.apply(&CoachEvent::Beat { beat_index: 5 });
        coach
    }

    #[test]
    fn there_is_no_drill_before_one_starts() {
        assert_eq!(CoachState::default().view().drill, None);
    }

    #[test]
    fn the_view_carries_the_facts_the_screen_draws() {
        let drill = playing().view().drill.expect("a running drill");

        assert_eq!(drill.phase, DrillPhase::Playing);
        assert_eq!(drill.drill_title, "Rootless voicings");
        assert_eq!(drill.section.as_deref(), Some("A section"));
        assert_eq!(drill.destination.as_deref(), Some("Strasbourg / St. Denis"));
        assert_eq!(drill.tempo_bpm, Some(120));
        assert_eq!(drill.click_level, "beats 2 & 4");
        assert_eq!(
            (drill.bar, drill.beat),
            (2, 2),
            "beat 5 of 4/4 is bar 2 beat 2"
        );
        assert_eq!(drill.phrase_beats, 32, "8 bars of 4");
        assert_eq!(
            drill.click_pattern,
            vec![false, true, false, true],
            "beats 2 & 4, as the level says and the audio must now do (#1224)"
        );
        assert!(drill.pulse_running);
        assert_eq!(drill.gate_question, "Clean at 120?");
        assert_eq!(drill.gate_summary, "3 clean at 120");
        assert_eq!((drill.gate_filled, drill.gate_target), (0, 3));
        assert_eq!(drill.ceiling_seconds, Some(360));
        assert_eq!(drill.minutes, 6);
        assert!(
            !drill.why.is_empty(),
            "the card's why line is the core's sentence, not the shell's"
        );
    }

    // ── l0: the clickless acquisition block (decision 20) ──

    fn playing_untimed() -> CoachState {
        let mut coach = CoachState::default();
        coach.session.start_untimed_fixture(at(0));
        coach.apply(&CoachEvent::StartBlock { now: at(0) });
        coach
    }

    #[test]
    fn the_drill_view_claims_no_tempo_at_l0() {
        let drill = playing_untimed().view().drill.expect("a running drill");

        assert_eq!(
            drill.tempo_bpm, None,
            "the shell cannot draw a tempo the rung does not have, and a beat \
             position is the same fact"
        );
        assert_eq!(drill.click_level, "no click");
        assert!(
            !drill.pulse_running,
            "the metronome at l0 is absent, not silenced"
        );
        assert_eq!(drill.count_in_beats, 0);
        assert_eq!(
            drill.elapsed_seconds, 0,
            "the ceiling and the elapsed clock stay (decision 15)"
        );
        assert_eq!(drill.ceiling_seconds, Some(360));
    }

    #[test]
    fn the_glance_at_l0_is_ended_by_the_clock_rather_than_by_a_beat() {
        let mut coach = playing_untimed();
        coach.apply(&CoachEvent::Tap {
            clean: true,
            now: at(20),
        });
        assert_eq!(
            coach.view().drill.unwrap().phase,
            DrillPhase::Acknowledged { clean: true },
            "the tap is still acknowledged"
        );

        coach.apply(&CoachEvent::Tick { now: at(22) });

        assert_eq!(
            coach.view().drill.unwrap().phase,
            DrillPhase::Playing,
            "no beat turns the page at l0, so a glance left to a beat would \
             hold the screen on a phase with nothing to tap"
        );
    }

    #[test]
    fn the_gate_question_at_l0_makes_no_claim_about_tempo() {
        let drill = playing_untimed().view().drill.expect("a running drill");

        assert_eq!(drill.gate_question, "Clean?");
        assert_eq!(drill.gate_summary, "3 clean");
    }

    #[test]
    fn l0_evidence_never_lands_on_the_clocked_rung() {
        let mut coach = playing_untimed();
        let clocked = ParameterLevel {
            tempo_bpm: 120,
            click_level: ClickLevel::TwoAndFour,
        };
        // Real evidence at the tempo, so "unchanged" is a claim about this
        // tap rather than about a rung nothing has ever touched.
        for second in [1, 2, 3] {
            coach
                .mastery
                .record("rootless-a-b", clocked, Verdict::Clean, at(second));
        }
        // Read at the same instant as the assertion below: decay is a read,
        // so two clocks would differ by elapsed time, not by the tap.
        let before = coach
            .mastery
            .reading("rootless-a-b", clocked, at(20))
            .evidence;
        assert!(before > 0.0);

        coach.apply(&CoachEvent::Tap {
            clean: true,
            now: at(20),
        });

        assert_eq!(
            coach
                .mastery
                .reading("rootless-a-b", clocked, at(20))
                .evidence,
            before,
            "l0 is a level, so knowing it out of time cannot vouch for the tempo"
        );
        assert!(
            coach
                .mastery
                .reading(
                    "rootless-a-b",
                    ParameterLevel {
                        tempo_bpm: 0,
                        click_level: ClickLevel::NoClick,
                    },
                    at(20),
                )
                .evidence
                > 0.0,
            "and it lands on the rung that was actually played"
        );
    }

    #[test]
    fn a_block_entry_card_is_silent_and_carries_what_it_draws() {
        let mut coach = CoachState::default();
        coach.session.start_fixture(at(0));

        let drill = coach.view().drill.expect("the card is the drill surface");
        assert_eq!(drill.phase, DrillPhase::BlockEntry);
        assert!(
            !drill.pulse_running,
            "nothing is scheduled until the user taps Start"
        );
        assert_eq!(drill.drill_title, "Rootless voicings");
        assert_eq!(drill.section.as_deref(), Some("A section"));
        assert_eq!(drill.minutes, 6);
        assert!(!drill.why.is_empty());
    }

    #[test]
    fn an_interrupted_block_comes_back_as_a_card_that_owes_its_minutes() {
        let mut coach = playing();
        tap(&mut coach, true, 9);
        coach.apply(&CoachEvent::Tick { now: at(180) });

        coach.apply(&CoachEvent::ClickInterrupted { now: at(180) });

        let drill = coach.view().drill.expect("the block is still here");
        assert_eq!(drill.phase, DrillPhase::BlockEntry);
        assert!(
            !drill.pulse_running,
            "nothing is sounding to report beats from"
        );
        assert_eq!(
            drill.elapsed_seconds, 180,
            "the card owes the minutes already practised, unlike a fresh one"
        );
        assert_eq!(drill.gate_filled, 1, "and the pass banked before it");
    }

    #[test]
    fn a_gate_interrupted_open_still_moves_the_mastery_track_up() {
        let mut coach = playing();
        let seed = *coach
            .mastery
            .get("rootless-a-b", next_rung())
            .expect("the content seeds every runnable rung");
        for second in [9, 18, 27] {
            tap(&mut coach, true, second);
        }

        coach.apply(&CoachEvent::ClickInterrupted { now: at(30) });

        let above = coach
            .mastery
            .get("rootless-a-b", next_rung())
            .expect("the rung above");
        assert_ne!(
            above.prior, seed.prior,
            "the pass was banked before the phone rang, so the level-up is owed"
        );
    }

    #[test]
    fn the_pulse_key_survives_a_tap_and_moves_when_the_ladder_acts() {
        let mut coach = playing();
        let pulse = coach.view().drill.unwrap().pulse_seq;

        tap(&mut coach, true, 9);
        assert_eq!(
            coach.view().drill.unwrap().pulse_seq,
            pulse,
            "a verdict is not a reason to break the pulse"
        );

        coach.apply(&CoachEvent::Stuck { now: at(12) });
        assert_ne!(
            coach.view().drill.unwrap().pulse_seq,
            pulse,
            "a new tempo is a new pulse"
        );
    }

    #[test]
    fn the_question_follows_the_tempo_the_ladder_dropped_it_to() {
        let mut coach = playing();
        coach.apply(&CoachEvent::Stuck { now: at(5) });

        let drill = coach.view().drill.unwrap();
        assert_eq!(drill.tempo_bpm, Some(96));
        assert_eq!(drill.gate_question, "Clean at 96?");
        assert_eq!(drill.gate_summary, "3 clean at 96");
        assert_eq!(
            drill.phase,
            DrillPhase::Playing,
            "escalation acts rather than narrates — the screen just plays on"
        );
    }

    #[test]
    fn the_glance_after_a_tap_yields_to_the_next_beat() {
        let mut coach = playing();
        coach.apply(&CoachEvent::Beat { beat_index: 32 });
        coach.apply(&CoachEvent::Tap {
            clean: false,
            now: at(9),
        });

        assert_eq!(
            coach.view().drill.unwrap().phase,
            DrillPhase::Acknowledged { clean: false },
            "the glance carries the verdict and nothing to read (#1184)"
        );

        coach.apply(&CoachEvent::Beat { beat_index: 33 });
        assert_eq!(
            coach.view().drill.unwrap().phase,
            DrillPhase::Playing,
            "half a second at 120bpm, and the page turns back to the hands (T10)"
        );
    }

    #[test]
    fn the_first_count_in_of_a_block_has_no_glance_to_show() {
        let mut coach = CoachState::default();
        coach.session.start_fixture(at(0));
        coach.apply(&CoachEvent::StartBlock { now: at(0) });

        assert_eq!(
            coach.view().drill.unwrap().phase,
            DrillPhase::CountIn { remaining: 4 },
            "a fresh block counts in on the during-play page"
        );
    }

    #[test]
    fn the_gate_open_moment_shows_the_criterion_met() {
        let mut coach = playing();
        for second in [9, 18, 27] {
            tap(&mut coach, true, second);
        }

        let drill = coach.view().drill.unwrap();
        assert_eq!(drill.phase, DrillPhase::GateOpen);
        assert_eq!((drill.gate_filled, drill.gate_target), (3, 3));
        assert_eq!(drill.gate_summary, "3 clean at 120");
    }

    #[test]
    fn the_clock_only_moves_when_the_shell_says_so() {
        let mut coach = playing();
        assert_eq!(coach.view().drill.unwrap().elapsed_seconds, 0);

        coach.apply(&CoachEvent::Tick { now: at(97) });
        assert_eq!(coach.view().drill.unwrap().elapsed_seconds, 97);
    }

    // ── The soft landing (#1323) ──

    /// Built by hand because the wording is what is under test here; the state
    /// machine's tests own the counting.
    fn landed(
        blocks_played: u16,
        minutes: u16,
        gates_passed: u16,
        plan_finished: bool,
    ) -> CoachView {
        let mut coach = CoachState::default();
        coach.session.state = SessionState::Closing {
            landing: Some(SoftLanding {
                blocks_played,
                minutes,
                gates_passed,
                plan_finished,
            }),
        };
        coach.view()
    }

    #[test]
    fn there_is_no_landing_until_a_session_has_ended() {
        assert_eq!(CoachState::default().view().landing, None);
        assert_eq!(playing().view().landing, None);
    }

    #[test]
    fn a_short_session_is_told_what_it_banked_and_never_what_it_missed() {
        let landing = landed(2, 12, 1, false).landing.expect("a landing");

        assert_eq!(
            landing.detail.as_deref(),
            Some("2 blocks, 12 minutes. 1 gate passed.")
        );
        let note = landing.note.expect(
            "quitting at minute eight is one of the design's three failure \
             stories, so a short session gets one true sentence about it",
        );
        let said = format!("{} {note}", landing.headline).to_lowercase();
        for word in ["left", "remaining", "unfinished", "missed", "incomplete"] {
            assert!(
                !said.contains(word),
                "the landing frames progress, never loss: {said:?} says {word:?}"
            );
        }
    }

    #[test]
    fn a_finished_session_is_not_offered_consolation() {
        let short = landed(2, 12, 1, false).landing.expect("a landing");
        let finished = landed(3, 24, 3, true).landing.expect("a landing");

        assert_eq!(
            finished.note, None,
            "there is nothing to soften: the plan was run out"
        );
        assert_ne!(
            finished.headline, short.headline,
            "and finishing does not read the same as stopping early, or the \
             landing says nothing at all"
        );
        assert_eq!(
            finished.detail.as_deref(),
            Some("3 blocks, 24 minutes. 3 gates passed.")
        );
    }

    #[test]
    fn a_session_that_played_nothing_is_given_no_scoreboard() {
        let landing = landed(0, 0, 0, false).landing.expect("a landing");

        assert_eq!(
            landing.detail, None,
            "\"0 blocks, under a minute\" is a scoreboard for a session that did \
             not happen"
        );
        assert!(landing.note.is_some(), "and it still says something true");
    }

    #[test]
    fn the_landing_counts_in_the_words_a_musician_would_use() {
        assert_eq!(
            landed(1, 1, 1, true).landing.unwrap().detail.as_deref(),
            Some("1 block, 1 minute. 1 gate passed."),
            "singulars, not \"1 blocks\""
        );
        assert_eq!(
            landed(1, 0, 0, false).landing.unwrap().detail.as_deref(),
            Some("1 block, under a minute."),
            "a block closed inside a minute took some time, and \"0 minutes\" \
             says it took none"
        );
        assert_eq!(
            landed(2, 5, 0, false).landing.unwrap().detail.as_deref(),
            Some("2 blocks, 5 minutes."),
            "no gate clause where no gate opened, rather than \"0 gates passed\""
        );
    }

    #[test]
    fn the_landing_survives_the_ffi_wire() {
        for view in [
            landed(2, 12, 1, false),
            landed(3, 24, 3, true),
            landed(0, 0, 0, false),
        ] {
            assert_round_trips(view);
        }
        assert_round_trips(crate::app::Event::Coach(CoachEvent::CloseLanding));
    }

    // ── Press-start: the plan before it runs (#1182, #1189) ──

    fn press_start_preview() -> CoachState {
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::PlanSession {
            now: at(0),
            available_minutes: Some(20),
            utc_offset_minutes: 0,
        });
        coach
    }

    #[test]
    fn planning_a_session_shows_it_without_starting_it() {
        let coach = press_start_preview();
        let plan = coach.view().plan.expect("the press-start surface");

        assert_eq!(coach.view().drill, None, "nothing is running yet");
        assert!(plan.blocks.len() > 1, "a session has a shape");
        assert_eq!(
            plan.total_minutes,
            plan.blocks.iter().map(|block| block.minutes).sum::<u16>()
        );
        assert!(
            plan.blocks.iter().all(|block| !block.why.is_empty()),
            "every block says why it is there"
        );
        assert!(
            !plan.deferred.is_empty(),
            "and what today could not take is on the surface too, not dropped"
        );
    }

    #[test]
    fn a_planned_session_runs_the_plan_that_was_shown() {
        let mut coach = press_start_preview();
        let shown = coach.view().plan.expect("a preview").blocks[0]
            .drill_title
            .clone();

        coach.apply(&CoachEvent::StartPlannedSession {
            now: at(1),
            utc_offset_minutes: 0,
        });

        let drill = coach.view().drill.expect("a running drill");
        assert_eq!(
            drill.drill_title, shown,
            "press start runs the first block of the plan press-start showed"
        );
        assert_eq!(
            coach.view().plan,
            None,
            "and the press-start surface gives way to the drill"
        );
    }

    #[test]
    fn planning_a_session_leaves_no_recovery_blob_behind() {
        let mut coach = CoachState::default();

        let writes = coach.apply(&CoachEvent::PlanSession {
            now: at(0),
            available_minutes: Some(20),
            utc_offset_minutes: 0,
        });

        assert_eq!(
            writes.snapshot,
            SnapshotAction::Unchanged,
            "the blob means a block was cut off mid-flight. A planned session is \
             remade from the content and the clock, and a blob holding one sends \
             the shell into recovery, which hands back a session with no drill \
             and dead-ends press-start (#1219)"
        );
    }

    #[test]
    fn press_start_reaches_a_drill_on_the_path_the_shell_actually_takes() {
        let mut coach = CoachState::default();
        let planning = coach.apply(&CoachEvent::PlanSession {
            now: at(0),
            available_minutes: Some(20),
            utc_offset_minutes: 0,
        });

        // The shell's rule (DrillLoopHost.run): a blob means recover, no blob
        // means start. Follow whichever branch the core just asked for.
        if planning.snapshot == SnapshotAction::Save {
            let blob = coach.session.clone();
            coach.apply(&CoachEvent::RecoverSession {
                session: blob,
                now: at(1),
                utc_offset_minutes: 0,
            });
        } else {
            coach.apply(&CoachEvent::StartPlannedSession {
                now: at(1),
                utc_offset_minutes: 0,
            });
        }

        assert!(
            coach.view().drill.is_some(),
            "whichever branch the snapshot sends the shell down has to end at a \
             drill, or the headline feature never runs on a device (#1219)"
        );
    }

    #[test]
    fn a_block_in_flight_is_still_saved_for_crash_recovery() {
        let mut coach = CoachState::default();
        let writes = coach.apply(&CoachEvent::StartPlannedSession {
            now: at(0),
            utc_offset_minutes: 0,
        });

        assert_eq!(
            writes.snapshot,
            SnapshotAction::Save,
            "a running block is exactly what the blob is for (#1181)"
        );
    }

    #[test]
    fn arriving_back_on_practice_leaves_the_block_in_flight_alone() {
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::StartPlannedSession {
            now: at(0),
            utc_offset_minutes: 0,
        });
        let running = coach.view().drill.expect("a running drill").drill_title;

        let writes = coach.apply(&CoachEvent::PlanSession {
            now: at(5),
            available_minutes: Some(20),
            utc_offset_minutes: 0,
        });

        assert_eq!(
            coach.view().drill.map(|drill| drill.drill_title),
            Some(running),
            "the shell plans on every arrival at Practice, so planning must never \
             drop the block in flight (#1219)"
        );
        assert!(
            writes.blocks.is_empty(),
            "and nothing was recorded, because nothing ended"
        );
    }

    // ── The launch recovery prompt (#1193, #1305) ──

    fn crashed(state: &str) -> EngineSession {
        let mut coach = CoachState::default();
        match state {
            "running" => {
                coach.apply(&CoachEvent::StartPlannedSession {
                    now: at(0),
                    utc_offset_minutes: 0,
                });
                tap(&mut coach, true, 9);
            }
            "run-through" => {
                coach.apply(&CoachEvent::StartRunThrough {
                    item_id: "01J000000000000000000PIECE".into(),
                    title: "Alice in Wonderland".into(),
                    sections: vec!["A".into(), "Bridge".into(), "Out".into()],
                    now: at(0),
                });
                coach.apply(&CoachEvent::JudgeSection {
                    held: true,
                    now: at(40),
                });
            }
            "off-piste" => {
                coach.apply(&CoachEvent::GoOffPiste {
                    item_id: Some("01J000000000000000000PIECE".into()),
                    now: at(0),
                });
            }
            "unmonitored" => {
                coach.apply(&CoachEvent::GoUnmonitored { now: at(0) });
            }
            other => panic!("no fixture for {other}"),
        }
        coach.session
    }

    fn offered(state: &str) -> CoachState {
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::OfferRecovery {
            session: crashed(state),
        });
        coach
    }

    #[test]
    fn a_blob_found_at_launch_is_offered_rather_than_resumed() {
        let coach = offered("running");

        assert!(
            coach.view().recovery.is_some(),
            "the prompt is the whole fix: a user who crashed out of a drill they \
             no longer want must be able to decline it (#1193)"
        );
        assert_eq!(
            coach.view().drill,
            None,
            "and nothing runs until they answer — an offer, not a resumption"
        );
    }

    #[test]
    fn resuming_the_offer_hands_back_the_session_it_held() {
        let mut coach = offered("running");
        let writes = coach.apply(&CoachEvent::AcceptRecovery {
            now: at(600),
            utc_offset_minutes: 0,
        });

        let drill = coach.view().drill.expect("the recovered block");
        assert_eq!(
            drill.gate_filled, 1,
            "the evidence already banked survives the prompt (#1181)"
        );
        assert_eq!(
            coach.view().recovery,
            None,
            "and the prompt goes with the answer"
        );
        assert_eq!(
            writes.snapshot,
            SnapshotAction::Save,
            "a resumed session is a session in flight again, so it earns a blob"
        );
    }

    #[test]
    fn discarding_the_offer_clears_the_blob_that_would_offer_it_again() {
        let mut coach = offered("running");
        let writes = coach.apply(&CoachEvent::DeclineRecovery);

        assert_eq!(
            coach.view().recovery,
            None,
            "declined, so the prompt is gone"
        );
        assert_eq!(coach.view().drill, None, "and nothing was started by it");
        assert_eq!(
            writes.snapshot,
            SnapshotAction::ClearOffer,
            "Discard has to reach the blob itself, or the same session is offered \
             again on the next launch (#1193)"
        );
    }

    #[test]
    fn the_prompt_is_worded_for_the_altitude_the_blob_holds() {
        let wordings: Vec<(String, String)> =
            ["running", "run-through", "off-piste", "unmonitored"]
                .iter()
                .map(|state| {
                    let view = offered(state).view().recovery.expect("a prompt");
                    (view.headline, view.detail)
                })
                .collect();

        let headlines: Vec<_> = wordings.iter().map(|(headline, _)| headline).collect();
        let mut distinct = headlines.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            wordings.len(),
            "one headline per altitude: coming back to a gated drill is not the \
             same offer as coming back to off-the-record play, and a prompt that \
             says the same thing either way is asking the user to guess which \
             they consented to (decision 16): {headlines:?}"
        );
        assert!(
            wordings.iter().all(|(_, detail)| !detail.is_empty()),
            "every altitude says what it was, or the prompt asks the user to \
             guess what they are resuming"
        );

        let (_, running) = &wordings[0];
        assert!(
            running.contains("block 1 of"),
            "a prescribed session says where in the plan it stopped: {running}"
        );
        let (_, run) = &wordings[1];
        assert!(
            run.contains("Alice in Wonderland") && run.contains("1 of 3 sections"),
            "and a run-through says the piece and the verdicts already given: {run}"
        );
    }

    #[test]
    fn the_offer_carries_the_instant_the_session_actually_started() {
        let mut wandering = CoachState::default();
        wandering.apply(&CoachEvent::GoOffPiste {
            item_id: None,
            now: at(0),
        });
        // Twenty minutes in, so the two instants differ: projecting the wrong
        // one is invisible on a blob where they are the same.
        wandering.apply(&CoachEvent::Tick { now: at(1200) });

        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::OfferRecovery {
            session: wandering.session,
        });
        let view = coach.view().recovery.expect("a prompt");

        assert_eq!(
            view.started_at,
            at(0),
            "the rebase to 'now' happens on accept, so the prompt still shows \
             when the user left rather than when they came back"
        );
    }

    #[test]
    fn a_blob_that_recovers_nothing_is_cleared_rather_than_offered() {
        let mut planned = CoachState::default();
        planned.apply(&CoachEvent::PlanSession {
            now: at(0),
            available_minutes: Some(20),
            utc_offset_minutes: 0,
        });

        let mut coach = CoachState::default();
        let writes = coach.apply(&CoachEvent::OfferRecovery {
            session: planned.session,
        });

        assert_eq!(
            coach.view().recovery,
            None,
            "a plan is remade from the content and the clock, so offering one \
             would dead-end on a prompt that recovers no drill (#1219)"
        );
        assert_eq!(
            writes.snapshot,
            SnapshotAction::ClearOffer,
            "and the dud blob goes, rather than sitting in UserDefaults for the \
             life of the install"
        );
    }

    #[test]
    fn an_offer_never_replaces_a_session_already_in_flight() {
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::StartPlannedSession {
            now: at(0),
            utc_offset_minutes: 0,
        });

        let writes = coach.apply(&CoachEvent::OfferRecovery {
            session: crashed("run-through"),
        });

        assert_eq!(
            coach.view().recovery,
            None,
            "the blocks in flight have evidence riding on them — the same rule \
             every other door into a session obeys"
        );
        assert!(coach.view().drill.is_some(), "and the block plays on");
        assert_eq!(
            writes.snapshot,
            SnapshotAction::Unchanged,
            "and the blob is left alone: it belongs to the session in flight \
             now, so clearing it here would strip that session's own crash \
             protection"
        );
    }

    /// `adopt_plan` does not go through `apply`, so no test written against
    /// `apply` could see this one — and it was a live wrong-action bug.
    #[test]
    fn starting_a_composed_session_answers_the_offer_it_did_not_take() {
        let mut coach = offered("running");
        coach.adopt_plan(Plan::fixture(), at(600));

        assert_eq!(
            coach.view().recovery,
            None,
            "the composition is what the user pressed Start on — an offer still \
             live behind it is a session waiting to replace theirs"
        );
        assert!(
            coach.view().drill.is_some(),
            "and the composition is running"
        );
    }

    /// Launch, ignore the prompt, go and play something else, come back: the
    /// session run in between took the blob's place on disk.
    #[test]
    fn an_offer_does_not_outlive_a_session_run_after_it() {
        let mut coach = offered("running");
        coach.apply(&CoachEvent::GoUnmonitored { now: at(100) });
        let writes = coach.apply(&CoachEvent::CloseSession { now: at(200) });

        assert_eq!(
            writes.snapshot,
            SnapshotAction::Clear,
            "closing the unmonitored run removes the blob from UserDefaults"
        );
        assert_eq!(
            coach.view().recovery,
            None,
            "so the prompt cannot still be offering what was written in it"
        );
    }

    /// The narrower half of the same rule: a clear answers the offer even where
    /// the state never left the ones that accept something new.
    #[test]
    fn an_offer_does_not_outlive_the_blob_it_was_made_from() {
        let mut coach = offered("running");
        let writes = coach.apply(&CoachEvent::LeaveSession { now: at(100) });

        assert_eq!(writes.snapshot, SnapshotAction::Clear, "the blob is gone");
        assert_eq!(coach.view().recovery, None, "so the offer is too");
    }

    #[test]
    fn recovering_by_the_drill_loops_own_door_takes_the_offer_with_it() {
        let mut coach = offered("running");
        coach.apply(&CoachEvent::RecoverSession {
            session: crashed("running"),
            now: at(600),
            utc_offset_minutes: 0,
        });

        assert_eq!(
            coach.view().recovery,
            None,
            "answered by the other door, so leaving the prompt up would offer a \
             session that is already running"
        );
    }

    /// A double press, or the drill loop's entry arriving behind the prompt's.
    #[test]
    fn resuming_a_second_time_does_not_rewind_the_session() {
        // The fixture plan, whose gate wants three passes: today's opens on one,
        // so the blob would come back on a gate the first tick closes.
        let mut mid_block = playing();
        mid_block.apply(&CoachEvent::Tick { now: at(30) });
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::OfferRecovery {
            session: mid_block.session,
        });

        coach.apply(&CoachEvent::AcceptRecovery {
            now: at(600),
            utc_offset_minutes: 0,
        });
        coach.apply(&CoachEvent::Tick { now: at(660) });
        let elapsed = coach
            .view()
            .drill
            .expect("the recovered block")
            .elapsed_seconds;
        assert!(elapsed >= 60, "a minute of it has been played: {elapsed}");

        coach.apply(&CoachEvent::AcceptRecovery {
            now: at(720),
            utc_offset_minutes: 0,
        });

        assert_eq!(
            coach.view().drill.expect("still running").elapsed_seconds,
            elapsed,
            "the offer was taken, so there is nothing left to take: a second \
             Resume that re-ran it would rebase the block to the blob and throw \
             away the minute played since"
        );
    }

    #[test]
    fn the_recovery_prompt_survives_the_ffi_wire() {
        for state in ["running", "run-through", "off-piste", "unmonitored"] {
            assert_round_trips(offered(state).view());
        }
        assert_round_trips(crate::app::Event::Coach(CoachEvent::OfferRecovery {
            session: crashed("running"),
        }));
        assert_round_trips(crate::app::Event::Coach(CoachEvent::AcceptRecovery {
            now: at(0),
            utc_offset_minutes: 60,
        }));
        assert_round_trips(crate::app::Event::Coach(CoachEvent::DeclineRecovery));
    }

    #[test]
    fn pressing_start_with_nothing_planned_plans_first() {
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::StartPlannedSession {
            now: at(0),
            utc_offset_minutes: 0,
        });

        assert!(
            coach.view().drill.is_some(),
            "a shell that wants no preview can start in one event"
        );
    }

    #[test]
    fn the_session_length_falls_back_to_the_authored_default() {
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::PlanSession {
            now: at(0),
            available_minutes: None,
            utc_offset_minutes: 0,
        });

        let plan = coach.view().plan.expect("a plan");
        assert!(
            plan.total_minutes > 0 && plan.total_minutes <= 20,
            "the [defaults] session length until a surface can ask: {}",
            plan.total_minutes
        );
    }

    // ── The mastery track, fed by the loop (#1188) ──

    fn fixture_level() -> ParameterLevel {
        ParameterLevel {
            tempo_bpm: 120,
            click_level: ClickLevel::TwoAndFour,
        }
    }

    /// One pass of the phrase, answered, from wherever the block has got to.
    fn tap(coach: &mut CoachState, clean: bool, second: i64) {
        tap_at(coach, clean, at(second));
    }

    fn tap_at(coach: &mut CoachState, clean: bool, now: DateTime<Utc>) {
        if coach.session.phase() == Some(&Phase::BlockEntry) {
            coach.apply(&CoachEvent::StartBlock { now });
        }
        if matches!(coach.session.phase(), Some(Phase::CountIn { .. })) {
            coach.apply(&CoachEvent::Beat { beat_index: 0 });
        }
        let block = coach.session.block().expect("a running block");
        let phrase = block.body_beats();
        let boundary = (block.beat_index / phrase + 1) * phrase;
        coach.apply(&CoachEvent::Beat {
            beat_index: boundary,
        });
        coach.apply(&CoachEvent::Tap { clean, now });
    }

    #[test]
    fn a_tap_is_evidence_against_the_rung_it_was_played_at() {
        let mut coach = playing();
        tap(&mut coach, true, 9);

        let mastery = coach
            .mastery
            .get("rootless-a-b", fixture_level())
            .expect("evidence at the rung the block ran");
        assert!((mastery.evidence() - 1.0).abs() < 0.01);
        assert_eq!(mastery.last_attempt_at, Some(at(9)));
        assert!(
            mastery.estimate() > 0.5,
            "a clean tap moves the estimate up: {}",
            mastery.estimate()
        );
    }

    #[test]
    fn a_missed_tap_is_evidence_too() {
        let mut coach = playing();
        tap(&mut coach, false, 9);

        let mastery = coach.mastery.get("rootless-a-b", fixture_level()).unwrap();
        assert!(
            mastery.estimate() < 0.5,
            "{} should have fallen",
            mastery.estimate()
        );
    }

    #[test]
    fn a_gate_pass_moves_the_cursor_up_the_ladder_without_resetting_it() {
        let mut coach = playing();
        let seed = *coach
            .mastery
            .get("rootless-a-b", next_rung())
            .expect("the content seeds every runnable rung");

        for second in [9, 18, 27] {
            tap(&mut coach, true, second);
        }
        coach.apply(&CoachEvent::Tick { now: at(40) });

        let above = coach
            .mastery
            .get("rootless-a-b", next_rung())
            .expect("the rung above");
        assert_ne!(
            above.prior, seed.prior,
            "passing the gate re-seeds the rung above from what the hands just said"
        );
        assert!(
            above.estimate() > seed.estimate(),
            "and it inherits optimism rather than starting from the content seed: \
             {} against {}",
            above.estimate(),
            seed.estimate()
        );
        assert_eq!(above.evidence(), 0.0, "inheritance is prior, not evidence");
    }

    #[test]
    fn the_first_rep_of_the_day_on_returning_material_is_the_cold_test() {
        let mut coach = CoachState::default();
        coach.mastery.record(
            "rootless-a-b",
            fixture_level(),
            Verdict::Clean,
            at(0) - TimeDelta::days(3),
        );
        coach.session.start_fixture(at(0));

        tap(&mut coach, true, 9);
        tap(&mut coach, true, 18);

        let attempts = &coach.session.block().unwrap().attempts;
        assert!(
            attempts[0].cold,
            "played cold, three days after the last go"
        );
        assert!(!attempts[1].cold, "the second rep is warm");
    }

    #[test]
    fn material_practised_earlier_the_same_day_is_not_a_cold_test() {
        let mut coach = CoachState::default();
        coach.mastery.record(
            "rootless-a-b",
            fixture_level(),
            Verdict::Clean,
            at(0) - TimeDelta::hours(1),
        );
        coach.session.start_fixture(at(0));

        tap(&mut coach, true, 9);

        assert!(!coach.session.block().unwrap().attempts[0].cold);
    }

    /// Without the wiring the same instant reads as one warm evening (#1221).
    #[test]
    fn the_cold_test_turns_the_day_over_where_the_shell_says_the_user_is() {
        let last_night = Utc.with_ymd_and_hms(2026, 8, 4, 21, 0, 0).unwrap();
        // 00:30 local in BST, which UTC still calls the 4th.
        let after_midnight = Utc.with_ymd_and_hms(2026, 8, 4, 23, 30, 0).unwrap();

        let mut in_london = CoachState::default();
        in_london
            .mastery
            .record("rootless-a-b", fixture_level(), Verdict::Clean, last_night);
        in_london.apply(&CoachEvent::PlanSession {
            now: after_midnight,
            available_minutes: Some(20),
            utc_offset_minutes: 60,
        });
        in_london.session.start_fixture(after_midnight);
        tap_at(&mut in_london, true, after_midnight);

        assert!(
            in_london.session.block().unwrap().attempts[0].cold,
            "half past midnight is a new day, and the first rep of it is cold"
        );

        let mut in_utc = CoachState::default();
        in_utc
            .mastery
            .record("rootless-a-b", fixture_level(), Verdict::Clean, last_night);
        in_utc.apply(&CoachEvent::PlanSession {
            now: after_midnight,
            available_minutes: Some(20),
            utc_offset_minutes: 0,
        });
        in_utc.session.start_fixture(after_midnight);
        tap_at(&mut in_utc, true, after_midnight);

        assert!(
            !in_utc.session.block().unwrap().attempts[0].cold,
            "the same instant reported from UTC is still the same evening"
        );
    }

    /// The no-preview door is documented as one a shell may use alone, so it
    /// has to report the offset too or it silently puts #1221 back.
    #[test]
    fn starting_without_a_preview_still_decides_cold_on_the_users_day() {
        let last_night = Utc.with_ymd_and_hms(2026, 8, 4, 21, 0, 0).unwrap();
        let after_midnight = Utc.with_ymd_and_hms(2026, 8, 4, 23, 30, 0).unwrap();

        let mut coach = CoachState::default();
        coach
            .mastery
            .record("rootless-a-b", fixture_level(), Verdict::Clean, last_night);
        coach.apply(&CoachEvent::StartPlannedSession {
            now: after_midnight,
            utc_offset_minutes: 60,
        });
        coach.session.start_fixture(after_midnight);
        tap_at(&mut coach, true, after_midnight);

        assert!(coach.session.block().unwrap().attempts[0].cold);
    }

    fn next_rung() -> ParameterLevel {
        ParameterLevel {
            tempo_bpm: 80,
            click_level: ClickLevel::EveryBeat,
        }
    }

    // ── Rebuilt from the persisted evidence at launch (#1214) ──

    fn closed_block(id: &str, verdicts: &[(bool, i64)], exit: Exit) -> BlockRecord {
        use crate::engine::gate::EvidenceSource;
        use crate::engine::session::AttemptSummary;
        BlockRecord {
            id: id.to_string(),
            started_at: at(0),
            ended_at: at(verdicts.last().map(|(_, second)| *second).unwrap_or(0)),
            attempts: verdicts
                .iter()
                .map(|(clean, second)| AttemptSummary {
                    at: at(*second),
                    verdict: if *clean {
                        Verdict::Clean
                    } else {
                        Verdict::Missed
                    },
                    source: EvidenceSource::TapVerdict,
                    cold: false,
                    self_predicted: None,
                    level: fixture_level(),
                })
                .collect(),
            attempts_to_pass: None,
            gate_opened_at_attempt: None,
            exit,
            ..BlockRecord::fixture()
        }
    }

    #[test]
    fn replaying_a_persisted_record_rebuilds_the_evidence_it_holds() {
        let mut coach = CoachState::default();
        coach.rebuild_mastery(
            vec![closed_block(
                "b1",
                &[(true, 9), (false, 18)],
                Exit::SessionEnded,
            )],
            vec![],
        );

        let mastery = coach
            .mastery
            .get("rootless-a-b", fixture_level())
            .expect("evidence at the rung the block ran");
        assert!((mastery.evidence() - 2.0).abs() < 0.01, "both attempts");
        assert_eq!(
            mastery.last_attempt_at,
            Some(at(18)),
            "so overdue and the cold test read the real gap, not the launch"
        );
    }

    #[test]
    fn a_replayed_gate_pass_reseeds_the_rung_above_like_the_live_one_did() {
        let mut coach = CoachState::default();
        let seed = *coach
            .mastery
            .get("rootless-a-b", next_rung())
            .expect("the content seeds every runnable rung");

        coach.rebuild_mastery(
            vec![closed_block(
                "b1",
                &[(true, 9), (true, 18), (true, 27)],
                Exit::GatePassed,
            )],
            vec![],
        );

        let above = coach.mastery.get("rootless-a-b", next_rung()).unwrap();
        assert_ne!(above.prior, seed.prior, "the level-up replayed too");
        assert_eq!(above.evidence(), 0.0, "inheritance is prior, not evidence");
    }

    /// Decision 17 has to hold on an *authored* node too. A `journal:`-prefixed
    /// id makes the built-session case unobservable — `next_rung` simply cannot
    /// find the node — so this pins the rule to `BlockOrigin`, which is what
    /// Phase C needs when the run-through altitude puts a real node on the
    /// judgement track.
    #[test]
    fn a_judgement_gate_pass_banks_nothing_on_either_path() {
        let passed = closed_block("b1", &[(true, 9), (true, 18), (true, 27)], Exit::GatePassed);
        let judgement = BlockRecord {
            origin: BlockOrigin::Judgement,
            ..passed.clone()
        };

        assert_eq!(
            banked_passes(&[passed]).count(),
            1,
            "an authored pass banks"
        );
        assert_eq!(
            banked_passes(std::slice::from_ref(&judgement)).count(),
            0,
            "the same pass on the judgement track does not"
        );

        let mut replayed = CoachState::default();
        replayed.rebuild_mastery(vec![judgement], vec![]);
        assert_eq!(
            replayed.mastery,
            CoachState::default().mastery,
            "and the launch replay agrees, attempts and level-up alike"
        );
    }

    #[test]
    fn a_second_load_replaces_what_the_first_one_built() {
        let record = closed_block("b1", &[(true, 9)], Exit::SessionEnded);
        let mut coach = CoachState::default();
        coach.rebuild_mastery(vec![record.clone()], vec![]);
        let once = coach.mastery.clone();

        coach.rebuild_mastery(vec![record], vec![]);

        assert_eq!(
            coach.mastery, once,
            "a reload must never double the evidence"
        );
    }

    #[test]
    fn records_replay_in_time_order_however_the_store_returns_them() {
        let earlier = closed_block("b1", &[(true, 9)], Exit::SessionEnded);
        let later = closed_block("b2", &[(false, 100)], Exit::SessionEnded);

        let mut coach = CoachState::default();
        coach.rebuild_mastery(vec![later.clone(), earlier.clone()], vec![]);

        let mut sorted = CoachState::default();
        sorted.rebuild_mastery(vec![earlier, later], vec![]);
        assert_eq!(coach.mastery, sorted.mastery);
        assert_eq!(
            coach
                .mastery
                .get("rootless-a-b", fixture_level())
                .unwrap()
                .last_attempt_at,
            Some(at(100)),
            "the newest attempt wins whatever order the rows arrive in"
        );
    }

    #[test]
    fn a_restart_rebuilds_the_store_the_live_session_left_behind() {
        let mut live = CoachState::default();
        live.session.start_fixture(at(0));
        let mut records = Vec::new();
        for second in [9, 18, 27] {
            records.extend(tap_collect(&mut live, true, second));
        }
        records.extend(live.apply(&CoachEvent::Tick { now: at(40) }).blocks);
        assert!(!records.is_empty(), "the run must have closed a block");

        let mut rebuilt = CoachState::default();
        rebuilt.rebuild_mastery(records, vec![]);

        assert_eq!(
            rebuilt.mastery, live.mastery,
            "a restart loses nothing the records hold"
        );
    }

    /// Live-vs-rebuilt equality through an escalation: the ladder's first rung
    /// drops the tempo mid-block, so the block holds attempts on two rungs and
    /// a replay keyed only by the block's closing level loses one (#1214).
    #[test]
    fn a_rebuild_reproduces_the_store_an_escalated_block_left_behind() {
        let mut live = playing();
        for second in [9, 18, 27] {
            tap(&mut live, false, second);
        }
        assert!(
            !live
                .session
                .block()
                .expect("a running block")
                .escalation_fired
                .is_empty(),
            "three misses fire the ladder, which is what puts two rungs in one block"
        );
        let mut records = tap_collect(&mut live, true, 36);
        records.extend(live.apply(&CoachEvent::LeaveSession { now: at(40) }).blocks);
        assert!(!records.is_empty(), "the block it ended is written down");

        let mut rebuilt = CoachState::default();
        rebuilt.rebuild_mastery(records, vec![]);

        assert_eq!(
            rebuilt.mastery, live.mastery,
            "the record has to carry what the live path told the mastery track, or \
             the rung the user was actually stuck at comes back as untouched"
        );
    }

    /// `tap`, keeping the records the events closed.
    fn tap_collect(coach: &mut CoachState, clean: bool, second: i64) -> Vec<BlockRecord> {
        let mut records = Vec::new();
        if coach.session.phase() == Some(&Phase::BlockEntry) {
            records.extend(coach.apply(&CoachEvent::StartBlock { now: at(0) }).blocks);
        }
        if matches!(coach.session.phase(), Some(Phase::CountIn { .. })) {
            records.extend(coach.apply(&CoachEvent::Beat { beat_index: 0 }).blocks);
        }
        let block = coach.session.block().expect("a running block");
        let phrase = block.body_beats();
        let boundary = (block.beat_index / phrase + 1) * phrase;
        records.extend(
            coach
                .apply(&CoachEvent::Beat {
                    beat_index: boundary,
                })
                .blocks,
        );
        records.extend(
            coach
                .apply(&CoachEvent::Tap {
                    clean,
                    now: at(second),
                })
                .blocks,
        );
        records
    }

    // ── The #846 hazard: every bridge type on the real wire ──

    #[test]
    fn every_coach_event_survives_the_ffi_wire() {
        for event in [
            CoachEvent::PlanSession {
                now: at(0),
                available_minutes: Some(20),
                utc_offset_minutes: -330,
            },
            CoachEvent::StartPlannedSession {
                now: at(0),
                utc_offset_minutes: -330,
            },
            CoachEvent::CountInBeat { remaining: 3 },
            CoachEvent::Beat { beat_index: 12 },
            CoachEvent::Tap {
                clean: true,
                now: at(1),
            },
            CoachEvent::StartBlock { now: at(1) },
            CoachEvent::SkipBlock { now: at(1) },
            CoachEvent::DiscardAttempt { now: at(1) },
            CoachEvent::ClickInterrupted { now: at(1) },
            CoachEvent::Stuck { now: at(2) },
            CoachEvent::Tick { now: at(3) },
            CoachEvent::LeaveSession { now: at(4) },
            CoachEvent::ClickUnavailable { now: at(8) },
            CoachEvent::GoOffPiste {
                item_id: None,
                now: at(5),
            },
            CoachEvent::GoOffPiste {
                item_id: Some("01J000000000000000000PIECE".into()),
                now: at(5),
            },
            CoachEvent::StartRunThrough {
                item_id: "01J000000000000000000PIECE".into(),
                title: "Alice in Wonderland".into(),
                sections: vec!["A".into(), "Bridge".into()],
                now: at(5),
            },
            CoachEvent::JudgeSection {
                held: true,
                now: at(6),
            },
            CoachEvent::DiscardRunThrough { now: at(7) },
            CoachEvent::GoUnmonitored { now: at(6) },
            CoachEvent::KeepWanderAsDrill { keep: true },
            CoachEvent::CloseSession { now: at(7) },
        ] {
            assert_round_trips(crate::app::Event::Coach(event));
        }
    }

    #[test]
    fn a_recovered_session_carries_the_whole_plan_across_the_wire() {
        let mut coach = press_start_preview();
        coach.apply(&CoachEvent::StartPlannedSession {
            now: at(1),
            utc_offset_minutes: 0,
        });

        assert_round_trips(crate::app::Event::Coach(CoachEvent::RecoverSession {
            session: coach.session.clone(),
            now: at(2),
            utc_offset_minutes: 0,
        }));
    }

    #[test]
    fn the_press_start_surface_survives_the_ffi_wire() {
        assert_round_trips(press_start_preview().view());
    }

    #[test]
    fn the_coach_view_survives_the_ffi_wire() {
        assert_round_trips(CoachState::default().view());

        let mut carded = CoachState::default();
        carded.session.start_fixture(at(0));
        assert_round_trips(carded.view());

        let mut coach = playing();
        coach.apply(&CoachEvent::Beat { beat_index: 32 });
        coach.apply(&CoachEvent::Tap {
            clean: false,
            now: at(9),
        });
        assert_round_trips(coach.view());

        coach.apply(&CoachEvent::Stuck { now: at(10) });
        coach.apply(&CoachEvent::CountInBeat { remaining: 3 });
        assert_round_trips(coach.view());

        let mut awaiting = playing();
        awaiting.apply(&CoachEvent::Beat { beat_index: 32 });
        assert_eq!(
            awaiting.view().drill.unwrap().phase,
            DrillPhase::AwaitingVerdict
        );
        assert_round_trips(awaiting.view());

        let mut open = playing();
        for second in [9, 18, 27] {
            tap(&mut open, true, second);
        }
        assert_eq!(open.view().drill.unwrap().phase, DrillPhase::GateOpen);
        assert_round_trips(open.view());

        // A view with no tempo: the one shape a bincode wire has no "absent"
        // for, and the one the shell would otherwise draw a 0 from.
        let mut untimed = playing_untimed();
        assert_round_trips(untimed.view());
        untimed.apply(&CoachEvent::Tap {
            clean: true,
            now: at(20),
        });
        assert_round_trips(untimed.view());

        // The altitudes, whose view fields are `None` in every case above.
        let mut run = CoachState::default();
        run.apply(&CoachEvent::StartRunThrough {
            item_id: "01J000000000000000000PIECE".into(),
            title: "Alice in Wonderland".into(),
            sections: vec!["A".into(), "Bridge".into()],
            now: at(0),
        });
        assert_round_trips(run.view());
        run.apply(&CoachEvent::JudgeSection {
            held: false,
            now: at(30),
        });
        assert_round_trips(run.view());

        let mut wandering = CoachState::default();
        wandering.apply(&CoachEvent::GoOffPiste {
            item_id: Some("01J000000000000000000PIECE".into()),
            now: at(0),
        });
        assert_round_trips(wandering.view());
    }

    #[test]
    fn an_l0_session_survives_the_ffi_wire() {
        let mut coach = playing_untimed();
        coach.apply(&CoachEvent::Tap {
            clean: true,
            now: at(20),
        });

        assert_round_trips(crate::app::Event::Coach(CoachEvent::RecoverSession {
            session: coach.session.clone(),
            now: at(30),
            utc_offset_minutes: 0,
        }));
    }
}
