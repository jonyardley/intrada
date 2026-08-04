//! The engine's read half of the bridge (spec §6, as scoped by decision 18):
//! one `ViewModel` field, built from the session machine. The capture types
//! (`NoteBatch` and friends) belong to the deferred scoring path and are not
//! here yet.
//!
//! Everything the drill screen draws comes from this module, so counting,
//! gating and what-comes-next stay in Rust.

use serde::{Deserialize, Serialize};

use super::gate::{Requirement, Verdict};
use super::session::{CoachEvent, CoachWrites, EngineSession, Phase, SessionState};
use crate::domain::item::ItemKind;

/// Spec §1 gives this five fields; the mastery, judgement, ledger and content
/// stores arrive with #1148 and the `gates.toml` parser.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CoachState {
    pub session: EngineSession,
}

impl CoachState {
    pub fn apply(&mut self, event: &CoachEvent) -> CoachWrites {
        self.session.apply(event)
    }

    pub fn view(&self) -> CoachView {
        CoachView {
            drill: self.drill_view(),
        }
    }

    fn drill_view(&self) -> Option<DrillView> {
        let SessionState::Running { plan, .. } = &self.session.state else {
            return None;
        };
        let block = self.session.block()?;
        let spec = self.session.spec()?;

        Some(DrillView {
            phase: match block.phase {
                Phase::AwaitingVerdict => DrillPhase::AwaitingVerdict,
                Phase::GateOpen => DrillPhase::GateOpen,
                Phase::CountIn { beats_remaining } => match block.last_verdict {
                    Some(verdict) => DrillPhase::Acknowledged {
                        clean: verdict == Verdict::Clean,
                    },
                    None => DrillPhase::CountIn {
                        remaining: beats_remaining,
                    },
                },
                Phase::Listening | Phase::Escalating { .. } => DrillPhase::Playing,
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
            click_beats: block.click_beats(),
            elapsed_seconds: block.elapsed_seconds(),
            ceiling_seconds: Some(u32::from(spec.minutes) * 60),
            block_kinds: plan.blocks.iter().map(|block| block.kind.clone()).collect(),
            block_index: block.spec_index,
            gate_question: gate_question(&spec.gate.requirement, block.level.tempo_bpm),
            gate_summary: gate_summary(&spec.gate.requirement, block.level.tempo_bpm),
            gate_filled: block.gate_progress.filled(),
            gate_target: block.gate_progress.target(),
            rep_seq: block.rep_seq,
        })
    }
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
}

/// What the drill screen shows. Deliberately presentational: `Escalating`
/// reads as `Playing`, and the tap's glance holds only until the first
/// count-in click turns the page (#1184, T10).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum DrillPhase {
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
    /// Beats for the shell to schedule per rep, count-in excluded: the phrase
    /// body plus the landing beat that ends it.
    pub click_beats: u32,
    pub elapsed_seconds: u32,
    pub ceiling_seconds: Option<u32>,
    pub block_kinds: Vec<ItemKind>,
    pub block_index: usize,
    pub gate_question: String,
    pub gate_summary: String,
    pub gate_filled: u8,
    pub gate_target: u8,
    /// Changes when the shell should restart the click. Nothing else in this
    /// view tells it that a new rep began at a new tempo or scope.
    pub rep_seq: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::assert_round_trips;
    use chrono::{DateTime, TimeZone, Utc};

    fn at(second: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_754_300_000 + second, 0).unwrap()
    }

    fn playing() -> CoachState {
        let mut coach = CoachState::default();
        coach.session.start_fixture(at(0));
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
        assert_eq!(
            drill.click_beats, 33,
            "32 beats of phrase plus the landing beat"
        );
        assert_eq!(drill.gate_question, "Clean at 120?");
        assert_eq!(drill.gate_summary, "3 clean at 120");
        assert_eq!((drill.gate_filled, drill.gate_target), (0, 3));
        assert_eq!(drill.ceiling_seconds, Some(360));
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
    fn the_glance_after_a_tap_yields_to_the_count_in() {
        let mut coach = playing();
        coach.apply(&CoachEvent::Beat { beat_index: 32 });
        coach.apply(&CoachEvent::Tap {
            clean: false,
            now: at(9),
        });

        let before = coach.view().drill.unwrap();
        assert_eq!(
            before.phase,
            DrillPhase::Acknowledged { clean: false },
            "the glance carries the verdict and nothing to read (#1184)"
        );
        assert_eq!(before.rep_seq, 2);

        coach.apply(&CoachEvent::CountInBeat { remaining: 3 });
        assert_eq!(
            coach.view().drill.unwrap().phase,
            DrillPhase::CountIn { remaining: 3 },
            "the first count-in click turns the page to the next rep's facts"
        );

        coach.apply(&CoachEvent::Beat { beat_index: 0 });
        assert_eq!(
            coach.view().drill.unwrap().phase,
            DrillPhase::Playing,
            "the glance goes when the hands are back"
        );
    }

    #[test]
    fn the_first_count_in_of_a_block_has_no_glance_to_show() {
        let mut coach = CoachState::default();
        coach.session.start_fixture(at(0));

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
            coach.apply(&CoachEvent::Beat { beat_index: 32 });
            coach.apply(&CoachEvent::Tap {
                clean: true,
                now: at(second),
            });
            coach.apply(&CoachEvent::Beat { beat_index: 0 });
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

    // ── The #846 hazard: every bridge type on the real wire ──

    #[test]
    fn every_coach_event_survives_the_ffi_wire() {
        for event in [
            CoachEvent::StartDrillLoop { now: at(0) },
            CoachEvent::CountInBeat { remaining: 3 },
            CoachEvent::Beat { beat_index: 12 },
            CoachEvent::Tap {
                clean: true,
                now: at(1),
            },
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
    fn the_coach_view_survives_the_ffi_wire() {
        assert_round_trips(CoachState::default().view());

        let mut coach = playing();
        coach.apply(&CoachEvent::Beat { beat_index: 32 });
        coach.apply(&CoachEvent::Tap {
            clean: false,
            now: at(9),
        });
        assert_round_trips(coach.view());

        coach.apply(&CoachEvent::CountInBeat { remaining: 3 });
        assert_round_trips(coach.view());
    }
}
