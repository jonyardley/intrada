use crate::domain::item::ItemKind;
use crate::domain::key::Key;
use crate::domain::profile::{suggest_icon, Profile};
use crate::domain::section::{BarRange, BarsInput, SectionDraft, SectionEdit};
use crate::domain::session::{PlayWay, SetlistEntry};
use crate::domain::types::{CreateItem, Tempo, TempoInput, UpdateItem};
use crate::domain::variation::Variation;
use crate::error::LibraryError;
use crate::model::Model;

/// Validation limits shared across shells (web, CLI).
pub const MAX_TITLE: usize = 500;
pub const MAX_COMPOSER: usize = 200;
pub const MAX_NOTES: usize = 5000;
pub const MAX_INTENTION: usize = 500;
pub const MAX_TAG: usize = 100;
pub const MAX_TEMPO_MARKING: usize = 100;
pub const MIN_BPM: u16 = 1;
pub const MAX_BPM: u16 = 400;
pub const MIN_SCORE: u8 = 1;
pub const MAX_SCORE: u8 = 10;
// Equal to MAX on purpose: the stepper exists to ask for fewer passes, never more.
pub const DEFAULT_REP_TARGET: u8 = 10;
pub const MIN_REP_TARGET: u8 = 3;
pub const MAX_REP_TARGET: u8 = 10;
pub const MAX_REP_HISTORY: usize = 500;
pub const MAX_VARIATION_LABEL: usize = 100;
/// Per item, and per play.
pub const MAX_VARIATIONS: usize = 24;
pub const MAX_KEYS: usize = 24;
pub const MAX_SECTION_NAME: usize = 100;
pub const MAX_BAR: u16 = 9999;
pub const MAX_PLAYS_PER_ENTRY: usize = 24;
/// Under this, a play with no mark and no repetitions is a stray tap on the
/// picker rather than practice, and the terminal transition drops it (#1739).
pub const MIN_PLAY_SECONDS: u64 = 5;
pub const MIN_PLANNED_DURATION_SECS: u32 = 60;
pub const MAX_PLANNED_DURATION_SECS: u32 = 3600;
pub const DEFAULT_PLANNED_DURATION_SECS: u32 = 360;
pub const MIN_SESSION_LENGTH_MINS: u16 = 10;
pub const MAX_SESSION_LENGTH_MINS: u16 = 120;
pub const SESSION_LENGTH_STEP_MINS: u16 = 5;
/// Where the stepper starts when a length is switched on, never a length
/// applied on its own: no length is `None` (`specs/session-length.md`).
pub const DEFAULT_SESSION_LENGTH_MINS: u16 = 30;
pub const MIN_ACHIEVED_TEMPO: u16 = 1;
pub const MAX_ACHIEVED_TEMPO: u16 = 500;
pub const MIN_METRE_BEATS: u8 = 2;
pub const MAX_METRE_BEATS: u8 = 12;
pub const METRE_UNITS: [u8; 3] = [2, 4, 8];
pub const MAX_PROFILE_NAME: usize = 100;
pub const MAX_INSTRUMENT: usize = 100;

// ── Normalisation ──
// Trim free-text on input and collapse a now-blank value to absent, so a
// whitespace-only field behaves exactly like an empty one through validation
// and storage (#883). Runs before validate_*, so "   " fails "required" just
// like "" rather than persisting as stored whitespace.

fn trimmed_nonempty(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// One tag per spelling whatever its case, when stored and when listed (#898).
pub(crate) fn distinct_ignoring_case<S: AsRef<str>>(
    values: impl IntoIterator<Item = S>,
) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    values
        .into_iter()
        .map(|v| v.as_ref().trim().to_string())
        .filter(|v| !v.is_empty())
        .filter(|v| seen.insert(v.to_lowercase()))
        .collect()
}

pub fn normalize_create_item(mut input: CreateItem) -> CreateItem {
    input.title = input.title.trim().to_string();
    input.composer = trimmed_nonempty(input.composer);
    input.notes = trimmed_nonempty(input.notes);
    input.tags = distinct_ignoring_case(input.tags);
    input.variation_labels = distinct_ignoring_case(input.variation_labels);
    input
}

pub fn normalize_update_item(mut input: UpdateItem) -> UpdateItem {
    input.title = input.title.map(|t| t.trim().to_string());
    input.composer = input.composer.map(trimmed_nonempty);
    input.notes = input.notes.map(trimmed_nonempty);
    input.tags = input.tags.map(distinct_ignoring_case);
    input
}

/// Every cap is in characters, as the messages say; `str::len` is bytes (#1944).
pub(crate) fn exceeds_chars(value: &str, max: usize) -> bool {
    value.chars().count() > max
}

pub fn validate_title(title: &str) -> Result<(), LibraryError> {
    if title.is_empty() || exceeds_chars(title, MAX_TITLE) {
        return Err(LibraryError::Validation {
            field: "title".to_string(),
            message: format!("Title must be between 1 and {MAX_TITLE} characters"),
        });
    }
    Ok(())
}

/// Returns the tempo the form's text reads as, the one field parsed here.
pub fn validate_create_item(input: &CreateItem) -> Result<Option<Tempo>, LibraryError> {
    validate_title(&input.title)?;
    if let Some(photo_id) = input.photo_id.as_deref() {
        validate_photo_id(photo_id)?;
    }
    // Composer is optional for both pieces and exercises; when given, it must
    // be a sensible length.
    if let Some(ref composer) = input.composer {
        if composer.is_empty() || exceeds_chars(composer, MAX_COMPOSER) {
            return Err(LibraryError::Validation {
                field: "composer".to_string(),
                message: format!("Composer must be between 1 and {MAX_COMPOSER} characters"),
            });
        }
    }
    if let Some(ref notes) = input.notes {
        if exceeds_chars(notes, MAX_NOTES) {
            return Err(LibraryError::Validation {
                field: "notes".to_string(),
                message: format!("Notes must not exceed {MAX_NOTES} characters"),
            });
        }
    }
    validate_tags(&input.tags)?;
    let tempo = match input.tempo {
        Some(ref tempo) => parse_tempo(tempo)?,
        None => None,
    };
    validate_variation_labels(&input.variation_labels)?;
    Ok(tempo)
}

/// `ItemEvent::Add` is the only event that honours
/// `CreateItem.variation_labels` (#1783): `AddLinkedExercise` and
/// `AddPieceInFull` create items too, and silently dropping a caller's labels
/// there would be the #846 shape, a field that validates but never lands.
/// Both reject instead.
pub fn validate_no_variation_labels(input: &CreateItem) -> Result<(), LibraryError> {
    if input.variation_labels.is_empty() {
        return Ok(());
    }
    Err(LibraryError::Validation {
        field: "variation_labels".to_string(),
        message: "Variations can only be added when creating an item directly".to_string(),
    })
}

/// Returns the tempo to set, `None` when the update leaves it alone.
pub fn validate_update_item(input: &UpdateItem) -> Result<Option<Option<Tempo>>, LibraryError> {
    if let Some(ref title) = input.title {
        validate_title(title)?;
    }
    if let Some(Some(ref composer)) = input.composer {
        if composer.is_empty() || exceeds_chars(composer, MAX_COMPOSER) {
            return Err(LibraryError::Validation {
                field: "composer".to_string(),
                message: format!("Composer must be between 1 and {MAX_COMPOSER} characters"),
            });
        }
    }
    if let Some(Some(ref notes)) = input.notes {
        if exceeds_chars(notes, MAX_NOTES) {
            return Err(LibraryError::Validation {
                field: "notes".to_string(),
                message: format!("Notes must not exceed {MAX_NOTES} characters"),
            });
        }
    }
    if let Some(ref tags) = input.tags {
        validate_tags(tags)?;
    }
    input.tempo.as_ref().map(parse_tempo).transpose()
}

pub fn validate_session_notes(notes: &Option<String>) -> Result<(), LibraryError> {
    if let Some(ref n) = notes {
        if exceeds_chars(n, MAX_NOTES) {
            return Err(LibraryError::Validation {
                field: "session_notes".to_string(),
                message: format!("Practice notes must not exceed {MAX_NOTES} characters"),
            });
        }
    }
    Ok(())
}

pub fn validate_entry_notes(notes: &Option<String>) -> Result<(), LibraryError> {
    if let Some(ref n) = notes {
        if exceeds_chars(n, MAX_NOTES) {
            return Err(LibraryError::Validation {
                field: "notes".to_string(),
                message: format!("Notes must not exceed {MAX_NOTES} characters"),
            });
        }
    }
    Ok(())
}

pub fn validate_entries_not_empty<T>(entries: &[T], context: &str) -> Result<(), LibraryError> {
    if entries.is_empty() {
        return Err(LibraryError::Validation {
            field: "entries".to_string(),
            message: format!("{context} must have at least one entry"),
        });
    }
    Ok(())
}

pub fn validate_tags(tags: &[String]) -> Result<(), LibraryError> {
    for tag in tags {
        if tag.is_empty() || exceeds_chars(tag, MAX_TAG) {
            return Err(LibraryError::Validation {
                field: "tags".to_string(),
                message: format!("Each tag must be between 1 and {MAX_TAG} characters"),
            });
        }
    }
    Ok(())
}

pub fn validate_intention(intention: &Option<String>) -> Result<(), LibraryError> {
    if let Some(ref text) = intention {
        if exceeds_chars(text, MAX_INTENTION) {
            return Err(LibraryError::Validation {
                field: "intention".to_string(),
                message: format!("Intention must not exceed {MAX_INTENTION} characters"),
            });
        }
    }
    Ok(())
}

pub fn validate_score(score: u8) -> Result<(), LibraryError> {
    if !(MIN_SCORE..=MAX_SCORE).contains(&score) {
        return Err(LibraryError::Validation {
            field: "score".to_string(),
            message: format!("Mark must be between {MIN_SCORE} and {MAX_SCORE}"),
        });
    }
    Ok(())
}

pub fn validate_rep_target(rep_target: &Option<u8>) -> Result<(), LibraryError> {
    if let Some(t) = rep_target {
        if !(MIN_REP_TARGET..=MAX_REP_TARGET).contains(t) {
            return Err(LibraryError::Validation {
                field: "rep_target".to_string(),
                message: format!(
                    "Rep target must be between {MIN_REP_TARGET} and {MAX_REP_TARGET}"
                ),
            });
        }
    }
    Ok(())
}

pub fn validate_planned_duration(planned_duration_secs: &Option<u32>) -> Result<(), LibraryError> {
    if let Some(d) = planned_duration_secs {
        if !(MIN_PLANNED_DURATION_SECS..=MAX_PLANNED_DURATION_SECS).contains(d) {
            return Err(LibraryError::Validation {
                field: "planned_duration_secs".to_string(),
                message: format!(
                    "Planned duration must be between {MIN_PLANNED_DURATION_SECS} and {MAX_PLANNED_DURATION_SECS} seconds"
                ),
            });
        }
    }
    Ok(())
}

pub fn validate_session_length(length_mins: &Option<u16>) -> Result<(), LibraryError> {
    if let Some(m) = length_mins {
        if !(MIN_SESSION_LENGTH_MINS..=MAX_SESSION_LENGTH_MINS).contains(m)
            || m % SESSION_LENGTH_STEP_MINS != 0
        {
            return Err(LibraryError::Validation {
                field: "session_length_mins".to_string(),
                message: format!(
                    "Session length must be between {MIN_SESSION_LENGTH_MINS} and {MAX_SESSION_LENGTH_MINS} minutes, in steps of {SESSION_LENGTH_STEP_MINS}"
                ),
            });
        }
    }
    Ok(())
}

/// A stored length the range or step has since moved past, brought back
/// onto it rather than thrown away.
pub fn clamp_session_length(length_mins: u16) -> u16 {
    let step = SESSION_LENGTH_STEP_MINS;
    let rounded = length_mins.saturating_add(step / 2) / step * step;
    rounded.clamp(MIN_SESSION_LENGTH_MINS, MAX_SESSION_LENGTH_MINS)
}

pub fn validate_achieved_tempo(tempo: &Option<u16>) -> Result<(), LibraryError> {
    if let Some(t) = tempo {
        if !(MIN_ACHIEVED_TEMPO..=MAX_ACHIEVED_TEMPO).contains(t) {
            return Err(LibraryError::Validation {
                field: "achieved_tempo".to_string(),
                message: format!(
                    "Achieved tempo must be between {MIN_ACHIEVED_TEMPO} and {MAX_ACHIEVED_TEMPO} BPM"
                ),
            });
        }
    }
    Ok(())
}

/// Blank is no BPM; anything else must be a whole number in range (#2224).
pub fn parse_bpm(text: &str) -> Result<Option<u16>, LibraryError> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    match text.parse::<u16>() {
        Ok(bpm)
            if text.bytes().all(|b| b.is_ascii_digit()) && (MIN_BPM..=MAX_BPM).contains(&bpm) =>
        {
            Ok(Some(bpm))
        }
        _ => Err(LibraryError::Validation {
            field: "tempo".to_string(),
            message: format!("BPM must be a whole number between {MIN_BPM} and {MAX_BPM}"),
        }),
    }
}

/// Both parts blank is no tempo, which on an update clears it.
pub fn parse_tempo(input: &TempoInput) -> Result<Option<Tempo>, LibraryError> {
    let bpm = parse_bpm(input.bpm.as_deref().unwrap_or_default())?;
    let Some(tempo) = Tempo::from_parts(trimmed_nonempty(input.marking.clone()), bpm) else {
        return Ok(None);
    };
    validate_tempo(&tempo)?;
    Ok(Some(tempo))
}

pub fn validate_tempo(tempo: &Tempo) -> Result<(), LibraryError> {
    if tempo.marking.is_none() && tempo.bpm.is_none() {
        return Err(LibraryError::Validation {
            field: "tempo".to_string(),
            message: "Tempo must have at least a marking or BPM value".to_string(),
        });
    }
    if let Some(ref marking) = tempo.marking {
        if exceeds_chars(marking, MAX_TEMPO_MARKING) {
            return Err(LibraryError::Validation {
                field: "tempo".to_string(),
                message: format!("Tempo marking must not exceed {MAX_TEMPO_MARKING} characters"),
            });
        }
    }
    if let Some(bpm) = tempo.bpm {
        if !(MIN_BPM..=MAX_BPM).contains(&bpm) {
            return Err(LibraryError::Validation {
                field: "tempo".to_string(),
                message: format!("BPM must be between {MIN_BPM} and {MAX_BPM}"),
            });
        }
    }
    Ok(())
}

/// A photo id becomes a path component in the shell, so a value that is not a
/// ulid is a traversal out of the app container (`specs/piece-from-photo.md`).
/// The core is the only layer that can refuse it before it is stored.
pub fn validate_photo_id(photo_id: &str) -> Result<(), LibraryError> {
    if ulid::Ulid::from_string(photo_id).is_err() {
        return Err(LibraryError::Validation {
            field: "photo_id".to_string(),
            message: "That photo could not be saved".to_string(),
        });
    }
    Ok(())
}

pub fn validate_metre(metre: &crate::domain::Metre) -> Result<(), LibraryError> {
    let invalid = |message: String| LibraryError::Validation {
        field: "metre".to_string(),
        message,
    };
    if !(MIN_METRE_BEATS..=MAX_METRE_BEATS).contains(&metre.beats) {
        return Err(invalid(format!(
            "Beats in the bar must be between {MIN_METRE_BEATS} and {MAX_METRE_BEATS}"
        )));
    }
    if !METRE_UNITS.contains(&metre.unit) {
        return Err(invalid(
            "The beat must be a minim, crotchet or quaver".to_string(),
        ));
    }
    if let Some(groups) = &metre.groups {
        let sum: u32 = groups.iter().map(|&g| u32::from(g)).sum();
        if groups.is_empty() || groups.contains(&0) || sum != u32::from(metre.beats) {
            return Err(invalid(
                "The grouping must add up to the beats in the bar".to_string(),
            ));
        }
    }
    Ok(())
}

/// A click state is a valid metre plus at least one sounding beat inside it.
pub fn validate_click_state(
    state: &crate::domain::session::ClickState,
) -> Result<(), LibraryError> {
    validate_metre(&state.metre)?;
    let in_bar: u16 = (1u16 << state.metre.beats) - 1;
    if state.sounding == 0 || state.sounding & !in_bar != 0 {
        return Err(LibraryError::Validation {
            field: "click".to_string(),
            message: "The click must sound on at least one beat of the bar".to_string(),
        });
    }
    Ok(())
}

/// The id names an item that exists and is a piece.
pub fn validate_piece_host(piece_id: &str, model: &Model) -> Result<(), LibraryError> {
    let piece = model
        .items
        .iter()
        .find(|i| i.id == piece_id)
        .ok_or_else(|| LibraryError::NotFound {
            id: piece_id.to_string(),
        })?;

    if piece.kind != ItemKind::Piece {
        return Err(LibraryError::Validation {
            field: "piece_id".to_string(),
            message: "Target must be a piece, not an exercise".to_string(),
        });
    }

    Ok(())
}

/// The id names an item that exists and is an exercise.
pub fn validate_exercise_link_target(exercise_id: &str, model: &Model) -> Result<(), LibraryError> {
    let exercise = model
        .items
        .iter()
        .find(|i| i.id == exercise_id)
        .ok_or_else(|| LibraryError::NotFound {
            id: exercise_id.to_string(),
        })?;

    if exercise.kind != ItemKind::Exercise {
        return Err(LibraryError::Validation {
            field: "exercise_id".to_string(),
            message: "Linked item must be an exercise, not a piece".to_string(),
        });
    }

    Ok(())
}

/// A link names a live section of the piece it sits on, or none (the whole
/// piece). A removed section is refused: its links went with it (#2248).
pub fn validate_link_section(
    piece: &crate::domain::item::Item,
    section_id: Option<&str>,
) -> Result<(), LibraryError> {
    match section_id {
        Some(id) if !piece.is_live_section(id) => Err(LibraryError::Validation {
            field: "section_id".to_string(),
            message: "That section is not part of this piece".to_string(),
        }),
        _ => Ok(()),
    }
}

/// A chord chart hangs off a piece. The host must exist and be a `Piece` —
/// exercises don't carry changes.
pub fn validate_chart_host(piece_id: &str, model: &Model) -> Result<(), LibraryError> {
    let piece = model
        .items
        .iter()
        .find(|i| i.id == piece_id)
        .ok_or_else(|| LibraryError::NotFound {
            id: piece_id.to_string(),
        })?;

    if piece.kind != ItemKind::Piece {
        return Err(LibraryError::Validation {
            field: "piece_id".to_string(),
            message: "Only a piece can have a chord chart".to_string(),
        });
    }

    Ok(())
}

fn sections_error(message: impl Into<String>) -> LibraryError {
    LibraryError::Validation {
        field: "sections".to_string(),
        message: message.into(),
    }
}

/// One range of bars as a musician types or dictates it: "1-16", "1 to 16",
/// "bars 5 to 12", "bar 12", "bb. 5-12", "mm. 5-12" or a bare "12". En and em
/// dashes and the minus sign read as a hyphen, since iOS Smart Dashes and
/// pasted text bring them in. Reading bars out of a sentence is #2307's.
pub(crate) fn parse_bar_range(raw: &str) -> Result<Option<BarRange>, LibraryError> {
    let lowered = raw.trim().to_lowercase();
    if lowered.is_empty() {
        return Ok(None);
    }
    let rest = ["bars", "bar", "bb.", "mm."]
        .iter()
        .find_map(|prefix| lowered.strip_prefix(prefix))
        .unwrap_or(&lowered)
        .replace(['\u{2013}', '\u{2014}', '\u{2212}'], "-");
    let words: Vec<&str> = rest.split_whitespace().collect();
    let (first, last) = match (rest.split_once('-'), words.as_slice()) {
        (Some((first, last)), _) => (first.trim(), last.trim()),
        (None, [first, "to", last]) => (*first, *last),
        (None, [only]) => (*only, *only),
        _ => return Err(unreadable_bars()),
    };
    validate_bar_range(parse_bar(first)?, parse_bar(last)?).map(Some)
}

fn unreadable_bars() -> LibraryError {
    sections_error("Write bars as 1-16, 1 to 16 or bar 12")
}

fn parse_bar(text: &str) -> Result<u16, LibraryError> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(unreadable_bars());
    }
    text.parse::<u16>()
        .ok()
        .filter(|bar| *bar <= MAX_BAR)
        .ok_or_else(|| sections_error(format!("Bars go up to {MAX_BAR}")))
}

pub(crate) fn validate_bar_range(first: u16, last: u16) -> Result<BarRange, LibraryError> {
    if first == 0 {
        return Err(sections_error("Bars start at 1"));
    }
    if last > MAX_BAR {
        return Err(sections_error(format!("Bars go up to {MAX_BAR}")));
    }
    if last < first {
        return Err(sections_error("Bars run from the lower bar to the higher"));
    }
    Ok(BarRange { first, last })
}

/// Every row of an `UpdateSections`, refused whole on the first bad one.
pub(crate) fn validate_section_edits(
    edits: Vec<SectionEdit>,
) -> Result<Vec<SectionDraft>, LibraryError> {
    edits
        .into_iter()
        .map(|edit| {
            let name = edit.name.trim().to_string();
            if exceeds_chars(&name, MAX_SECTION_NAME) {
                return Err(sections_error(format!(
                    "A section name must not exceed {MAX_SECTION_NAME} characters"
                )));
            }
            let bars = match edit.bars {
                BarsInput::Blank => None,
                BarsInput::Picked { first, last } => Some(validate_bar_range(first, last)?),
                BarsInput::Typed(raw) => parse_bar_range(&raw)?,
            };
            if name.is_empty() && bars.is_none() {
                return Err(sections_error("A section needs a name or bars"));
            }
            let target_bpm = parse_bpm(&edit.target_bpm).map_err(|e| match e {
                LibraryError::Validation { .. } => sections_error(format!(
                    "A section's target tempo must be a whole number between {MIN_BPM} and {MAX_BPM}"
                )),
                other => other,
            })?;
            Ok(SectionDraft {
                id: edit.id,
                name,
                bars,
                kind: edit.kind,
                target_bpm,
            })
        })
        .collect()
}

pub fn validate_variation_label(label: &str) -> Result<(), LibraryError> {
    if label.is_empty() || exceeds_chars(label, MAX_VARIATION_LABEL) {
        return Err(LibraryError::Validation {
            field: "labels".to_string(),
            message: format!(
                "Each variation label must be between 1 and {MAX_VARIATION_LABEL} characters"
            ),
        });
    }
    Ok(())
}

pub fn validate_variation_labels(labels: &[String]) -> Result<(), LibraryError> {
    if labels.len() > MAX_VARIATIONS {
        return Err(LibraryError::Validation {
            field: "labels".to_string(),
            message: format!("An item can have at most {MAX_VARIATIONS} variations"),
        });
    }
    labels
        .iter()
        .try_for_each(|label| validate_variation_label(label))
}

/// A label already on another live row would make two rows one name, which a
/// picker cannot tell apart.
pub fn validate_variation_label_free(
    library: &[Variation],
    label: &str,
    except_id: &str,
) -> Result<(), LibraryError> {
    match crate::domain::variation::live_with_label(library, label) {
        Some(v) if v.id != except_id => Err(LibraryError::Validation {
            field: "labels".to_string(),
            message: format!("There is already a variation called \u{201c}{label}\u{201d}"),
        }),
        _ => Ok(()),
    }
}

/// Live library rows, no repeats, within the cap.
pub fn validate_variation_ids(model: &Model, ids: &[String]) -> Result<(), LibraryError> {
    let invalid = |message: &str| LibraryError::Validation {
        field: "variation_ids".to_string(),
        message: message.to_string(),
    };
    if ids.len() > MAX_VARIATIONS {
        return Err(invalid(&format!(
            "At most {MAX_VARIATIONS} variations at once"
        )));
    }
    let mut seen = std::collections::HashSet::new();
    for id in ids {
        if !seen.insert(id.as_str()) {
            return Err(invalid("A variation is listed twice"));
        }
        if !crate::domain::variation::is_live(&model.variations, id) {
            return Err(invalid("That variation is no longer in the library"));
        }
    }
    Ok(())
}

pub fn validate_keys(keys: &[Key]) -> Result<(), LibraryError> {
    if keys.len() > MAX_KEYS {
        return Err(LibraryError::Validation {
            field: "keys".to_string(),
            message: format!("An item can have at most {MAX_KEYS} keys"),
        });
    }
    if keys
        .iter()
        .enumerate()
        .any(|(i, k)| keys[..i].iter().any(|seen| seen.same_key(k)))
    {
        return Err(LibraryError::Validation {
            field: "keys".to_string(),
            message: "A key is listed twice".to_string(),
        });
    }
    Ok(())
}

/// A section named against an entry must be a live section of its item.
fn validate_entry_section(
    entry: &SetlistEntry,
    section_id: &str,
    model: &Model,
) -> Result<(), LibraryError> {
    let live = model
        .items
        .iter()
        .find(|i| i.id == entry.item_id)
        .is_some_and(|item| {
            item.sections
                .iter()
                .any(|s| s.id == section_id && s.deleted_at.is_none())
        });
    if live {
        return Ok(());
    }
    Err(LibraryError::Validation {
        field: "section_id".to_string(),
        message: "That section doesn't belong to this item".to_string(),
    })
}

/// At most one planned section for now; segments (#2315) lift the cap.
pub fn validate_entry_plan(
    entry: &SetlistEntry,
    section_ids: &[String],
    variation_ids: &[String],
    model: &Model,
) -> Result<(), LibraryError> {
    if section_ids.len() > 1 {
        return Err(LibraryError::Validation {
            field: "section_ids".to_string(),
            message: "Plan one section at a time".to_string(),
        });
    }
    for id in section_ids {
        validate_entry_section(entry, id, model)?;
    }
    validate_variation_ids(model, variation_ids)
}

pub fn validate_play_way(
    entry: &SetlistEntry,
    way: &PlayWay,
    model: &Model,
) -> Result<(), LibraryError> {
    if let Some(id) = &way.section_id {
        validate_entry_section(entry, id, model)?;
    }
    validate_variation_ids(model, &way.variation_ids)
}

/// A `play_id` from the shell must name a play of the entry it was sent with,
/// so one row of the item-complete sheet can never write another's (#1739).
pub fn validate_play_belongs(entry: &SetlistEntry, play_id: &str) -> Result<(), LibraryError> {
    if entry.plays.iter().any(|p| p.id == play_id) {
        return Ok(());
    }

    Err(LibraryError::Validation {
        field: "play_id".to_string(),
        message: "That play doesn't belong to this item".to_string(),
    })
}

/// Bounds the crash-recovery blob on the tier where the device is the only
/// copy: an entry holds at most `MAX_PLAYS_PER_ENTRY` plays (#1739).
pub fn validate_play_capacity(entry: &SetlistEntry) -> Result<(), LibraryError> {
    if entry.plays.len() >= MAX_PLAYS_PER_ENTRY {
        return Err(LibraryError::Validation {
            field: "plays".to_string(),
            message: format!("An item can record at most {MAX_PLAYS_PER_ENTRY} plays"),
        });
    }

    Ok(())
}

// ── Profile ──

pub fn normalize_profile(mut profile: Profile) -> Profile {
    profile.name = profile.name.trim().to_string();
    profile.instrument = profile.instrument.trim().to_string();
    if profile.icon_choice == Some(suggest_icon(&profile.instrument)) {
        profile.icon_choice = None;
    }
    profile
}

pub fn validate_profile(profile: &Profile) -> Result<(), LibraryError> {
    if exceeds_chars(&profile.name, MAX_PROFILE_NAME) {
        return Err(LibraryError::Validation {
            field: "name".to_string(),
            message: format!("Name must be {MAX_PROFILE_NAME} characters or fewer"),
        });
    }
    if exceeds_chars(&profile.instrument, MAX_INSTRUMENT) {
        return Err(LibraryError::Validation {
            field: "instrument".to_string(),
            message: format!("Instrument must be {MAX_INSTRUMENT} characters or fewer"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- normalisation (#883) ---

    #[test]
    fn normalize_create_trims_and_drops_blank_optionals() {
        let input = CreateItem {
            title: "  Clair de Lune  ".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("   ".to_string()),
            key: crate::domain::key::Key::parse("  D major "),
            tempo: Some(TempoInput {
                marking: Some("   ".to_string()),
                bpm: None,
            }),
            notes: Some("  ".to_string()),
            tags: vec![
                "  warm-up ".to_string(),
                "   ".to_string(),
                "scales".to_string(),
            ],
            photo_id: None,
            variation_labels: Vec::new(),
        };

        let out = normalize_create_item(input);

        assert_eq!(out.title, "Clair de Lune");
        assert_eq!(
            out.composer, None,
            "whitespace-only composer collapses to None"
        );
        assert_eq!(out.notes, None, "whitespace-only notes collapses to None");
        assert_eq!(out.tags, vec!["warm-up".to_string(), "scales".to_string()]);
        assert_eq!(
            validate_create_item(&out),
            Ok(None),
            "blank marking + no bpm drops the tempo"
        );
    }

    #[test]
    fn distinct_ignoring_case_reads_what_a_musician_types() {
        let cases: &[(&[&str], &[&str])] = &[
            (&[], &[]),
            (&["", "   "], &[]),
            (&["Jazz", "jazz", " JAZZ "], &["Jazz"]),
            (&[" jazz", "Jazz"], &["jazz"]),
            (&["Bach ", "bach", "Chopin"], &["Bach", "Chopin"]),
            (&["scales", "warm-up", "Scales"], &["scales", "warm-up"]),
        ];
        for (input, expected) in cases {
            assert_eq!(distinct_ignoring_case(input.iter()), *expected, "{input:?}");
        }
    }

    #[test]
    fn normalize_dedupes_tags_case_insensitively_keeping_first_casing() {
        let input = CreateItem {
            title: "Etude".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![
                "Jazz".to_string(),
                "jazz".to_string(),
                "  JAZZ  ".to_string(),
                "blues".to_string(),
            ],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert_eq!(
            normalize_create_item(input).tags,
            vec!["Jazz".to_string(), "blues".to_string()]
        );

        let update = UpdateItem {
            tags: Some(vec!["Recital".to_string(), "recital".to_string()]),
            ..Default::default()
        };
        assert_eq!(
            normalize_update_item(update).tags,
            Some(vec!["Recital".to_string()])
        );
    }

    #[test]
    fn normalize_tempo_keeps_bpm_when_marking_blank() {
        // Blank marking must not discard a valid bpm — only a fully-empty
        // tempo collapses to None.
        let input = CreateItem {
            title: "Etude".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: Some(TempoInput {
                marking: Some("  ".to_string()),
                bpm: Some("120".to_string()),
            }),
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert_eq!(
            validate_create_item(&normalize_create_item(input)),
            Ok(Some(Tempo {
                marking: None,
                bpm: Some(120)
            }))
        );
    }

    #[test]
    fn normalize_create_keeps_padded_composer_trimmed() {
        let input = CreateItem {
            title: "Hanon".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("  Charles-Louis Hanon ".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert_eq!(
            normalize_create_item(input).composer,
            Some("Charles-Louis Hanon".to_string())
        );
    }

    #[test]
    fn normalize_update_trims_and_three_state_clears_blank() {
        let input = UpdateItem {
            title: Some("  Renamed ".to_string()),
            composer: Some(Some("   ".to_string())),
            tempo: Some(TempoInput {
                marking: Some("  ".to_string()),
                bpm: None,
            }),
            tags: Some(vec![" a ".to_string(), "".to_string()]),
            ..Default::default()
        };

        let out = normalize_update_item(input);

        assert_eq!(out.title, Some("Renamed".to_string()));
        assert_eq!(
            out.composer,
            Some(None),
            "blank set-composer becomes a clear"
        );
        assert_eq!(
            validate_update_item(&out),
            Ok(Some(None)),
            "blank tempo becomes a clear"
        );
        assert_eq!(out.tags, Some(vec!["a".to_string()]));
    }

    #[test]
    fn normalize_update_leaves_untouched_fields_none() {
        let out = normalize_update_item(UpdateItem::default());
        assert_eq!(
            out.composer, None,
            "absent field stays absent (not a clear)"
        );
        assert_eq!(out.title, None);
        assert_eq!(out.tags, None);
    }

    // --- validate_create_item tests (piece kind) ---

    #[test]
    fn test_valid_create_piece() {
        let input = CreateItem {
            title: "Moonlight Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: crate::domain::key::Key::parse("C# minor"),
            tempo: Some(TempoInput {
                marking: Some("Adagio sostenuto".to_string()),
                bpm: Some("60".to_string()),
            }),
            notes: Some("First movement".to_string()),
            tags: vec!["classical".to_string(), "piano".to_string()],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert!(validate_create_item(&input).is_ok());
    }

    #[test]
    fn test_create_piece_empty_title() {
        let input = CreateItem {
            title: "".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_title_too_long() {
        let input = CreateItem {
            title: "x".repeat(501),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_no_composer() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert!(validate_create_item(&input).is_ok());
    }

    #[test]
    fn test_create_piece_empty_composer() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_composer_too_long() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("x".repeat(201)),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_notes_too_long() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: Some("x".repeat(5001)),
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "notes");
                assert_eq!(message, "Notes must not exceed 5000 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_notes_at_limit() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: Some("x".repeat(5000)),
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert!(validate_create_item(&input).is_ok());
    }

    #[test]
    fn test_create_piece_minimal() {
        let input = CreateItem {
            title: "A".to_string(),
            kind: ItemKind::Piece,
            composer: Some("B".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert!(validate_create_item(&input).is_ok());
    }

    // --- validate_create_item tests (exercise kind) ---

    #[test]
    fn test_valid_create_exercise() {
        let input = CreateItem {
            title: "Scale Practice".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("Hanon".to_string()),
            key: crate::domain::key::Key::parse("C major"),
            tempo: Some(TempoInput {
                marking: Some("Moderato".to_string()),
                bpm: Some("100".to_string()),
            }),
            notes: Some("Practice daily".to_string()),
            tags: vec!["technique".to_string()],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert!(validate_create_item(&input).is_ok());
    }

    #[test]
    fn test_create_exercise_with_inline_variations_is_valid() {
        let input = CreateItem {
            title: "Scale Practice".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: vec!["C".to_string(), "F".to_string()],
        };
        assert!(validate_create_item(&input).is_ok());
    }

    #[test]
    fn test_create_piece_with_variations_is_allowed() {
        let input = CreateItem {
            title: "Clair de Lune".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Debussy".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: vec!["Slow".to_string()],
        };
        assert!(
            validate_create_item(&input).is_ok(),
            "pieces and exercises alike (#2246)"
        );
    }

    #[test]
    fn validate_no_variant_labels_passes_an_empty_list_and_rejects_a_populated_one() {
        let mut input = CreateItem {
            title: "Shell voicings".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert!(validate_no_variation_labels(&input).is_ok());

        input.variation_labels = vec!["C".to_string()];
        let err = validate_no_variation_labels(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, .. } => assert_eq!(field, "variation_labels"),
            _ => panic!("expected a validation error"),
        }
    }

    #[test]
    fn test_create_exercise_empty_title() {
        let input = CreateItem {
            title: "".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_title_too_long() {
        let input = CreateItem {
            title: "x".repeat(501),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_empty_composer() {
        let input = CreateItem {
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_composer_too_long() {
        let input = CreateItem {
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("x".repeat(201)),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_notes_too_long() {
        let input = CreateItem {
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: Some("x".repeat(5001)),
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "notes");
                assert_eq!(message, "Notes must not exceed 5000 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_no_optional_fields() {
        let input = CreateItem {
            title: "Warm up".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        assert!(validate_create_item(&input).is_ok());
    }

    // --- validate_tags tests ---

    #[test]
    fn test_valid_tags() {
        let tags = vec!["classical".to_string(), "piano".to_string()];
        assert!(validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_empty_tag() {
        let tags = vec!["classical".to_string(), "".to_string()];
        let err = validate_tags(&tags).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tag_too_long() {
        let tags = vec!["x".repeat(101)];
        let err = validate_tags(&tags).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tag_at_limit() {
        let tags = vec!["x".repeat(100)];
        assert!(validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_empty_tags_vec() {
        let tags: Vec<String> = vec![];
        assert!(validate_tags(&tags).is_ok());
    }

    // --- validate_tempo tests ---

    #[test]
    fn test_valid_tempo_both_fields() {
        let tempo = Tempo {
            marking: Some("Allegro".to_string()),
            bpm: Some(120),
        };
        assert!(validate_tempo(&tempo).is_ok());
    }

    #[test]
    fn test_valid_tempo_marking_only() {
        let tempo = Tempo {
            marking: Some("Adagio".to_string()),
            bpm: None,
        };
        assert!(validate_tempo(&tempo).is_ok());
    }

    #[test]
    fn test_valid_tempo_bpm_only() {
        let tempo = Tempo {
            marking: None,
            bpm: Some(120),
        };
        assert!(validate_tempo(&tempo).is_ok());
    }

    #[test]
    fn test_tempo_neither_field() {
        let tempo = Tempo {
            marking: None,
            bpm: None,
        };
        let err = validate_tempo(&tempo).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "Tempo must have at least a marking or BPM value");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tempo_marking_too_long() {
        let tempo = Tempo {
            marking: Some("x".repeat(101)),
            bpm: None,
        };
        let err = validate_tempo(&tempo).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "Tempo marking must not exceed 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tempo_marking_at_limit() {
        let tempo = Tempo {
            marking: Some("x".repeat(100)),
            bpm: None,
        };
        assert!(validate_tempo(&tempo).is_ok());
    }

    #[test]
    fn test_tempo_bpm_zero() {
        let tempo = Tempo {
            marking: None,
            bpm: Some(0),
        };
        let err = validate_tempo(&tempo).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tempo_bpm_too_high() {
        let tempo = Tempo {
            marking: None,
            bpm: Some(401),
        };
        let err = validate_tempo(&tempo).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tempo_bpm_at_limits() {
        let tempo_min = Tempo {
            marking: None,
            bpm: Some(1),
        };
        assert!(validate_tempo(&tempo_min).is_ok());

        let tempo_max = Tempo {
            marking: None,
            bpm: Some(400),
        };
        assert!(validate_tempo(&tempo_max).is_ok());
    }

    // --- validate_update_item tests ---

    #[test]
    fn test_valid_update_item_no_fields() {
        let input = UpdateItem::default();
        assert!(validate_update_item(&input).is_ok());
    }

    #[test]
    fn test_update_item_empty_title() {
        let input = UpdateItem {
            title: Some("".to_string()),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_title_too_long() {
        let input = UpdateItem {
            title: Some("x".repeat(501)),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_empty_composer() {
        let input = UpdateItem {
            composer: Some(Some("".to_string())),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_clear_composer() {
        let input = UpdateItem {
            composer: Some(None),
            ..Default::default()
        };
        assert!(validate_update_item(&input).is_ok());
    }

    #[test]
    fn test_update_item_notes_too_long() {
        let input = UpdateItem {
            notes: Some(Some("x".repeat(5001))),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "notes");
                assert_eq!(message, "Notes must not exceed 5000 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_clear_notes() {
        let input = UpdateItem {
            notes: Some(None),
            ..Default::default()
        };
        assert!(validate_update_item(&input).is_ok());
    }

    #[test]
    fn test_update_item_invalid_tags() {
        let input = UpdateItem {
            tags: Some(vec!["".to_string()]),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_clear_tempo() {
        let input = UpdateItem {
            tempo: Some(TempoInput::default()),
            ..Default::default()
        };
        assert_eq!(validate_update_item(&input), Ok(Some(None)));
    }

    #[test]
    fn test_update_item_leaves_tempo_alone() {
        assert_eq!(validate_update_item(&UpdateItem::default()), Ok(None));
    }

    #[test]
    fn test_create_piece_with_invalid_tempo() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Bach".to_string()),
            key: None,
            tempo: Some(TempoInput {
                marking: Some("x".repeat(101)),
                bpm: Some("120".to_string()),
            }),
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "Tempo marking must not exceed 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_with_invalid_tags() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Bach".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec!["good".to_string(), "".to_string()],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_with_invalid_tempo() {
        let input = CreateItem {
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: Some(TempoInput {
                marking: None,
                bpm: Some("500".to_string()),
            }),
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be a whole number between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_invalid_tempo_bpm() {
        let input = UpdateItem {
            tempo: Some(TempoInput {
                marking: None,
                bpm: Some("12a".to_string()),
            }),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be a whole number between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_tags_too_long() {
        let input = UpdateItem {
            tags: Some(vec!["x".repeat(101)]),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // --- validate_metre / validate_click_state ---

    fn metre(beats: u8, unit: u8, groups: Option<Vec<u8>>) -> crate::domain::Metre {
        crate::domain::Metre {
            beats,
            unit,
            groups,
        }
    }

    /// Metres a musician would actually enter, and the ones the picker must
    /// refuse, asserting what the click sheet needs: accepted means playable.
    #[test]
    fn metres_a_musician_enters_validate_as_the_picker_expects() {
        let accepted = [
            metre(4, 4, None),
            metre(3, 4, None),
            metre(2, 2, None),
            metre(6, 8, Some(vec![3, 3])),
            metre(7, 8, Some(vec![3, 2, 2])),
            metre(12, 8, Some(vec![3, 3, 3, 3])),
            metre(5, 4, Some(vec![3, 2])),
        ];
        for m in accepted {
            assert!(validate_metre(&m).is_ok(), "{m:?}");
        }
        let refused = [
            metre(1, 4, None),
            metre(13, 8, None),
            metre(4, 3, None),
            metre(4, 16, None),
            metre(7, 8, Some(vec![3, 3])),
            metre(7, 8, Some(vec![])),
            metre(4, 4, Some(vec![4, 0])),
        ];
        for m in refused {
            assert!(validate_metre(&m).is_err(), "{m:?}");
        }
    }

    #[test]
    fn a_click_must_sound_inside_the_bar() {
        use crate::domain::session::ClickState;
        let ok = ClickState {
            metre: metre(4, 4, None),
            sounding: 0b1010,
        };
        assert!(validate_click_state(&ok).is_ok());
        let silent = ClickState {
            metre: metre(4, 4, None),
            sounding: 0,
        };
        assert!(validate_click_state(&silent).is_err());
        let beyond = ClickState {
            metre: metre(3, 4, None),
            sounding: 0b1000,
        };
        assert!(
            validate_click_state(&beyond).is_err(),
            "beat 4 of a 3/4 bar"
        );
    }

    // --- validate_achieved_tempo tests ---

    #[test]
    fn test_achieved_tempo_none() {
        assert!(validate_achieved_tempo(&None).is_ok());
    }

    #[test]
    fn test_achieved_tempo_valid() {
        assert!(validate_achieved_tempo(&Some(120)).is_ok());
    }

    #[test]
    fn test_achieved_tempo_at_min() {
        assert!(validate_achieved_tempo(&Some(1)).is_ok());
    }

    #[test]
    fn test_achieved_tempo_at_max() {
        assert!(validate_achieved_tempo(&Some(500)).is_ok());
    }

    #[test]
    fn test_achieved_tempo_zero() {
        let err = validate_achieved_tempo(&Some(0)).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "achieved_tempo");
                assert_eq!(message, "Achieved tempo must be between 1 and 500 BPM");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_achieved_tempo_above_max() {
        let err = validate_achieved_tempo(&Some(501)).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "achieved_tempo");
                assert_eq!(message, "Achieved tempo must be between 1 and 500 BPM");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // --- validate_entries_not_empty tests ---

    #[test]
    fn test_entries_not_empty_ok() {
        assert!(validate_entries_not_empty(&[1, 2, 3], "Setlist").is_ok());
    }

    #[test]
    fn test_entries_empty_setlist() {
        let entries: Vec<i32> = vec![];
        let err = validate_entries_not_empty(&entries, "Setlist").unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "entries");
                assert_eq!(message, "Setlist must have at least one entry");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_entries_empty_set() {
        let entries: Vec<i32> = vec![];
        let err = validate_entries_not_empty(&entries, "Set").unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "entries");
                assert_eq!(message, "Set must have at least one entry");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn too_many_variation_labels_names_variations() {
        let labels: Vec<String> = (0..=MAX_VARIATIONS).map(|i| format!("Way {i}")).collect();
        let err = validate_variation_labels(&labels).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "labels");
                assert_eq!(message, "An item can have at most 24 variations");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn empty_variation_label_names_variation() {
        let labels = vec![String::new()];
        let err = validate_variation_labels(&labels).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "labels");
                assert_eq!(
                    message,
                    "Each variation label must be between 1 and 100 characters"
                );
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // --- length caps count characters, not bytes (#1944) ---

    fn create_with(composer: Option<String>, notes: Option<String>) -> CreateItem {
        CreateItem {
            title: "Etude".to_string(),
            kind: ItemKind::Piece,
            composer,
            key: None,
            tempo: None,
            notes,
            tags: vec![],
            photo_id: None,
            variation_labels: vec![],
        }
    }

    /// "é" is two bytes, so a byte count refuses these at half the cap. Deleting
    /// the character count from any one validator fails its row at the cap.
    #[test]
    fn every_length_cap_counts_accented_characters_not_bytes() {
        type Check = fn(String) -> Result<(), LibraryError>;
        let table: [(&str, usize, Check); 13] = [
            ("title", MAX_TITLE, |s| validate_title(&s)),
            ("create composer", MAX_COMPOSER, |s| {
                validate_create_item(&create_with(Some(s), None)).map(drop)
            }),
            ("create notes", MAX_NOTES, |s| {
                validate_create_item(&create_with(None, Some(s))).map(drop)
            }),
            ("update composer", MAX_COMPOSER, |s| {
                validate_update_item(&UpdateItem {
                    composer: Some(Some(s)),
                    ..Default::default()
                })
                .map(drop)
            }),
            ("update notes", MAX_NOTES, |s| {
                validate_update_item(&UpdateItem {
                    notes: Some(Some(s)),
                    ..Default::default()
                })
                .map(drop)
            }),
            ("session notes", MAX_NOTES, |s| {
                validate_session_notes(&Some(s))
            }),
            ("entry notes", MAX_NOTES, |s| validate_entry_notes(&Some(s))),
            ("tag", MAX_TAG, |s| validate_tags(&[s])),
            ("intention", MAX_INTENTION, |s| validate_intention(&Some(s))),
            ("tempo marking", MAX_TEMPO_MARKING, |s| {
                validate_tempo(&Tempo {
                    marking: Some(s),
                    bpm: None,
                })
            }),
            ("variation label", MAX_VARIATION_LABEL, |s| {
                validate_variation_labels(&[s])
            }),
            ("profile name", MAX_PROFILE_NAME, |s| {
                validate_profile(&Profile {
                    name: s,
                    ..Default::default()
                })
            }),
            ("instrument", MAX_INSTRUMENT, |s| {
                validate_profile(&Profile {
                    instrument: s,
                    ..Default::default()
                })
            }),
        ];

        for (field, max, check) in table {
            assert!(
                check("é".repeat(max)).is_ok(),
                "{field}: {max} characters at the cap"
            );
            assert!(
                check("é".repeat(max + 1)).is_err(),
                "{field}: one over the cap"
            );
        }
    }

    // --- session length (#1736) ---

    #[test]
    fn session_length_accepts_the_stepper_s_values_and_refuses_the_rest() {
        let table: &[(Option<u16>, bool)] = &[
            (None, true),
            (Some(10), true),
            (Some(30), true),
            (Some(45), true),
            (Some(120), true),
            (Some(0), false),
            (Some(5), false),
            (Some(32), false),
            (Some(125), false),
            (Some(u16::MAX), false),
        ];
        for (length, ok) in table {
            assert_eq!(
                validate_session_length(length).is_ok(),
                *ok,
                "{length:?} minutes"
            );
        }
    }

    #[test]
    fn a_stored_session_length_is_brought_back_onto_the_stepper() {
        for (stored, expected) in [
            (0, 10),
            (5, 10),
            (32, 30),
            (33, 35),
            (45, 45),
            (300, 120),
            (u16::MAX, 120),
        ] {
            assert_eq!(clamp_session_length(stored), expected, "{stored} minutes");
            assert!(validate_session_length(&Some(expected)).is_ok());
        }
    }

    // --- typed BPM (#2224) ---

    #[test]
    fn parse_bpm_reads_what_a_musician_types() {
        let refused = Err("BPM must be a whole number between 1 and 400");
        let cases: [(&str, Result<Option<u16>, &str>); 16] = [
            ("", Ok(None)),
            ("   ", Ok(None)),
            ("96", Ok(Some(96))),
            (" 96 ", Ok(Some(96))),
            ("096", Ok(Some(96))),
            ("1", Ok(Some(1))),
            ("400", Ok(Some(400))),
            ("0", refused),
            ("401", refused),
            ("9000", refused),
            ("12a", refused),
            ("96.5", refused),
            ("+96", refused),
            ("-5", refused),
            ("96 bpm", refused),
            ("99999999999999999999", refused),
        ];
        for (typed, expected) in cases {
            let got = parse_bpm(typed).map_err(|e| match e {
                LibraryError::Validation { field, message } => {
                    assert_eq!(field, "tempo", "{typed:?}");
                    message
                }
                other => panic!("{typed:?}: expected a validation error, got {other:?}"),
            });
            assert_eq!(got, expected.map_err(str::to_string), "{typed:?}");
        }
    }

    #[test]
    fn parse_tempo_clears_when_both_parts_are_blank() {
        let blank = TempoInput {
            marking: Some("  ".to_string()),
            bpm: Some(" ".to_string()),
        };
        assert_eq!(parse_tempo(&blank), Ok(None));
        let marked = TempoInput {
            marking: Some(" Allegro ".to_string()),
            bpm: Some("".to_string()),
        };
        assert_eq!(
            parse_tempo(&marked),
            Ok(Some(Tempo {
                marking: Some("Allegro".to_string()),
                bpm: None,
            }))
        );
    }
}
