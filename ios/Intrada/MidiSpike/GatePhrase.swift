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

  /// Any octave matches — this is display text for "what to play", not a
  /// scoring detail.
  var noteNames: String {
    pitchClasses.sorted().map { Self.pitchClassNames[$0] }.joined(separator: "/")
  }
}

enum RepVerdict: Equatable {
  case pass
  case early
  case late
  case wrongNotes(count: Int)
}

struct RepResult: Equatable {
  let verdict: RepVerdict

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

/// A hard-coded 8-note single-line phrase (drawn from the Dm7 and G7 shells
/// in C, but played as a melody, not a chord): F C F C, then B F B F,
/// crotchets, 80 bpm. Trivially edited constants — this is deliberately
/// crude, not a scoring engine.
enum GatePhrase {
  static let bpm: Double = 80
  static let beatsPerBar = 4
  static let countInBeats = 4
  static let gateTargetPasses = 3
  static let timingToleranceMs: Double = 80

  // The Dm7 and G7 shells (3rd+7th) are arpeggiated one note at a time, not
  // played as two-note chords — each beat below is a single fresh keystroke
  // (re-struck, not sustained from the previous beat), one hand, any octave.
  // Pitch classes: Dm7 shell is F(5)/C(0), G7 shell is B(11)/F(5) — see
  // `noteNames` for the name mapping.
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

  /// "1. F   2. C   3. F   4. C   5. B   6. F   7. B   8. F" — a single
  /// ordered sequence, one note at a time, any octave, each struck fresh on
  /// its beat (never a chord, never held). The numbering is deliberate: it's
  /// the clearest way to say "in this order, one after another."
  static var displaySequence: String {
    expected.enumerated().map { index, beat in "\(index + 1). \(beat.noteNames)" }
      .joined(separator: "   ")
  }

  /// Classifies one rep's captured note-on events against `expected`.
  /// Matches each expected beat to the *nearest-in-time* unmatched note-on
  /// of the right pitch class (not gated by `timingToleranceMs` at match
  /// time — the tolerance only decides the early/late verdict afterward, on
  /// whichever match came closest to any target beat). Notes with no
  /// matching expected beat, or expected beats with no matching note, count
  /// toward "wrong notes".
  ///
  /// `transport` gates whether a timing verdict is even possible — spec
  /// decision 7 (transport-tiered scoring): Bluetooth's connection-interval
  /// jitter (±10-20ms) doesn't support a fine early/late verdict, so a
  /// Bluetooth rep only ever resolves to pass or wrong-notes, never early/late.
  /// This is a per-rep simplification for the spike, not the final scoring
  /// model — spec decision 6 wants *consistency across reps*, not one rep's
  /// absolute deviation, graded; that needs a rep-history model this spike
  /// doesn't have.
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
      // Bluetooth: note accuracy only, no fine timing verdict — see the
      // doc comment above.
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
