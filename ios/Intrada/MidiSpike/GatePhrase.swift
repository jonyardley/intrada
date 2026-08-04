import Foundation

/// One expected beat in the hard-coded drill phrase — matched by pitch class
/// only (octave-tolerant, crude on purpose; this is a spike, not a scorer).
struct ExpectedBeat {
  let bar: Int
  let beat: Int
  let pitchClasses: Set<Int>

  private static let pitchClassNames = [
    "C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B",
  ]

  /// Display text ("what to play") — any octave matches for scoring.
  var noteNames: String {
    pitchClasses.sorted().map { Self.pitchClassNames[$0] }.joined(separator: "/")
  }
}

/// The *machine* verdict from the deferred scoring path (decision 18) — not the
/// shipped `RepVerdict` primitive, which draws the user's own tap-verdict.
enum CapturedVerdict: Equatable {
  case pass
  case early
  case late
  case wrongNotes(count: Int)
}

struct RepResult: Equatable {
  let verdict: CapturedVerdict

  var isPass: Bool {
    if case .pass = verdict { return true }
    return false
  }

  /// The single glanceable fact per Layer 1 of the feedback choreography —
  /// timing facts say early/late, never bare "wrong".
  var fact: String {
    switch verdict {
    case .pass: return "clean"
    case .early: return "rushing"
    case .late: return "dragging"
    case .wrongNotes(let count): return count == 1 ? "1 wrong note" : "\(count) wrong notes"
    }
  }
}

/// Hard-coded 8-note phrase: F C F C, then B F B F, crotchets, 80 bpm.
/// Trivially edited constants — deliberately crude, not a scoring engine.
enum GatePhrase {
  static let bpm: Double = 80
  static let beatsPerBar = 4
  static let countInBeats = 4
  static let gateTargetPasses = 3
  static let timingToleranceMs: Double = 80

  // Dm7 shell (F=5/C=0) then G7 shell (B=11/F=5), arpeggiated one fresh
  // keystroke per beat — a melody, not a chord.
  static let expected: [ExpectedBeat] = [
    ExpectedBeat(bar: 1, beat: 1, pitchClasses: [5]),
    ExpectedBeat(bar: 1, beat: 2, pitchClasses: [0]),
    ExpectedBeat(bar: 1, beat: 3, pitchClasses: [5]),
    ExpectedBeat(bar: 1, beat: 4, pitchClasses: [0]),
    ExpectedBeat(bar: 2, beat: 1, pitchClasses: [11]),
    ExpectedBeat(bar: 2, beat: 2, pitchClasses: [5]),
    ExpectedBeat(bar: 2, beat: 3, pitchClasses: [11]),
    ExpectedBeat(bar: 2, beat: 4, pitchClasses: [5]),
  ]

  /// "1. F   2. C   3. F   4. C   5. B   6. F   7. B   8. F" — numbered to
  /// make "one at a time, in this order" unambiguous on screen.
  static var displaySequence: String {
    expected.enumerated().map { index, beat in "\(index + 1). \(beat.noteNames)" }
      .joined(separator: "   ")
  }

  /// Matches each expected beat to its nearest-in-time unmatched note-on of
  /// the right pitch class; unmatched beats/notes count as "wrong notes".
  /// `transport` == .bluetooth suppresses early/late (spec decision 7: BLE
  /// jitter can't support a fine verdict) — note this is per-rep, not
  /// decision 6's cross-rep consistency, which needs history this spike lacks.
  static func evaluate(notes: [NoteEvent], against grid: BeatGrid, transport: TransportType)
    -> RepResult
  {
    let noteOns = notes.filter(\.isNoteOn)
    var unmatchedNotes = noteOns
    var worstOffsetMs = 0.0
    var mismatchCount = 0

    for expectedBeat in expected {
      let targetHostTime = grid.hostTime(bar: expectedBeat.bar, beat: expectedBeat.beat)

      var bestIndex: Int?
      var bestDistanceSeconds = Double.infinity
      for index in unmatchedNotes.indices {
        let note = unmatchedNotes[index]
        guard expectedBeat.pitchClasses.contains(Int(note.midiNote) % 12) else { continue }
        let distanceSeconds = abs(HostClock.secondsBetween(note.hostTime, targetHostTime))
        if distanceSeconds < bestDistanceSeconds {
          bestDistanceSeconds = distanceSeconds
          bestIndex = index
        }
      }

      guard let index = bestIndex else {
        mismatchCount += 1
        continue
      }

      let note = unmatchedNotes.remove(at: index)
      let offsetMs = HostClock.secondsBetween(note.hostTime, targetHostTime) * 1000
      if abs(offsetMs) > abs(worstOffsetMs) {
        worstOffsetMs = offsetMs
      }
    }

    mismatchCount += unmatchedNotes.count

    if mismatchCount > 0 {
      return RepResult(verdict: .wrongNotes(count: mismatchCount))
    }
    guard transport == .usb else {
      return RepResult(verdict: .pass)
    }
    if worstOffsetMs > timingToleranceMs {
      return RepResult(verdict: .late)
    }
    if worstOffsetMs < -timingToleranceMs {
      return RepResult(verdict: .early)
    }
    return RepResult(verdict: .pass)
  }
}
