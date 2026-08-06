//! The engine's read half of the bridge (spec §6, as scoped by decision 18):
//! one `ViewModel` field, built from the session machine. The capture types
//! (`NoteBatch` and friends) belong to the deferred scoring path and are not
//! here yet.
//!
//! Everything the drill screen draws comes from this module, so counting,
//! gating and what-comes-next stay in Rust.

use serde::{Deserialize, Serialize};

use super::content::ContentIndex;
use super::gate::{Requirement, Verdict};
use super::mastery::MasteryStore;
use super::plan::{plan, ParameterLevel, Plan, PlanContext};
use super::session::{
    BlockRecord, CoachEvent, CoachWrites, EngineSession, Exit, Phase, SessionState,
};
use crate::domain::item::ItemKind;

/// Spec §1 gives this five fields; the judgement track and the interruption
/// ledger arrive with Phase 2b.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CoachState {
    pub session: EngineSession,
    pub mastery: MasteryStore,
}

impl Default for CoachState {
    fn default() -> Self {
        Self {
            session: EngineSession::default(),
            mastery: MasteryStore::seeded_from(ContentIndex::shipped()),
        }
    }
}

impl CoachState {
    pub fn apply(&mut self, event: &CoachEvent) -> CoachWrites {
        self.mark_cold(event);
        let planned = self.plan_for(event);
        let writes = self.session.apply_with_plan(event, planned);
        for attempt in &writes.evidence {
            self.mastery
                .record(&attempt.node, attempt.level, attempt.verdict, attempt.at);
        }
        for record in writes.blocks.iter().filter(|r| r.exit == Exit::GatePassed) {
            if let Some(next) = next_rung(record) {
                self.mastery.level_up(&record.node, record.level, next);
            }
        }
        writes
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
            } => (*now, *available_minutes),
            CoachEvent::StartPlannedSession { now } => (*now, None),
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
        let cold = self.mastery.is_cold(&node, level, now);
        self.session.mark_cold(cold);
    }

    pub fn view(&self) -> CoachView {
        CoachView {
            drill: self.drill_view(),
            plan: self.plan_view(),
        }
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
            tempo_bpm: block.level.tempo_bpm,
            click_level: block.level.click_level.spoken().to_string(),
            beat: block.beat(),
            beats_per_bar: block.beats_per_bar,
            bar: block.bar(),
            bars: block.bars,
            count_in_beats: block.count_in_beats,
            phrase_beats: block.body_beats(),
            pulse_seq: block.pulse_seq,
            pulse_running: block.phase != Phase::BlockEntry,
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
            gate_question: gate_question(&spec.gate.requirement, block.level.tempo_bpm),
            gate_summary: gate_summary(&spec.gate.requirement, block.level.tempo_bpm),
            gate_filled: block.gate_progress.filled(),
            gate_target: block.gate_progress.target(),
        })
    }
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

fn gate_question(requirement: &Requirement, tempo_bpm: u16) -> String {
    match requirement {
        Requirement::CleanPasses { .. } | Requirement::KeyCoverage { .. } => {
            format!("Clean at {tempo_bpm}?")
        }
        Requirement::Chained { .. } => format!("Clean at {tempo_bpm}, no stops?"),
        Requirement::SelfConfirmed { .. } => "Did that match?".to_string(),
    }
}

fn gate_summary(requirement: &Requirement, tempo_bpm: u16) -> String {
    match requirement {
        Requirement::CleanPasses { count, .. } => format!("{count} clean at {tempo_bpm}"),
        Requirement::KeyCoverage {
            keys_required,
            per_key_passes,
            first_attempt,
        } => {
            if *first_attempt {
                format!("clean first time, in {keys_required} keys")
            } else {
                format!("{per_key_passes} clean at {tempo_bpm}, in {keys_required} keys")
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
    pub tempo_bpm: u16,
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
        assert_eq!(drill.tempo_bpm, 120);
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
        assert_eq!(drill.tempo_bpm, 96);
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

    // ── Press-start: the plan before it runs (#1182, #1189) ──

    fn press_start_preview() -> CoachState {
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::PlanSession {
            now: at(0),
            available_minutes: Some(20),
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

        coach.apply(&CoachEvent::StartPlannedSession { now: at(1) });

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
        });

        // The shell's rule (DrillLoopHost.run): a blob means recover, no blob
        // means start. Follow whichever branch the core just asked for.
        if planning.snapshot == SnapshotAction::Save {
            let blob = coach.session.clone();
            coach.apply(&CoachEvent::RecoverSession {
                session: blob,
                now: at(1),
            });
        } else {
            coach.apply(&CoachEvent::StartPlannedSession { now: at(1) });
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
        let writes = coach.apply(&CoachEvent::StartPlannedSession { now: at(0) });

        assert_eq!(
            writes.snapshot,
            SnapshotAction::Save,
            "a running block is exactly what the blob is for (#1181)"
        );
    }

    #[test]
    fn arriving_back_on_practice_leaves_the_block_in_flight_alone() {
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::StartPlannedSession { now: at(0) });
        let running = coach.view().drill.expect("a running drill").drill_title;

        let writes = coach.apply(&CoachEvent::PlanSession {
            now: at(5),
            available_minutes: Some(20),
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

    #[test]
    fn pressing_start_with_nothing_planned_plans_first() {
        let mut coach = CoachState::default();
        coach.apply(&CoachEvent::StartPlannedSession { now: at(0) });

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
        if coach.session.phase() == Some(&Phase::BlockEntry) {
            coach.apply(&CoachEvent::StartBlock { now: at(0) });
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
        coach.apply(&CoachEvent::Tap {
            clean,
            now: at(second),
        });
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

    fn next_rung() -> ParameterLevel {
        ParameterLevel {
            tempo_bpm: 80,
            click_level: ClickLevel::EveryBeat,
        }
    }

    // ── The #846 hazard: every bridge type on the real wire ──

    #[test]
    fn every_coach_event_survives_the_ffi_wire() {
        for event in [
            CoachEvent::PlanSession {
                now: at(0),
                available_minutes: Some(20),
            },
            CoachEvent::StartPlannedSession { now: at(0) },
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
            CoachEvent::GoOffPiste { now: at(5) },
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
        coach.apply(&CoachEvent::StartPlannedSession { now: at(1) });

        assert_round_trips(crate::app::Event::Coach(CoachEvent::RecoverSession {
            session: coach.session.clone(),
            now: at(2),
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
    }
}
