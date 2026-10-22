// ── Shared codec ─────────────────────────────────────────────────────

use chrono::{DateTime, SecondsFormat, Utc};
use intrada_core::domain::chart::{
    Bar, ChartChord, ChartSection, ChordChart, ChordQuality, ChordSymbol,
};
use intrada_core::domain::key::{key_from_stored, key_to_stored};
use intrada_core::domain::section::SectionKind;
use intrada_core::domain::Metre;
use intrada_core::{ItemKind, Key, Modality};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// What a load could not read: each is reported by the phone, never a
/// silent default (#949).
pub(crate) type Unreadable = Vec<String>;

/// The text the bridge carries for a time, so both phones store one format.
pub(crate) fn time_text(time: &DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

pub(crate) fn parse_time(text: &str) -> Option<DateTime<Utc>> {
    text.parse().ok()
}

pub(crate) fn encode_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub(crate) fn try_decode_json<T: DeserializeOwned>(json: &str) -> Option<T> {
    serde_json::from_str(json).ok()
}

/// A JSON column that will not decode is reported and reads back as none;
/// the save keeps the stored text (#1117).
pub(crate) fn decode_json<T: DeserializeOwned>(
    json: &str,
    field: &str,
    id: &str,
    unreadable: &mut Unreadable,
) -> Option<T> {
    let value = try_decode_json(json);
    if value.is_none() {
        unreadable.push(format!("{field} of {id} failed to decode"));
    }
    value
}

fn unknown(kind: &str, raw: &str, unreadable: &mut Unreadable) {
    unreadable.push(format!("unknown {kind} on decode: \"{raw}\""));
}

// ── Stored enums ─────────────────────────────────────────────────────

pub(crate) fn item_kind_text(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::Piece => "piece",
        ItemKind::Exercise => "exercise",
    }
}

fn known_item_kind(raw: &str) -> Option<ItemKind> {
    match raw {
        "piece" => Some(ItemKind::Piece),
        "exercise" => Some(ItemKind::Exercise),
        _ => None,
    }
}

const UNKNOWN_ITEM_KIND: ItemKind = ItemKind::Piece;

pub(crate) fn item_kind(raw: &str, unreadable: &mut Unreadable) -> ItemKind {
    known_item_kind(raw).unwrap_or_else(|| {
        unknown("ItemKind", raw, unreadable);
        UNKNOWN_ITEM_KIND
    })
}

/// Text a newer build wrote loads as the fallback, so saving the fallback
/// keeps the text until the musician picks a kind this build knows (#2437).
pub(crate) fn item_kind_to_write(kind: &ItemKind, stored: Option<String>) -> String {
    match stored {
        Some(text) if known_item_kind(&text).is_none() && *kind == UNKNOWN_ITEM_KIND => text,
        _ => item_kind_text(kind).to_string(),
    }
}

pub(crate) fn section_kind_text(kind: SectionKind) -> &'static str {
    match kind {
        SectionKind::Form => "form",
        SectionKind::TroubleSpot => "trouble_spot",
    }
}

fn known_section_kind(raw: &str) -> Option<SectionKind> {
    match raw {
        "form" => Some(SectionKind::Form),
        "trouble_spot" => Some(SectionKind::TroubleSpot),
        _ => None,
    }
}

const UNKNOWN_SECTION_KIND: SectionKind = SectionKind::Form;

pub(crate) fn section_kind(raw: &str, unreadable: &mut Unreadable) -> SectionKind {
    known_section_kind(raw).unwrap_or_else(|| {
        unknown("SectionKind", raw, unreadable);
        UNKNOWN_SECTION_KIND
    })
}

pub(crate) fn section_kind_to_write(kind: SectionKind, stored: Option<String>) -> String {
    match stored {
        Some(text) if known_section_kind(&text).is_none() && kind == UNKNOWN_SECTION_KIND => text,
        _ => section_kind_text(kind).to_string(),
    }
}

fn modality_text(modality: Modality) -> &'static str {
    match modality {
        Modality::Major => "major",
        Modality::Minor => "minor",
    }
}

fn known_modality(raw: &str) -> Option<Modality> {
    match raw {
        "major" => Some(Modality::Major),
        "minor" => Some(Modality::Minor),
        _ => None,
    }
}

fn modality(raw: &str, unreadable: &mut Unreadable) -> Option<Modality> {
    let mode = known_modality(raw);
    if mode.is_none() {
        unknown("Modality", raw, unreadable);
    }
    mode
}

pub(crate) fn is_unknown_modality(raw: Option<&str>) -> bool {
    raw.is_some_and(|raw| known_modality(raw).is_none())
}

fn chord_quality_text(quality: ChordQuality) -> &'static str {
    match quality {
        ChordQuality::Maj7 => "maj7",
        ChordQuality::Dom7 => "dom7",
        ChordQuality::Min7 => "min7",
        ChordQuality::Min7b5 => "min7b5",
        ChordQuality::Dim7 => "dim7",
        ChordQuality::MinMaj7 => "minMaj7",
        ChordQuality::Six => "six",
        ChordQuality::Min6 => "min6",
        ChordQuality::Alt => "alt",
        ChordQuality::Sus4 => "sus4",
        ChordQuality::Sus2 => "sus2",
        ChordQuality::Aug => "aug",
        ChordQuality::Dom7Sharp5 => "dom7Sharp5",
        ChordQuality::Other => "other",
    }
}

fn chord_quality(raw: &str, unreadable: &mut Unreadable) -> Option<ChordQuality> {
    let quality = known_chord_quality(raw);
    if quality.is_none() {
        unknown("ChordQuality", raw, unreadable);
    }
    quality
}

/// The core's reading of a chord symbol it cannot name.
const UNKNOWN_CHORD_QUALITY: ChordQuality = ChordQuality::Other;

fn known_chord_quality(raw: &str) -> Option<ChordQuality> {
    let quality = match raw {
        "maj7" => ChordQuality::Maj7,
        "dom7" => ChordQuality::Dom7,
        "min7" => ChordQuality::Min7,
        "min7b5" => ChordQuality::Min7b5,
        "dim7" => ChordQuality::Dim7,
        "minMaj7" => ChordQuality::MinMaj7,
        "six" => ChordQuality::Six,
        "min6" => ChordQuality::Min6,
        "alt" => ChordQuality::Alt,
        "sus4" => ChordQuality::Sus4,
        "sus2" => ChordQuality::Sus2,
        "aug" => ChordQuality::Aug,
        "dom7Sharp5" => ChordQuality::Dom7Sharp5,
        "other" => ChordQuality::Other,
        _ => return None,
    };
    Some(quality)
}

// ── Keys (#2106) ─────────────────────────────────────────────────────
// The core reads and writes a key's two columns; the store never parses one.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct StoredKeyJson {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
}

/// Text the core cannot read is `None`; the column keeps it on save.
pub(crate) fn key(
    text: Option<&str>,
    modality_raw: Option<&str>,
    unreadable: &mut Unreadable,
) -> Option<Key> {
    let text = text?;
    let mode = modality_raw.and_then(|raw| modality(raw, unreadable));
    key_from_stored(Some(text), mode)
}

/// The stored key stays, verbatim, when the core could not read it and the
/// save holds no key (#2106), or when its mode is one a newer build wrote and
/// the save holds the key as it loaded (#2437).
pub(crate) fn kept_key(
    text: &str,
    modality_raw: Option<&str>,
    saving: Option<&Key>,
) -> Option<StoredKeyJson> {
    let loaded = key(Some(text), modality_raw, &mut Vec::new());
    let keep = match &loaded {
        None => saving.is_none(),
        Some(_) => is_unknown_modality(modality_raw) && loaded.as_ref() == saving,
    };
    keep.then(|| StoredKeyJson {
        key: text.to_string(),
        modality: modality_raw.map(str::to_string),
    })
}

pub(crate) fn stored_key(key: &Key) -> StoredKeyJson {
    let (text, mode) = key_to_stored(key);
    StoredKeyJson {
        key: text,
        modality: mode.map(|m| modality_text(m).to_string()),
    }
}

pub(crate) fn encode_keys(keys: &[Key]) -> Result<String, serde_json::Error> {
    encode_json(&keys.iter().map(stored_key).collect::<Vec<_>>())
}

/// A key in the list the core cannot read is dropped from the list.
pub(crate) fn decode_keys(json: Option<&str>, id: &str, unreadable: &mut Unreadable) -> Vec<Key> {
    let Some(json) = json else { return vec![] };
    let Some(stored) = decode_json::<Vec<StoredKeyJson>>(json, "keys", id, unreadable) else {
        return vec![];
    };
    stored
        .iter()
        .filter_map(|k| key(Some(&k.key), k.modality.as_deref(), unreadable))
        .collect()
}

// ── Metre ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct StoredMetre {
    pub beats: u8,
    pub unit: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<u8>>,
}

pub(crate) fn encode_metre(metre: Option<&Metre>) -> Result<Option<String>, serde_json::Error> {
    metre
        .map(|m| {
            encode_json(&StoredMetre {
                beats: m.beats,
                unit: m.unit,
                groups: m.groups.clone(),
            })
        })
        .transpose()
}

pub(crate) fn decode_metre(
    json: Option<&str>,
    id: &str,
    unreadable: &mut Unreadable,
) -> Option<Metre> {
    let stored: StoredMetre = decode_json(json?, "metre", id, unreadable)?;
    Some(Metre {
        beats: stored.beats,
        unit: stored.unit,
        groups: stored.groups,
    })
}

// ── Chord chart ──────────────────────────────────────────────────────
// A nested aggregate, so JSON, never bincode: positional encoding would fail
// to decode old rows after a field change, and the device is the only copy.

/// `key` holds a spelling, as the item's column does, or text the core
/// could not read, kept until a key is picked; empty is no key.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct StoredChart {
    pub key: String,
    pub modality: String,
    // Pre-v17 rows carry the beats here; the item's `metre` column owns it now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metre: Option<u8>,
    pub sections: Vec<StoredChartSection>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct StoredChartSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub bars: Vec<StoredBar>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct StoredBar {
    pub chords: Vec<StoredChartChord>,
}

// Rows written before #1948 also carry `beats`; decoding ignores it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct StoredChartChord {
    pub symbol: StoredChordSymbol,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct StoredChordSymbol {
    pub root: u8,
    pub quality: String,
    pub extensions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bass: Option<u8>,
    pub raw: String,
}

/// `stored` is the chart on disk, if it reads, so a key or chord quality the
/// core could not read survives a save that leaves it alone.
pub(crate) fn encode_chord_chart(
    chart: Option<&ChordChart>,
    stored: Option<&StoredChart>,
) -> Result<Option<String>, serde_json::Error> {
    let Some(chart) = chart else { return Ok(None) };
    let key = stored
        .filter(|stored| !stored.key.is_empty())
        .and_then(|stored| kept_key(&stored.key, Some(&stored.modality), chart.key.as_ref()))
        .or_else(|| chart.key.as_ref().map(stored_key));
    let unknown_qualities: Vec<&StoredChordSymbol> = stored
        .into_iter()
        .flat_map(|stored| &stored.sections)
        .flat_map(|section| &section.bars)
        .flat_map(|bar| &bar.chords)
        .map(|chord| &chord.symbol)
        .filter(|symbol| known_chord_quality(&symbol.quality).is_none())
        .collect();
    let stored = StoredChart {
        key: key.as_ref().map(|k| k.key.clone()).unwrap_or_default(),
        modality: key
            .and_then(|k| k.modality)
            .unwrap_or_else(|| modality_text(Modality::Major).to_string()),
        metre: None,
        sections: chart
            .sections
            .iter()
            .map(|section| StoredChartSection {
                label: section.label.clone(),
                bars: section
                    .bars
                    .iter()
                    .map(|bar| StoredBar {
                        chords: bar
                            .chords
                            .iter()
                            .map(|chord| StoredChartChord {
                                symbol: StoredChordSymbol {
                                    root: chord.symbol.root,
                                    quality: quality_to_write(&chord.symbol, &unknown_qualities),
                                    extensions: chord.symbol.extensions.clone(),
                                    bass: chord.symbol.bass,
                                    raw: chord.symbol.raw.clone(),
                                },
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    };
    encode_json(&stored).map(Some)
}

/// A quality a newer build wrote loads as the fallback, so a chord saved as it
/// loaded keeps the stored text, matched by its symbol so an edit elsewhere in
/// the chart does not lose it (#2437).
fn quality_to_write(symbol: &ChordSymbol, unknown: &[&StoredChordSymbol]) -> String {
    unknown
        .iter()
        .find(|stored| {
            symbol.quality == UNKNOWN_CHORD_QUALITY
                && stored.root == symbol.root
                && stored.extensions == symbol.extensions
                && stored.bass == symbol.bass
                && stored.raw == symbol.raw
        })
        .map_or_else(
            || chord_quality_text(symbol.quality).to_string(),
            |stored| stored.quality.clone(),
        )
}

pub(crate) fn decode_chord_chart(
    json: Option<&str>,
    id: &str,
    unreadable: &mut Unreadable,
) -> Option<ChordChart> {
    let stored: StoredChart = decode_json(json?, "chord_chart", id, unreadable)?;
    Some(ChordChart {
        key: key(Some(&stored.key), Some(&stored.modality), unreadable),
        sections: stored
            .sections
            .into_iter()
            .map(|section| ChartSection {
                label: section.label,
                bars: section
                    .bars
                    .into_iter()
                    .map(|bar| Bar {
                        chords: bar
                            .chords
                            .into_iter()
                            .map(|chord| ChartChord {
                                symbol: ChordSymbol {
                                    root: chord.symbol.root,
                                    quality: chord_quality(&chord.symbol.quality, unreadable)
                                        .unwrap_or(UNKNOWN_CHORD_QUALITY),
                                    extensions: chord.symbol.extensions,
                                    bass: chord.symbol.bass,
                                    raw: chord.symbol.raw,
                                },
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    })
}
