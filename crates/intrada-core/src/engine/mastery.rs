//! The mastery track (spec §2): one Beta state per `(node, parameter_level)`,
//! fed by scored attempts and read three ways.
//!
//! Decay is a read, never a stored mutation, and spacing is computed from the
//! stored counts rather than the decayed ones — using the decayed counts would
//! put elapsed time on both sides of `overdue` and turn a long gap into "wildly
//! overdue" rather than "due".

use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

use super::content::{Band, ContentIndex};
use super::gate::Verdict;
use super::plan::ParameterLevel;

/// Spec §2's constants. They are `gates.toml` data by rights (lever 2), and the
/// file authors none of them yet, so this struct is the one place they live
/// until it does.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct MasteryConstants {
    pub evidence_max: f32,
    pub confidence_k: f32,
    pub base_interval_days: f32,
    pub e_scale: f32,
    pub retention_half_life_days: f32,
    pub transfer: f32,
    pub inherit_max: f32,
}

impl Default for MasteryConstants {
    fn default() -> Self {
        Self {
            evidence_max: 40.0,
            confidence_k: 8.0,
            base_interval_days: 2.0,
            e_scale: 8.0,
            retention_half_life_days: 30.0,
            transfer: 0.35,
            inherit_max: 6.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Mastery {
    pub alpha: f32,
    pub beta: f32,
    pub prior: (f32, f32),
    /// `None` on a prior nothing has been attempted against yet.
    pub last_attempt_at: Option<DateTime<Utc>>,
}

impl Mastery {
    pub fn from_prior(alpha: f32, beta: f32) -> Self {
        Self {
            alpha,
            beta,
            prior: (alpha, beta),
            last_attempt_at: None,
        }
    }

    pub fn estimate(&self) -> f32 {
        let total = self.alpha + self.beta;
        if total <= 0.0 {
            return 0.0;
        }
        self.alpha / total
    }

    /// Attempts beyond the prior.
    pub fn evidence(&self) -> f32 {
        (self.alpha + self.beta - self.prior.0 - self.prior.1).max(0.0)
    }

    pub fn confidence(&self, constants: &MasteryConstants) -> f32 {
        let evidence = self.evidence();
        evidence / (evidence + constants.confidence_k)
    }

    pub fn record(&mut self, verdict: Verdict, at: DateTime<Utc>, constants: &MasteryConstants) {
        match verdict {
            Verdict::Clean => self.alpha += 1.0,
            Verdict::Missed => self.beta += 1.0,
        }
        let total = self.alpha + self.beta;
        if total > constants.evidence_max {
            // Proportional, so the estimate the attempt just produced survives
            // exactly and the oldest evidence is what the cap discards.
            let scale = constants.evidence_max / total;
            self.alpha *= scale;
            self.beta *= scale;
        }
        self.last_attempt_at = Some(at);
    }

    /// Elapsed time pulls the counts toward the prior, never toward zero and
    /// never toward failure: absence of practice is absence of evidence.
    pub fn decayed_at(&self, now: DateTime<Utc>, constants: &MasteryConstants) -> Mastery {
        let Some(last) = self.last_attempt_at else {
            return *self;
        };
        let lambda = 0.5f32.powf(1.0 / constants.retention_half_life_days.max(f32::EPSILON));
        let factor = lambda.powf(days_between(last, now));
        Mastery {
            alpha: self.prior.0 + (self.alpha - self.prior.0) * factor,
            beta: self.prior.1 + (self.beta - self.prior.1) * factor,
            ..*self
        }
    }

    pub fn interval_days(&self, constants: &MasteryConstants) -> f32 {
        constants.base_interval_days
            * (1.0 + self.estimate()).powf(self.evidence() / constants.e_scale.max(f32::EPSILON))
    }

    /// Due at 1.0. Nothing attempted yet is not overdue — it is new.
    pub fn overdue(&self, now: DateTime<Utc>, constants: &MasteryConstants) -> f32 {
        let Some(last) = self.last_attempt_at else {
            return 0.0;
        };
        days_between(last, now) / self.interval_days(constants).max(f32::EPSILON)
    }
}

fn days_between(from: DateTime<Utc>, to: DateTime<Utc>) -> f32 {
    ((to - from).num_seconds().max(0) as f32) / 86_400.0
}

/// Spec §2's per-`(node, parameter_level)` state. There is no node-level
/// scalar: a node-wide figure is display only (§9.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MasteryKey {
    pub node: String,
    pub level: ParameterLevel,
}

/// What a reader needs in one call, so display, planner and tests cannot
/// disagree: the estimate and evidence as the decay reads them, and the
/// spacing as the stored counts read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reading {
    pub estimate: f32,
    pub evidence: f32,
    pub overdue: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct MasteryStore {
    entries: BTreeMap<MasteryKey, Mastery>,
    pub constants: MasteryConstants,
}

impl MasteryStore {
    /// Content's `(estimate, band)` seeds as priors (spec §2), one per rung the
    /// loop can actually run.
    pub fn seeded_from(content: &ContentIndex) -> Self {
        let mut entries = BTreeMap::new();
        for node in content.nodes.values() {
            let strength = band_strength(node.band.unwrap_or(Band::Low));
            for level in node
                .drills
                .iter()
                .filter_map(|drill| content.drill(drill))
                .filter_map(|drill| drill.level)
            {
                entries.insert(
                    MasteryKey {
                        node: node.id.clone(),
                        level,
                    },
                    Mastery::from_prior(strength * node.estimate, strength * (1.0 - node.estimate)),
                );
            }
        }
        Self {
            entries,
            constants: MasteryConstants::default(),
        }
    }

    pub fn get(&self, node: &str, level: ParameterLevel) -> Option<&Mastery> {
        self.entries.get(&MasteryKey {
            node: node.to_string(),
            level,
        })
    }

    pub fn record(
        &mut self,
        node: &str,
        level: ParameterLevel,
        verdict: Verdict,
        at: DateTime<Utc>,
    ) {
        let constants = self.constants;
        self.entries
            .entry(MasteryKey {
                node: node.to_string(),
                level,
            })
            .or_insert_with(|| Mastery::from_prior(1.0, 1.0))
            .record(verdict, at, &constants);
    }

    pub fn reading(&self, node: &str, level: ParameterLevel, now: DateTime<Utc>) -> Reading {
        let Some(mastery) = self.get(node, level) else {
            return Reading {
                estimate: 0.0,
                evidence: 0.0,
                overdue: 0.0,
            };
        };
        let decayed = mastery.decayed_at(now, &self.constants);
        Reading {
            estimate: decayed.estimate(),
            evidence: decayed.evidence(),
            overdue: mastery.overdue(now, &self.constants),
        }
    }

    /// A gate pass moves the cursor up a rung. The new level starts from a
    /// discounted inheritance of the one below, which is left untouched: a
    /// level-down is a cursor move, not a rewrite.
    pub fn level_up(&mut self, node: &str, from: ParameterLevel, to: ParameterLevel) {
        if self
            .get(node, to)
            .is_some_and(|above| above.evidence() > 0.0)
        {
            return;
        }
        let Some(below) = self.get(node, from) else {
            return;
        };
        let inherited =
            (self.constants.transfer * below.evidence()).min(self.constants.inherit_max);
        let estimate = below.estimate();
        self.entries.insert(
            MasteryKey {
                node: node.to_string(),
                level: to,
            },
            Mastery::from_prior(
                1.0 + inherited * estimate,
                1.0 + inherited * (1.0 - estimate),
            ),
        );
    }

    /// First rep of the day on returning material (spec §2), which is the
    /// highest-information tap-verdict there is.
    /// `utc_offset_minutes` is where the user is: midnight UTC is the wrong
    /// hour to turn the day over for anyone practising in the evening (#1221).
    pub fn is_cold(
        &self,
        node: &str,
        level: ParameterLevel,
        now: DateTime<Utc>,
        utc_offset_minutes: i32,
    ) -> bool {
        let local = local_offset(utc_offset_minutes);
        self.get(node, level)
            .and_then(|mastery| mastery.last_attempt_at)
            .is_some_and(|last| {
                last.with_timezone(&local).date_naive() < now.with_timezone(&local).date_naive()
            })
    }
}

/// Real offsets run from UTC-12 to UTC+14. `FixedOffset` would accept a great
/// deal more, and a date built from that is wrong rather than refused.
fn local_offset(minutes: i32) -> FixedOffset {
    FixedOffset::east_opt(minutes.clamp(-12 * 60, 14 * 60) * 60).expect("clamped to a real offset")
}

fn band_strength(band: Band) -> f32 {
    match band {
        Band::Low => 2.0,
        Band::Medium => 5.0,
        Band::High => 10.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::gate::ClickLevel;
    use chrono::{TimeDelta, TimeZone};

    fn at(day: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_754_300_000, 0).unwrap() + TimeDelta::days(day)
    }

    fn close(left: f32, right: f32) -> bool {
        (left - right).abs() < 0.01
    }

    // ── The three readings, defined once (spec §2) ──

    #[test]
    fn the_estimate_is_the_priors_until_something_is_attempted() {
        let mastery = Mastery::from_prior(1.4, 0.6);

        assert!(close(mastery.estimate(), 0.7), "{}", mastery.estimate());
        assert_eq!(mastery.evidence(), 0.0, "a prior is not evidence");
        assert_eq!(mastery.confidence(&MasteryConstants::default()), 0.0);
    }

    #[test]
    fn a_clean_attempt_moves_the_estimate_up_and_a_missed_one_down() {
        let constants = MasteryConstants::default();
        let mut clean = Mastery::from_prior(1.0, 1.0);
        let mut missed = clean;

        clean.record(Verdict::Clean, at(0), &constants);
        missed.record(Verdict::Missed, at(0), &constants);

        assert!(clean.estimate() > 0.5, "{}", clean.estimate());
        assert!(missed.estimate() < 0.5, "{}", missed.estimate());
        assert_eq!(clean.evidence(), 1.0);
        assert_eq!(clean.last_attempt_at, Some(at(0)));
    }

    #[test]
    fn confidence_grows_with_evidence_and_saturates_below_certainty() {
        let constants = MasteryConstants::default();
        let mut mastery = Mastery::from_prior(1.0, 1.0);
        for day in 0..8 {
            mastery.record(Verdict::Clean, at(day), &constants);
        }

        assert!(
            close(mastery.confidence(&constants), 0.5),
            "8 attempts against k = 8 is half confident: {}",
            mastery.confidence(&constants)
        );

        for day in 8..60 {
            mastery.record(Verdict::Clean, at(day), &constants);
        }
        assert!(
            mastery.confidence(&constants) < 0.85,
            "the evidence cap holds confidence below certainty: {}",
            mastery.confidence(&constants)
        );
    }

    // ── The evidence cap (spec §2) ──

    #[test]
    fn the_cap_scales_both_counts_down_and_leaves_the_estimate_exactly_where_it_was() {
        let constants = MasteryConstants::default();
        let mut mastery = Mastery::from_prior(1.0, 1.0);
        for day in 0..100 {
            mastery.record(
                if day % 4 == 0 {
                    Verdict::Missed
                } else {
                    Verdict::Clean
                },
                at(day),
                &constants,
            );
        }

        assert!(
            close(mastery.alpha + mastery.beta, constants.evidence_max),
            "counts held at the cap: {} + {}",
            mastery.alpha,
            mastery.beta
        );

        let before = mastery.estimate();
        let capped = mastery;
        mastery.record(Verdict::Clean, at(101), &constants);
        assert!(
            mastery.estimate() > before,
            "a capped node still changes its mind"
        );
        assert!(
            close(mastery.alpha + mastery.beta, constants.evidence_max),
            "and stays at the cap"
        );

        let mut scaled = capped;
        scaled.record(Verdict::Missed, at(102), &constants);
        assert!(
            scaled.estimate() < before,
            "in both directions, forever — which is what makes three bad sessions visible"
        );
    }

    #[test]
    fn scaling_down_preserves_the_estimate_of_the_attempt_that_triggered_it() {
        let constants = MasteryConstants {
            evidence_max: 4.0,
            ..MasteryConstants::default()
        };
        let mut mastery = Mastery::from_prior(2.0, 2.0);

        mastery.record(Verdict::Clean, at(0), &constants);

        assert!(
            close(mastery.estimate(), 3.0 / 5.0),
            "the ratio the fifth attempt produced, held after the scale-down: {}",
            mastery.estimate()
        );
        assert!(close(mastery.alpha + mastery.beta, 4.0));
    }

    // ── Decay is a read (spec §2) ──

    #[test]
    fn decay_pulls_the_counts_toward_the_prior_and_never_manufactures_failure() {
        let constants = MasteryConstants::default();
        let mut mastery = Mastery::from_prior(1.0, 1.0);
        for day in 0..10 {
            mastery.record(Verdict::Clean, at(day), &constants);
        }

        let decayed = mastery.decayed_at(at(9 + 60), &constants);

        assert!(
            decayed.evidence() < mastery.evidence(),
            "two half-lives shrink the evidence"
        );
        assert!(
            decayed.beta <= mastery.beta,
            "the failure count never grows with time: {} was {}",
            decayed.beta,
            mastery.beta
        );
        assert!(
            decayed.estimate() < mastery.estimate(),
            "and the estimate moves toward the prior, not toward failure"
        );
        assert!(decayed.estimate() > 0.5, "which is still above the prior");
    }

    #[test]
    fn decay_is_a_read_and_leaves_the_stored_counts_alone() {
        let constants = MasteryConstants::default();
        let mut mastery = Mastery::from_prior(1.0, 1.0);
        mastery.record(Verdict::Clean, at(0), &constants);
        let stored = mastery;

        let _ = mastery.decayed_at(at(400), &constants);

        assert_eq!(mastery, stored, "reading it must not rewrite it");
    }

    #[test]
    fn a_decayed_read_never_falls_past_the_prior() {
        let constants = MasteryConstants::default();
        let mut mastery = Mastery::from_prior(1.0, 1.0);
        for day in 0..20 {
            mastery.record(Verdict::Clean, at(day), &constants);
        }

        let forgotten = mastery.decayed_at(at(20_000), &constants);

        assert!(close(forgotten.alpha, 1.0), "{}", forgotten.alpha);
        assert!(close(forgotten.beta, 1.0), "{}", forgotten.beta);
        assert!(
            close(forgotten.evidence(), 0.0),
            "back to knowing nothing, not to knowing it fails"
        );
    }

    // ── Spacing, from the stored counts (spec §2's calibration table) ──

    #[test]
    fn brand_new_material_comes_back_in_two_days() {
        let mastery = Mastery::from_prior(0.6, 1.4);

        assert!(
            close(mastery.interval_days(&MasteryConstants::default()), 2.0),
            "{}",
            mastery.interval_days(&MasteryConstants::default())
        );
    }

    #[test]
    fn the_frontier_comes_back_in_about_three_days() {
        let interval = calibrated(0.3, 8).interval_days(&MasteryConstants::default());

        assert!(
            (2.5..3.5).contains(&interval),
            "estimate 0.3 on 8 attempts: {interval}"
        );
    }

    #[test]
    fn solid_material_comes_back_in_about_six_days() {
        let interval = calibrated(0.7, 16).interval_days(&MasteryConstants::default());

        assert!(
            (5.0..7.0).contains(&interval),
            "estimate 0.7 on 16 attempts: {interval}"
        );
    }

    #[test]
    fn a_node_at_the_evidence_cap_comes_back_in_about_six_weeks() {
        let interval = calibrated(0.9, 38).interval_days(&MasteryConstants::default());

        assert!(
            (35.0..49.0).contains(&interval),
            "estimate 0.9 at the cap: {interval}"
        );
    }

    #[test]
    fn overdue_reaches_one_when_the_interval_has_passed() {
        let constants = MasteryConstants::default();
        let mut mastery = calibrated(0.3, 8);
        mastery.last_attempt_at = Some(at(0));
        let interval = mastery.interval_days(&constants);

        assert!(close(mastery.overdue(at(0), &constants), 0.0));
        assert!(
            close(
                mastery.overdue(
                    at(0) + TimeDelta::seconds((interval * 86_400.0) as i64),
                    &constants
                ),
                1.0
            ),
            "due exactly at the interval"
        );
    }

    #[test]
    fn a_long_gap_is_due_rather_than_wildly_overdue() {
        let constants = MasteryConstants::default();
        let mut mastery = calibrated(0.3, 8);
        mastery.last_attempt_at = Some(at(0));

        let spacing_gap = mastery.overdue(at(30), &constants);
        let decayed_gap = mastery
            .decayed_at(at(30), &constants)
            .overdue(at(30), &constants);

        assert!(
            spacing_gap < decayed_gap,
            "spacing reads the stored counts, so elapsed time counts once, not twice: \
             {spacing_gap} against {decayed_gap}"
        );
    }

    #[test]
    fn material_never_attempted_is_new_rather_than_overdue() {
        let mastery = Mastery::from_prior(0.6, 1.4);

        assert_eq!(mastery.overdue(at(400), &MasteryConstants::default()), 0.0);
    }

    // ── The store: priors, level-ups and the cold test ──

    fn level(tempo_bpm: u16) -> ParameterLevel {
        ParameterLevel {
            tempo_bpm,
            click_level: ClickLevel::EveryBeat,
        }
    }

    #[test]
    fn the_seeds_become_priors_at_the_band_the_content_authored() {
        let store = MasteryStore::seeded_from(ContentIndex::shipped());
        let shells = store
            .get(
                "shells-ii-v-i",
                ParameterLevel {
                    tempo_bpm: 100,
                    click_level: ClickLevel::TwoAndFour,
                },
            )
            .expect("a prior at the rung shells-cycle runs");

        assert!(close(shells.estimate(), 0.7), "the seeded estimate");
        assert!(
            close(shells.alpha + shells.beta, 5.0),
            "a medium band is 5 pseudo-counts: {} + {}",
            shells.alpha,
            shells.beta
        );
        assert_eq!(shells.evidence(), 0.0, "a seed is a prior, not evidence");
    }

    #[test]
    fn a_low_band_seed_is_weaker_than_a_medium_one() {
        let store = MasteryStore::seeded_from(ContentIndex::shipped());
        let rootless = store
            .get(
                "rootless-a-b",
                ParameterLevel {
                    tempo_bpm: 60,
                    click_level: ClickLevel::EveryBeat,
                },
            )
            .expect("a prior at the rung rootless-one-key runs");

        assert!(
            close(rootless.alpha + rootless.beta, 2.0),
            "a low band is 2 pseudo-counts: {} + {}",
            rootless.alpha,
            rootless.beta
        );
    }

    #[test]
    fn a_stub_node_gets_no_prior_because_it_has_no_rung_to_hold_one() {
        let store = MasteryStore::seeded_from(ContentIndex::shipped());

        assert_eq!(store.get("diatonic-7ths", level(100)), None);
    }

    #[test]
    fn an_attempt_lands_on_the_level_it_was_played_at() {
        let mut store = MasteryStore::seeded_from(ContentIndex::shipped());
        store.record("rootless-a-b", level(60), Verdict::Clean, at(0));

        let played = store.get("rootless-a-b", level(60)).unwrap().evidence();
        assert!(close(played, 1.0), "{played}");
        assert_eq!(
            store.get("rootless-a-b", level(80)).map(Mastery::evidence),
            Some(0.0),
            "the rung above it heard nothing"
        );
    }

    #[test]
    fn an_attempt_on_an_unseeded_rung_starts_from_an_even_prior() {
        let mut store = MasteryStore::default();
        store.record("user-node", level(120), Verdict::Clean, at(0));

        let mastery = store.get("user-node", level(120)).expect("a new entry");
        assert_eq!(mastery.evidence(), 1.0);
        assert!(mastery.estimate() > 0.5);
    }

    #[test]
    fn the_reading_is_the_decayed_estimate_against_the_stored_spacing() {
        let mut store = MasteryStore::seeded_from(ContentIndex::shipped());
        for day in 0..8 {
            store.record("rootless-a-b", level(60), Verdict::Clean, at(day));
        }

        let fresh = store.reading("rootless-a-b", level(60), at(7));
        let stale = store.reading("rootless-a-b", level(60), at(7 + 60));

        assert!(stale.estimate < fresh.estimate, "decay moved the estimate");
        assert!(stale.evidence < fresh.evidence, "and shrank the evidence");
        assert!(
            close(
                stale.overdue,
                60.0 / store
                    .get("rootless-a-b", level(60))
                    .unwrap()
                    .interval_days(&store.constants)
            ),
            "but spacing came from the stored counts: {}",
            stale.overdue
        );
    }

    #[test]
    fn a_reading_of_material_the_store_has_never_seen_is_new_and_not_overdue() {
        let reading = MasteryStore::default().reading("nothing-here", level(90), at(400));

        assert_eq!(reading.evidence, 0.0);
        assert_eq!(reading.overdue, 0.0);
    }

    #[test]
    fn a_level_up_inherits_a_discounted_share_of_the_rung_below() {
        let constants = MasteryConstants::default();
        let mut store = MasteryStore::default();
        for day in 0..10 {
            store.record("rootless-a-b", level(60), Verdict::Clean, at(day));
        }
        let below = *store.get("rootless-a-b", level(60)).unwrap();

        store.level_up("rootless-a-b", level(60), level(80));

        let above = store.get("rootless-a-b", level(80)).expect("the new rung");
        assert_eq!(
            *store.get("rootless-a-b", level(60)).unwrap(),
            below,
            "the level below is left untouched: a level-down is a cursor move"
        );
        assert!(
            above.estimate() > 0.5,
            "the new rung starts optimistic, not reset: {}",
            above.estimate()
        );
        assert!(
            above.estimate() < below.estimate(),
            "but less certain than the rung below: {} against {}",
            above.estimate(),
            below.estimate()
        );
        assert_eq!(above.evidence(), 0.0, "inheritance is prior, not evidence");
        assert!(
            above.alpha + above.beta <= 2.0 + constants.inherit_max,
            "and it is capped: {} + {}",
            above.alpha,
            above.beta
        );
    }

    #[test]
    fn a_capped_rung_below_cannot_hand_down_a_prior_that_overrides_the_hands() {
        let constants = MasteryConstants::default();
        let mut store = MasteryStore::default();
        for day in 0..60 {
            store.record("rootless-a-b", level(60), Verdict::Clean, at(day));
        }

        store.level_up("rootless-a-b", level(60), level(80));
        let above = *store.get("rootless-a-b", level(80)).unwrap();

        assert!(
            close(above.alpha + above.beta, 2.0 + constants.inherit_max),
            "inherit_max holds it at 6 inherited counts, so a handful of \
             contrary attempts can still move it: {} + {}",
            above.alpha,
            above.beta
        );
    }

    #[test]
    fn a_level_up_onto_a_rung_that_already_has_evidence_leaves_it_alone() {
        let mut store = MasteryStore::default();
        store.record("rootless-a-b", level(60), Verdict::Clean, at(0));
        store.record("rootless-a-b", level(80), Verdict::Missed, at(1));
        let above = *store.get("rootless-a-b", level(80)).unwrap();

        store.level_up("rootless-a-b", level(60), level(80));

        assert_eq!(
            *store.get("rootless-a-b", level(80)).unwrap(),
            above,
            "what the hands already said at this tempo outranks an inheritance"
        );
    }

    #[test]
    fn the_first_rep_of_the_day_on_returning_material_is_cold() {
        let mut store = MasteryStore::default();
        store.record("rootless-a-b", level(60), Verdict::Clean, at(0));

        assert!(store.is_cold("rootless-a-b", level(60), at(1), 0));
        assert!(
            !store.is_cold("rootless-a-b", level(60), at(0) + TimeDelta::hours(2), 0),
            "a second block the same day is warm"
        );
    }

    /// In BST, 00:30 local is 23:30 the previous day in UTC, so a new day's
    /// first rep was recorded warm (#1221).
    #[test]
    fn the_day_boundary_is_the_users_midnight_not_utcs() {
        let mut store = MasteryStore::default();
        let bst = 60;
        // 22:00 on the 4th, where the user is.
        let last_night = Utc.with_ymd_and_hms(2026, 8, 4, 21, 0, 0).unwrap();
        store.record("rootless-a-b", level(60), Verdict::Clean, last_night);

        assert!(
            store.is_cold(
                "rootless-a-b",
                level(60),
                Utc.with_ymd_and_hms(2026, 8, 4, 23, 30, 0).unwrap(),
                bst
            ),
            "00:30 local is a new day where the user is"
        );
        assert!(
            !store.is_cold(
                "rootless-a-b",
                level(60),
                Utc.with_ymd_and_hms(2026, 8, 4, 22, 0, 0).unwrap(),
                bst
            ),
            "23:00 local is still the same evening"
        );
        assert!(
            !store.is_cold(
                "rootless-a-b",
                level(60),
                Utc.with_ymd_and_hms(2026, 8, 5, 3, 0, 0).unwrap(),
                -5 * 60
            ),
            "and in New York the pair straddle no boundary at all"
        );
    }

    /// UTC+13 puts midnight UTC at 1pm local, so an afternoon session reads as
    /// a new day from a morning one and two evening sessions read as one.
    #[test]
    fn a_far_eastern_offset_moves_the_boundary_the_whole_way() {
        let mut store = MasteryStore::default();
        let auckland = 13 * 60;
        // 09:00 on the 5th local, which is 20:00 on the 4th in UTC.
        let morning = Utc.with_ymd_and_hms(2026, 8, 4, 20, 0, 0).unwrap();
        store.record("rootless-a-b", level(60), Verdict::Clean, morning);

        assert!(
            !store.is_cold(
                "rootless-a-b",
                level(60),
                // 15:00 the same local day, which UTC calls the 5th.
                Utc.with_ymd_and_hms(2026, 8, 5, 2, 0, 0).unwrap(),
                auckland
            ),
            "the afternoon of the same local day is warm"
        );
        assert!(
            store.is_cold(
                "rootless-a-b",
                level(60),
                // 09:00 the next local day.
                Utc.with_ymd_and_hms(2026, 8, 5, 20, 0, 0).unwrap(),
                auckland
            ),
            "and the next local morning is cold"
        );
    }

    /// `FixedOffset` accepts anything under 24 hours, so without the clamp a
    /// nonsense offset builds a real date on the wrong day.
    #[test]
    fn an_impossible_offset_is_clamped_to_the_furthest_real_one() {
        let mut store = MasteryStore::default();
        store.record(
            "rootless-a-b",
            level(60),
            Verdict::Clean,
            Utc.with_ymd_and_hms(2026, 8, 4, 9, 0, 0).unwrap(),
        );

        // Clamped to +14:00 the day has turned over (23:00 then 00:00); at the
        // +20:00 asked for it has not (05:00 then 06:00).
        assert!(store.is_cold(
            "rootless-a-b",
            level(60),
            Utc.with_ymd_and_hms(2026, 8, 4, 10, 0, 0).unwrap(),
            20 * 60
        ));
    }

    #[test]
    fn material_with_no_history_is_new_rather_than_cold() {
        let store = MasteryStore::seeded_from(ContentIndex::shipped());

        assert!(
            !store.is_cold("rootless-a-b", level(60), at(400), 0),
            "a cold test is a test of what came back, and this has not been away"
        );
    }

    /// A state with the given estimate and evidence, built the way the spec's
    /// calibration table describes one rather than by recording attempts.
    fn calibrated(estimate: f32, evidence: u16) -> Mastery {
        let prior = (2.0 * estimate, 2.0 * (1.0 - estimate));
        let total = 2.0 + f32::from(evidence);
        Mastery {
            alpha: total * estimate,
            beta: total * (1.0 - estimate),
            prior,
            last_attempt_at: None,
        }
    }
}
