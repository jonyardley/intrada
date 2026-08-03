import Foundation

struct TakeSummary {
  let noteOnCount: Int
  let meanOffsetMs: Double
  let stdevOffsetMs: Double
}

enum TakeRecorder {
  private static let encoder: JSONEncoder = {
    let encoder = JSONEncoder()
    encoder.dateEncodingStrategy = .iso8601
    return encoder
  }()

  private static let decoder: JSONDecoder = {
    let decoder = JSONDecoder()
    decoder.dateDecodingStrategy = .iso8601
    return decoder
  }()

  /// Annotates each raw event's bar/beat/offsetMs against `grid`.
  static func annotate(_ events: [NoteEvent], against grid: BeatGrid) -> [NoteEvent] {
    events.map { event in
      var annotated = event
      let nearest = grid.nearestBeat(for: event.hostTime)
      annotated.bar = nearest.bar
      annotated.beat = nearest.beat
      annotated.offsetMs = nearest.offsetMs
      return annotated
    }
  }

  static func summary(of events: [NoteEvent]) -> TakeSummary {
    let onOffsets = events.filter(\.isNoteOn).compactMap(\.offsetMs)
    guard !onOffsets.isEmpty else {
      return TakeSummary(noteOnCount: 0, meanOffsetMs: 0, stdevOffsetMs: 0)
    }
    let mean: Double = onOffsets.reduce(0, +) / Double(onOffsets.count)
    var squaredDiffSum = 0.0
    for offset in onOffsets {
      let diff = offset - mean
      squaredDiffSum += diff * diff
    }
    let variance = squaredDiffSum / Double(onOffsets.count)
    return TakeSummary(
      noteOnCount: onOffsets.count, meanOffsetMs: mean, stdevOffsetMs: variance.squareRoot())
  }

  static func encodeJSONL(header: TakeHeader, events: [NoteEvent]) throws -> String {
    var lines: [String] = []
    lines.append(String(data: try encoder.encode(header), encoding: .utf8) ?? "")
    for event in events {
      let line = NoteEventLine(event: event)
      lines.append(String(data: try encoder.encode(line), encoding: .utf8) ?? "")
    }
    return lines.joined(separator: "\n") + "\n"
  }

  /// Parses a JSONL take back into (header, events) — used by the round-trip
  /// test and available for offline sanity-checking an exported fixture.
  static func decodeJSONL(_ contents: String) throws -> (header: TakeHeader, events: [NoteEvent]) {
    let lines = contents.split(separator: "\n").map(String.init)
    guard let headerLine = lines.first else {
      throw DecodeError.empty
    }
    let header = try decoder.decode(TakeHeader.self, from: Data(headerLine.utf8))
    let events = try lines.dropFirst().map {
      try decoder.decode(NoteEventLine.self, from: Data($0.utf8)).event
    }
    return (header, events)
  }

  enum DecodeError: Error {
    case empty
  }

  /// Writes the JSONL to `Documents/MidiSpikeTakes/<timestamp>.jsonl` and
  /// returns the file URL for `ShareLink` export.
  static func write(header: TakeHeader, events: [NoteEvent]) throws -> URL {
    let jsonl = try encodeJSONL(header: header, events: events)
    let documentsURL = try FileManager.default.url(
      for: .documentDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
    let takesURL = documentsURL.appendingPathComponent("MidiSpikeTakes", isDirectory: true)
    try FileManager.default.createDirectory(at: takesURL, withIntermediateDirectories: true)

    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    let filename = formatter.string(from: header.recordedAt)
      .replacingOccurrences(of: ":", with: "-")
    let fileURL = takesURL.appendingPathComponent("\(filename).jsonl")

    try jsonl.write(to: fileURL, atomically: true, encoding: .utf8)
    return fileURL
  }
}
