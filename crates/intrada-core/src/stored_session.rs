//! A practice session as the shells store it: one row of columns, the entries
//! as JSON (#2234). JSON, not bincode: bincode is positional, so a field change
//! would fail to decode old rows, and the device is the only copy.

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::item::{ItemKind, Modality};
use crate::domain::key::{key_from_stored, key_to_stored};
use crate::domain::metre::Metre;
use crate::domain::section::BarRange;
use crate::domain::session::{
    split_evenly, Away, ClickState, CompletionStatus, EntryStatus, Felt, FocusKind, IntentionFocus,
    IntentionMet, NotePoint, NotePointKind, NoteSpan, Obstacle, Play, PracticeSession, RepAction,
    RepEvent, Segment, SetlistEntry, TempoChange,
};

/// The `session` table's columns, as the shell reads and writes them.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredSession {
    pub id: String,
    pub started_at: String,
    pub completed_at: String,
    pub total_duration_secs: i64,
    pub completion_status: String,
    pub session_notes: Option<String>,
    pub entries: String,
    pub session_score: Option<i64>,
    pub capture_version: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionRead {
    pub session: PracticeSession,
    /// Each stored value the core could not read and replaced, for the shell
    /// to log: a silent default would hide a bad row (#949).
    pub unreadable: Vec<String>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum StoredSessionError {
    #[error("session {id}: {column} {raw:?} is not a time")]
    Time {
        id: String,
        column: &'static str,
        raw: String,
    },
    #[error("session {id}: entries did not encode: {reason}")]
    Encode { id: String, reason: String },
}

pub fn session_from_stored(row: &StoredSession) -> Result<SessionRead, StoredSessionError> {
    let time = |column: &'static str, raw: &str| {
        parse_time(raw).ok_or_else(|| StoredSessionError::Time {
            id: row.id.clone(),
            column,
            raw: raw.to_string(),
        })
    };
    let started_at = time("started_at", &row.started_at)?;
    let completed_at = time("completed_at", &row.completed_at)?;
    let mut reader = Reader {
        session_start: started_at,
        unreadable: Vec::new(),
    };
    let completion_status = reader.known(
        "CompletionStatus",
        &row.completion_status,
        completion_status,
        CompletionStatus::Completed,
    );
    let entries = reader.entries(&row.entries);
    let total_duration_secs = u64::try_from(row.total_duration_secs).unwrap_or_else(|_| {
        reader.note(format!(
            "total_duration_secs {} is negative",
            row.total_duration_secs
        ));
        0
    });
    let session = PracticeSession {
        id: row.id.clone(),
        entries,
        session_notes: row.session_notes.clone(),
        started_at,
        completed_at,
        total_duration_secs,
        completion_status,
        session_score: row.session_score.map(|s| clamp(s, u8::MAX)),
        capture_version: row.capture_version.map(|v| clamp(v, u32::MAX)),
    };
    Ok(SessionRead {
        session,
        unreadable: reader.unreadable,
    })
}

pub fn session_to_stored(session: &PracticeSession) -> Result<StoredSession, StoredSessionError> {
    let entries: Vec<StoredEntry> = session.entries.iter().map(stored_entry).collect();
    let entries = serde_json::to_string(&entries).map_err(|e| StoredSessionError::Encode {
        id: session.id.clone(),
        reason: e.to_string(),
    })?;
    Ok(StoredSession {
        id: session.id.clone(),
        started_at: time_text(&session.started_at),
        completed_at: time_text(&session.completed_at),
        total_duration_secs: i64::try_from(session.total_duration_secs).unwrap_or(i64::MAX),
        completion_status: completion_status_text(&session.completion_status).to_string(),
        session_notes: session.session_notes.clone(),
        entries,
        session_score: session.session_score.map(i64::from),
        capture_version: session.capture_version.map(i64::from),
    })
}

// ── Stored text for each enum ──

fn completion_status(raw: &str) -> Option<CompletionStatus> {
    match raw {
        "completed" => Some(CompletionStatus::Completed),
        "ended_early" => Some(CompletionStatus::EndedEarly),
        _ => None,
    }
}

fn completion_status_text(status: &CompletionStatus) -> &'static str {
    match status {
        CompletionStatus::Completed => "completed",
        CompletionStatus::EndedEarly => "ended_early",
    }
}

fn entry_status(raw: &str) -> Option<EntryStatus> {
    match raw {
        "completed" => Some(EntryStatus::Completed),
        "skipped" => Some(EntryStatus::Skipped),
        "not_attempted" => Some(EntryStatus::NotAttempted),
        _ => None,
    }
}

fn entry_status_text(status: &EntryStatus) -> &'static str {
    match status {
        EntryStatus::Completed => "completed",
        EntryStatus::Skipped => "skipped",
        EntryStatus::NotAttempted => "not_attempted",
    }
}

fn rep_action(raw: &str) -> Option<RepAction> {
    match raw {
        "missed" => Some(RepAction::Missed),
        "success" => Some(RepAction::Success),
        "undo" => Some(RepAction::Undo),
        _ => None,
    }
}

fn rep_action_text(action: RepAction) -> &'static str {
    match action {
        RepAction::Missed => "missed",
        RepAction::Success => "success",
        RepAction::Undo => "undo",
    }
}

fn item_kind(raw: &str) -> Option<ItemKind> {
    match raw {
        "piece" => Some(ItemKind::Piece),
        "exercise" => Some(ItemKind::Exercise),
        _ => None,
    }
}

fn item_kind_text(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::Piece => "piece",
        ItemKind::Exercise => "exercise",
    }
}

fn modality(raw: &str) -> Option<Modality> {
    match raw {
        "major" => Some(Modality::Major),
        "minor" => Some(Modality::Minor),
        _ => None,
    }
}

fn modality_text(modality: Modality) -> &'static str {
    match modality {
        Modality::Major => "major",
        Modality::Minor => "minor",
    }
}

fn focus_kind(raw: &str) -> Option<FocusKind> {
    match raw {
        "tempo" => Some(FocusKind::Tempo),
        "clean_reps" => Some(FocusKind::CleanReps),
        "from_memory" => Some(FocusKind::FromMemory),
        "evenness" => Some(FocusKind::Evenness),
        _ => None,
    }
}

fn focus_kind_text(kind: FocusKind) -> &'static str {
    match kind {
        FocusKind::Tempo => "tempo",
        FocusKind::CleanReps => "clean_reps",
        FocusKind::FromMemory => "from_memory",
        FocusKind::Evenness => "evenness",
    }
}

fn intention_met(raw: &str) -> Option<IntentionMet> {
    match raw {
        "yes" => Some(IntentionMet::Yes),
        "partly" => Some(IntentionMet::Partly),
        "not_yet" => Some(IntentionMet::NotYet),
        _ => None,
    }
}

fn intention_met_text(met: IntentionMet) -> &'static str {
    match met {
        IntentionMet::Yes => "yes",
        IntentionMet::Partly => "partly",
        IntentionMet::NotYet => "not_yet",
    }
}

fn felt(raw: &str) -> Option<Felt> {
    match raw {
        "easy" => Some(Felt::Easy),
        "effortful" => Some(Felt::Effortful),
        "tense" => Some(Felt::Tense),
        _ => None,
    }
}

fn felt_text(felt: Felt) -> &'static str {
    match felt {
        Felt::Easy => "easy",
        Felt::Effortful => "effortful",
        Felt::Tense => "tense",
    }
}

fn obstacle(raw: &str) -> Option<Obstacle> {
    match raw {
        "notes" => Some(Obstacle::Notes),
        "rhythm" => Some(Obstacle::Rhythm),
        "fingering" => Some(Obstacle::Fingering),
        "memory" => Some(Obstacle::Memory),
        "tone" => Some(Obstacle::Tone),
        "tension" => Some(Obstacle::Tension),
        _ => None,
    }
}

fn obstacle_text(obstacle: Obstacle) -> &'static str {
    match obstacle {
        Obstacle::Notes => "notes",
        Obstacle::Rhythm => "rhythm",
        Obstacle::Fingering => "fingering",
        Obstacle::Memory => "memory",
        Obstacle::Tone => "tone",
        Obstacle::Tension => "tension",
    }
}

fn parse_time(raw: &str) -> Option<DateTime<Utc>> {
    raw.parse().ok()
}

/// The text chrono's serde writes, which every row so far was stored with.
fn time_text(time: &DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

fn clamp<T>(value: i64, max: T) -> T
where
    T: TryFrom<i64> + Copy,
    i64: From<T>,
{
    T::try_from(value.clamp(0, i64::from(max))).unwrap_or(max)
}

// ── The entries column ──

/// The per-play fields below `plays` are LEGACY: every row written before
/// #1739 carries them at entry level and has no `plays`. They are read, never
/// written; nothing on device is rewritten (#1739 decision 8). A step a row
/// names, planned or played, is not read: steps were retired, not moved (#2246).
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredEntry {
    id: String,
    item_id: String,
    item_title: String,
    item_type: String,
    position: u64,
    duration_secs: u64,
    status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    intention: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    planned_duration_secs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    planned_variation_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    planned_rep_target: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    plays: Option<Vec<StoredPlay>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    segments: Option<Vec<StoredSegment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    focus: Option<StoredFocus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    intention_met: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    felt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    got_in_the_way: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note_points: Option<Vec<StoredNotePoint>>,

    /// The v0.16 plan, at most one section, before segments (#2315).
    #[serde(default, skip_serializing)]
    planned_section_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing)]
    score: Option<u8>,
    #[serde(default, skip_serializing)]
    rep_target: Option<u8>,
    #[serde(default, skip_serializing)]
    rep_count: Option<u8>,
    #[serde(default, skip_serializing)]
    rep_history: Option<Vec<StoredRepEvent>>,
    #[serde(default, skip_serializing)]
    achieved_tempo: Option<u16>,
    #[serde(default, skip_serializing)]
    click_pattern: Option<StoredClickState>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredPlay {
    id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    section_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    key: Option<StoredKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    variation_ids: Option<Vec<String>>,
    started_at: String,
    seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rep_target: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rep_count: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rep_history: Option<Vec<StoredRepEvent>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tempo_changes: Option<Vec<StoredTempoChange>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    achieved_tempo: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    click_pattern: Option<StoredClickState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    score: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    away: Option<Vec<StoredAway>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredSegment {
    section_id: String,
    planned_secs: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredFocus {
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    section_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target: Option<u16>,
}

/// `kind` is "bars" with `first` and `last`, "tempo" with `bpm`, or
/// "repetitions" with `count`, `clean` and `inARow`.
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct StoredNotePoint {
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    first: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bpm: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    count: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    clean: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    in_a_row: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    section_id: Option<String>,
    start: u32,
    end: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredAway {
    left_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    back_at: Option<String>,
    left_out: bool,
}

#[derive(Serialize, Deserialize)]
struct StoredKey {
    key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    modality: Option<String>,
}

/// Taps before the history was timestamped (#1367) are bare action strings.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum StoredRepEvent {
    Bare(String),
    Timed(StoredTimedRepEvent),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredTimedRepEvent {
    action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tempo: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    click_sounding: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredTempoChange {
    at: String,
    tempo: u16,
    click_sounding: bool,
}

#[derive(Serialize, Deserialize)]
struct StoredClickState {
    metre: StoredMetre,
    sounding: u16,
}

#[derive(Serialize, Deserialize)]
struct StoredMetre {
    beats: u8,
    unit: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    groups: Option<Vec<u8>>,
}

// ── Reading ──

struct Reader {
    session_start: DateTime<Utc>,
    unreadable: Vec<String>,
}

impl Reader {
    fn note(&mut self, what: String) {
        self.unreadable.push(what);
    }

    /// Conservative: an unknown value must not inflate stats (#949).
    fn known<T>(&mut self, kind: &str, raw: &str, read: fn(&str) -> Option<T>, fallback: T) -> T {
        read(raw).unwrap_or_else(|| {
            self.note(format!("unknown {kind} on decode: {raw:?}"));
            fallback
        })
    }

    fn time(&mut self, raw: &str) -> DateTime<Utc> {
        parse_time(raw).unwrap_or_else(|| {
            self.note(format!("time on decode: {raw:?}"));
            self.session_start
        })
    }

    fn entries(&mut self, json: &str) -> Vec<SetlistEntry> {
        match serde_json::from_str::<Vec<StoredEntry>>(json) {
            Ok(stored) => stored.iter().map(|e| self.entry(e)).collect(),
            Err(e) => {
                self.note(format!(
                    "entries failed to decode: {:?} at line {} column {}",
                    e.classify(),
                    e.line(),
                    e.column()
                ));
                Vec::new()
            }
        }
    }

    fn entry(&mut self, e: &StoredEntry) -> SetlistEntry {
        let status = self.known(
            "EntryStatus",
            &e.status,
            entry_status,
            EntryStatus::NotAttempted,
        );
        let plays = self.plays(e, &status);
        let segments = match &e.segments {
            Some(stored) => stored
                .iter()
                .map(|s| Segment {
                    section_id: s.section_id.clone(),
                    planned_secs: s.planned_secs,
                })
                .collect(),
            None => {
                let mut legacy: Vec<Segment> = e
                    .planned_section_ids
                    .iter()
                    .flatten()
                    .map(|id| Segment {
                        section_id: id.clone(),
                        planned_secs: 0,
                    })
                    .collect();
                split_evenly(&mut legacy, e.planned_duration_secs);
                legacy
            }
        };
        let focus = e.focus.as_ref().and_then(|f| {
            let kind = focus_kind(&f.kind).or_else(|| {
                self.note(format!("unknown FocusKind on decode: {:?}", f.kind));
                None
            })?;
            Some(IntentionFocus {
                kind,
                section_id: f.section_id.clone(),
                target: f.target,
            })
        });
        let intention_met = e
            .intention_met
            .as_deref()
            .and_then(|raw| self.known("IntentionMet", raw, |r| intention_met(r).map(Some), None));
        let felt = e
            .felt
            .as_deref()
            .and_then(|raw| self.known("Felt", raw, |r| felt(r).map(Some), None));
        let got_in_the_way = e
            .got_in_the_way
            .iter()
            .flatten()
            .filter_map(|raw| self.known("Obstacle", raw, |r| obstacle(r).map(Some), None))
            .collect();
        let note_points = e
            .note_points
            .iter()
            .flatten()
            .filter_map(|p| self.note_point(p))
            .collect();
        SetlistEntry {
            id: e.id.clone(),
            item_id: e.item_id.clone(),
            item_title: e.item_title.clone(),
            item_type: self.known("ItemKind", &e.item_type, item_kind, ItemKind::Piece),
            position: usize::try_from(e.position).unwrap_or(usize::MAX),
            duration_secs: e.duration_secs,
            status,
            notes: e.notes.clone(),
            intention: e.intention.clone(),
            planned_duration_secs: e.planned_duration_secs,
            group_id: e.group_id.clone(),
            planned_variation_ids: e.planned_variation_ids.clone().unwrap_or_default(),
            planned_rep_target: e.planned_rep_target.or(e.rep_target),
            plays,
            segments,
            focus,
            intention_met,
            felt,
            got_in_the_way,
            note_points,
        }
    }

    fn note_point(&mut self, p: &StoredNotePoint) -> Option<NotePoint> {
        let kind = match (p.kind.as_str(), p.first, p.last, p.bpm, p.count) {
            ("bars", Some(first), Some(last), _, _) => {
                NotePointKind::Bars(BarRange { first, last })
            }
            ("tempo", _, _, Some(bpm), _) => NotePointKind::Tempo { bpm },
            ("repetitions", _, _, _, Some(count)) => NotePointKind::Repetitions {
                count,
                clean: p.clean.unwrap_or(false),
                in_a_row: p.in_a_row.unwrap_or(false),
            },
            _ => {
                self.note(format!("unreadable note point on decode: {:?}", p.kind));
                return None;
            }
        };
        Some(NotePoint {
            kind,
            section_id: p.section_id.clone(),
            span: NoteSpan {
                start: p.start,
                end: p.end,
            },
        })
    }

    /// A row written before #1739 has no `plays` and folds into a single play
    /// so the record survives. An entry skipped or never reached keeps none.
    fn plays(&mut self, e: &StoredEntry, status: &EntryStatus) -> Vec<Play> {
        if let Some(stored) = e.plays.as_ref().filter(|p| !p.is_empty()) {
            return stored.iter().map(|p| self.play(p)).collect();
        }
        // A mark or a banked repetition is a record of practice whatever the
        // status says: rows before #1739 froze rep state on a skip.
        let recorded = e.score.is_some() || e.rep_count.unwrap_or(0) > 0;
        if *status != EntryStatus::Completed && !recorded {
            return Vec::new();
        }
        vec![Play {
            id: format!("{}-play", e.id),
            section_id: None,
            key: None,
            variation_ids: Vec::new(),
            started_at: self.session_start,
            seconds: e.duration_secs,
            rep_target: e.rep_target,
            rep_count: e.rep_count,
            rep_history: self.rep_history(e.rep_history.as_deref()),
            tempo_changes: Vec::new(),
            achieved_tempo: e.achieved_tempo,
            click_pattern: e.click_pattern.as_ref().map(click_state),
            score: e.score,
            away: Vec::new(),
        }]
    }

    fn play(&mut self, p: &StoredPlay) -> Play {
        let key = p.key.as_ref().and_then(|k| {
            let mode = k
                .modality
                .as_deref()
                .and_then(|m| self.known("Modality", m, |raw| modality(raw).map(Some), None));
            key_from_stored(Some(&k.key), mode)
        });
        Play {
            id: p.id.clone(),
            section_id: p.section_id.clone(),
            key,
            variation_ids: p.variation_ids.clone().unwrap_or_default(),
            started_at: self.time(&p.started_at),
            seconds: p.seconds,
            rep_target: p.rep_target,
            rep_count: p.rep_count,
            rep_history: self.rep_history(p.rep_history.as_deref()),
            tempo_changes: p
                .tempo_changes
                .iter()
                .flatten()
                .map(|t| TempoChange {
                    at: self.time(&t.at),
                    tempo: t.tempo,
                    click_sounding: t.click_sounding,
                })
                .collect(),
            achieved_tempo: p.achieved_tempo,
            click_pattern: p.click_pattern.as_ref().map(click_state),
            score: p.score,
            away: p
                .away
                .iter()
                .flatten()
                .map(|a| Away {
                    left_at: self.time(&a.left_at),
                    back_at: a.back_at.as_ref().map(|at| self.time(at)),
                    left_out: a.left_out,
                })
                .collect(),
        }
    }

    /// An untimed tap takes the session start: dated roughly, never dropped.
    fn rep_history(&mut self, stored: Option<&[StoredRepEvent]>) -> Option<Vec<RepEvent>> {
        stored.map(|events| {
            events
                .iter()
                .map(|event| match event {
                    StoredRepEvent::Bare(action) => RepEvent {
                        action: self.known("RepAction", action, rep_action, RepAction::Missed),
                        at: self.session_start,
                        tempo: None,
                        click_sounding: None,
                    },
                    StoredRepEvent::Timed(t) => RepEvent {
                        action: self.known("RepAction", &t.action, rep_action, RepAction::Missed),
                        at: match &t.at {
                            Some(at) => self.time(at),
                            None => self.session_start,
                        },
                        tempo: t.tempo,
                        click_sounding: t.click_sounding,
                    },
                })
                .collect()
        })
    }
}

fn click_state(stored: &StoredClickState) -> ClickState {
    ClickState {
        metre: Metre {
            beats: stored.metre.beats,
            unit: stored.metre.unit,
            groups: stored.metre.groups.clone(),
        },
        sounding: stored.sounding,
    }
}

// ── Writing ──

fn stored_entry(e: &SetlistEntry) -> StoredEntry {
    StoredEntry {
        id: e.id.clone(),
        item_id: e.item_id.clone(),
        item_title: e.item_title.clone(),
        item_type: item_kind_text(&e.item_type).to_string(),
        position: e.position as u64,
        duration_secs: e.duration_secs,
        status: entry_status_text(&e.status).to_string(),
        notes: e.notes.clone(),
        intention: e.intention.clone(),
        planned_duration_secs: e.planned_duration_secs,
        group_id: e.group_id.clone(),
        planned_variation_ids: Some(e.planned_variation_ids.clone()),
        planned_rep_target: e.planned_rep_target,
        plays: Some(e.plays.iter().map(stored_play).collect()),
        segments: Some(
            e.segments
                .iter()
                .map(|s| StoredSegment {
                    section_id: s.section_id.clone(),
                    planned_secs: s.planned_secs,
                })
                .collect(),
        ),
        focus: e.focus.as_ref().map(|f| StoredFocus {
            kind: focus_kind_text(f.kind).to_string(),
            section_id: f.section_id.clone(),
            target: f.target,
        }),
        intention_met: e.intention_met.map(|m| intention_met_text(m).to_string()),
        felt: e.felt.map(|f| felt_text(f).to_string()),
        got_in_the_way: Some(
            e.got_in_the_way
                .iter()
                .map(|o| obstacle_text(*o).to_string())
                .collect(),
        ),
        note_points: Some(e.note_points.iter().map(stored_note_point).collect()),
        planned_section_ids: None,
        score: None,
        rep_target: None,
        rep_count: None,
        rep_history: None,
        achieved_tempo: None,
        click_pattern: None,
    }
}

fn stored_play(p: &Play) -> StoredPlay {
    StoredPlay {
        id: p.id.clone(),
        section_id: p.section_id.clone(),
        key: p.key.as_ref().map(|key| {
            let (text, mode) = key_to_stored(key);
            StoredKey {
                key: text,
                modality: mode.map(|m| modality_text(m).to_string()),
            }
        }),
        variation_ids: Some(p.variation_ids.clone()),
        started_at: time_text(&p.started_at),
        seconds: p.seconds,
        rep_target: p.rep_target,
        rep_count: p.rep_count,
        rep_history: p.rep_history.as_ref().map(|events| {
            events
                .iter()
                .map(|r| {
                    StoredRepEvent::Timed(StoredTimedRepEvent {
                        action: rep_action_text(r.action).to_string(),
                        at: Some(time_text(&r.at)),
                        tempo: r.tempo,
                        click_sounding: r.click_sounding,
                    })
                })
                .collect()
        }),
        tempo_changes: Some(
            p.tempo_changes
                .iter()
                .map(|t| StoredTempoChange {
                    at: time_text(&t.at),
                    tempo: t.tempo,
                    click_sounding: t.click_sounding,
                })
                .collect(),
        ),
        achieved_tempo: p.achieved_tempo,
        click_pattern: p.click_pattern.as_ref().map(|c| StoredClickState {
            metre: StoredMetre {
                beats: c.metre.beats,
                unit: c.metre.unit,
                groups: c.metre.groups.clone(),
            },
            sounding: c.sounding,
        }),
        score: p.score,
        away: Some(
            p.away
                .iter()
                .map(|a| StoredAway {
                    left_at: time_text(&a.left_at),
                    back_at: a.back_at.as_ref().map(time_text),
                    left_out: a.left_out,
                })
                .collect(),
        ),
    }
}

fn stored_note_point(p: &NotePoint) -> StoredNotePoint {
    let base = StoredNotePoint {
        section_id: p.section_id.clone(),
        start: p.span.start,
        end: p.span.end,
        ..StoredNotePoint::default()
    };
    match p.kind {
        NotePointKind::Bars(bars) => StoredNotePoint {
            kind: "bars".to_string(),
            first: Some(bars.first),
            last: Some(bars.last),
            ..base
        },
        NotePointKind::Tempo { bpm } => StoredNotePoint {
            kind: "tempo".to_string(),
            bpm: Some(bpm),
            ..base
        },
        NotePointKind::Repetitions {
            count,
            clean,
            in_a_row,
        } => StoredNotePoint {
            kind: "repetitions".to_string(),
            count: Some(count),
            clean: Some(clean),
            in_a_row: Some(in_a_row),
            ..base
        },
    }
}

#[cfg(test)]
mod tests;
