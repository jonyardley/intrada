import Foundation

enum TransportType: String, Codable {
  case usb
  case bluetooth
  case unknown
}

/// A captured CoreMIDI note-on/off, timestamped in host-time ticks (the same
/// domain as `BeatGrid`). Bar/beat/offsetMs are computed against the grid at
/// record time so the exported fixture is self-contained.
struct NoteEvent: Codable {
  let hostTime: UInt64
  let midiNote: UInt8
  let velocity: UInt8
  let isNoteOn: Bool
  var bar: Int?
  var beat: Int?
  var offsetMs: Double?
}

/// Written once per take, first line of the JSONL fixture.
struct TakeHeader: Codable {
  var type = "take_header"
  let transport: TransportType
  let bpm: Double
  let beatsPerBar: Int
  let countInBeats: Int
  let startHostTime: UInt64
  let hostTimebaseNumer: UInt32
  let hostTimebaseDenom: UInt32
  let recordedAt: Date
}

/// One line per captured note, tagged so the JSONL stream self-describes.
struct NoteEventLine: Codable {
  var type = "note"
  let event: NoteEvent

  enum CodingKeys: String, CodingKey {
    case type, hostTime, midiNote, velocity, isNoteOn, bar, beat, offsetMs
  }

  init(event: NoteEvent) {
    self.event = event
  }

  init(from decoder: Decoder) throws {
    let container = try decoder.container(keyedBy: CodingKeys.self)
    type = try container.decode(String.self, forKey: .type)
    event = NoteEvent(
      hostTime: try container.decode(UInt64.self, forKey: .hostTime),
      midiNote: try container.decode(UInt8.self, forKey: .midiNote),
      velocity: try container.decode(UInt8.self, forKey: .velocity),
      isNoteOn: try container.decode(Bool.self, forKey: .isNoteOn),
      bar: try container.decodeIfPresent(Int.self, forKey: .bar),
      beat: try container.decodeIfPresent(Int.self, forKey: .beat),
      offsetMs: try container.decodeIfPresent(Double.self, forKey: .offsetMs))
  }

  func encode(to encoder: Encoder) throws {
    var container = encoder.container(keyedBy: CodingKeys.self)
    try container.encode(type, forKey: .type)
    try container.encode(event.hostTime, forKey: .hostTime)
    try container.encode(event.midiNote, forKey: .midiNote)
    try container.encode(event.velocity, forKey: .velocity)
    try container.encode(event.isNoteOn, forKey: .isNoteOn)
    try container.encodeIfPresent(event.bar, forKey: .bar)
    try container.encodeIfPresent(event.beat, forKey: .beat)
    try container.encodeIfPresent(event.offsetMs, forKey: .offsetMs)
  }
}
