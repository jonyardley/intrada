//! Loads a PR 2 capture JSONL into engine types. Host-time ticks are converted
//! to microseconds from the click anchor using the take's own mach timebase.

use intrada_core::engine::{ClickGrid, NoteEvent};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Header {
    bpm: f64,
    beats_per_bar: u8,
    count_in_beats: u8,
    start_host_time: u64,
    host_timebase_numer: u64,
    host_timebase_denom: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Line {
    #[serde(rename = "type")]
    kind: String,
    host_time: Option<u64>,
    midi_note: Option<u8>,
    velocity: Option<u8>,
    is_note_on: Option<bool>,
}

pub struct MidiTake {
    pub grid: ClickGrid,
    pub events: Vec<NoteEvent>,
}

impl MidiTake {
    pub fn load(name: &str) -> Self {
        let path = format!(
            "{}/tests/fixtures/midi_takes/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let mut lines = raw.lines().filter(|line| !line.trim().is_empty());

        let header: Header = serde_json::from_str(lines.next().expect("take header line"))
            .expect("take header parses");
        let grid = ClickGrid::new(
            (header.bpm * 1000.0).round() as u32,
            header.beats_per_bar,
            header.count_in_beats,
        );

        let events = lines
            .map(|line| serde_json::from_str::<Line>(line).expect("note line parses"))
            .filter(|line| line.kind == "note")
            .map(|line| NoteEvent {
                pitch: line.midi_note.expect("note line has midiNote"),
                velocity: line.velocity.expect("note line has velocity"),
                on: line.is_note_on.expect("note line has isNoteOn"),
                t_us: ticks_to_us(
                    line.host_time.expect("note line has hostTime"),
                    header.start_host_time,
                    header.host_timebase_numer,
                    header.host_timebase_denom,
                ),
            })
            .collect();

        Self { grid, events }
    }
}

fn ticks_to_us(host_time: u64, anchor: u64, numer: u64, denom: u64) -> i64 {
    let delta = host_time as i128 - anchor as i128;
    (delta * numer as i128 / denom as i128 / 1_000) as i64
}
