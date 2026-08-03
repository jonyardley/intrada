import Testing

@testable import Intrada

struct GatePhraseTests {
  // Large, non-zero start — mirrors a real mach_absolute_time value so an
  // "early" shift subtracts without wrapping (unlike a startHostTime of 0).
  private let grid = BeatGrid(
    bpm: GatePhrase.bpm, beatsPerBar: GatePhrase.beatsPerBar,
    countInBeats: GatePhrase.countInBeats, startHostTime: 10_000_000_000)

  /// One note-on per expected beat, offset by `shiftSeconds` from the grid
  /// (0 = exactly on time), using the lowest octave of each pitch class.
  private func notes(shiftSeconds: Double = 0) -> [NoteEvent] {
    GatePhrase.expected.map { expectedBeat in
      let onGrid = grid.hostTime(bar: expectedBeat.bar, beat: expectedBeat.beat)
      let shiftTicks = HostClock.ticks(fromSeconds: abs(shiftSeconds))
      let hostTime = shiftSeconds >= 0 ? onGrid &+ shiftTicks : onGrid &- shiftTicks
      let pitchClass = expectedBeat.pitchClasses.first!
      return NoteEvent(
        hostTime: hostTime, midiNote: UInt8(60 + pitchClass), velocity: 80, isNoteOn: true)
    }
  }

  @Test func cleanPlayThroughIsAPass() {
    let result = GatePhrase.evaluate(notes: notes(), against: grid, transport: .usb)
    #expect(result.verdict == .pass)
    #expect(result.fact == "clean")
  }

  @Test func consistentlyLateIsFlaggedLateNeverBareWrong() {
    let result = GatePhrase.evaluate(
      notes: notes(shiftSeconds: 0.1), against: grid, transport: .usb)
    #expect(result.verdict == .late)
    #expect(result.fact == "dragging")
  }

  @Test func consistentlyEarlyIsFlaggedEarly() {
    let result = GatePhrase.evaluate(
      notes: notes(shiftSeconds: -0.1), against: grid, transport: .usb)
    #expect(result.verdict == .early)
    #expect(result.fact == "rushing")
  }

  @Test func withinToleranceIsStillAPass() {
    // 40ms is inside the 80ms tolerance.
    let result = GatePhrase.evaluate(
      notes: notes(shiftSeconds: 0.04), against: grid, transport: .usb)
    #expect(result.verdict == .pass)
  }

  @Test func missingNoteIsWrongNotes() {
    let missingOne = Array(notes().dropLast())
    let result = GatePhrase.evaluate(notes: missingOne, against: grid, transport: .usb)
    #expect(result.verdict == .wrongNotes(count: 1))
    #expect(result.fact == "1 wrong note")
  }

  @Test func extraNoteIsWrongNotes() {
    let extra = notes() + [NoteEvent(hostTime: 0, midiNote: 64, velocity: 80, isNoteOn: true)]
    let result = GatePhrase.evaluate(notes: extra, against: grid, transport: .usb)
    #expect(result.verdict == .wrongNotes(count: 1))
  }

  @Test func emptyRepIsAllWrongNotes() {
    let result = GatePhrase.evaluate(notes: [], against: grid, transport: .usb)
    #expect(result.verdict == .wrongNotes(count: GatePhrase.expected.count))
    #expect(result.fact == "\(GatePhrase.expected.count) wrong notes")
  }

  // Spec decision 7 (transport-tiered scoring): Bluetooth never earns a fine
  // early/late verdict, regardless of measured offset — only note accuracy.

  @Test func bluetoothSuppressesLateVerdictEvenWhenOffsetExceedsTolerance() {
    let result = GatePhrase.evaluate(
      notes: notes(shiftSeconds: 0.1), against: grid, transport: .bluetooth)
    #expect(result.verdict == .pass)
    #expect(result.fact == "clean")
  }

  @Test func bluetoothSuppressesEarlyVerdictEvenWhenOffsetExceedsTolerance() {
    let result = GatePhrase.evaluate(
      notes: notes(shiftSeconds: -0.1), against: grid, transport: .bluetooth)
    #expect(result.verdict == .pass)
  }

  @Test func bluetoothStillCatchesWrongNotes() {
    let missingOne = Array(notes().dropLast())
    let result = GatePhrase.evaluate(notes: missingOne, against: grid, transport: .bluetooth)
    #expect(result.verdict == .wrongNotes(count: 1))
  }

  @Test func noteOffEventsAreIgnored() {
    let noteOffsOnly = notes().map { note -> NoteEvent in
      var copy = note
      copy = NoteEvent(
        hostTime: note.hostTime, midiNote: note.midiNote, velocity: 0, isNoteOn: false)
      return copy
    }
    let result = GatePhrase.evaluate(notes: noteOffsOnly, against: grid, transport: .usb)
    #expect(result.verdict == .wrongNotes(count: GatePhrase.expected.count))
  }
}
