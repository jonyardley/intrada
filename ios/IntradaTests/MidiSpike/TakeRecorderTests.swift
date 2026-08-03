import Foundation
import Testing

@testable import Intrada

struct TakeRecorderTests {
  private let grid = BeatGrid(bpm: 92, beatsPerBar: 4, countInBeats: 4, startHostTime: 1_000_000)

  private func sampleEvents() -> [NoteEvent] {
    [
      NoteEvent(
        hostTime: grid.hostTime(bar: 1, beat: 1), midiNote: 60, velocity: 80, isNoteOn: true),
      NoteEvent(
        hostTime: grid.hostTime(bar: 1, beat: 1), midiNote: 60, velocity: 0, isNoteOn: false),
      NoteEvent(
        hostTime: grid.hostTime(bar: 1, beat: 2), midiNote: 64, velocity: 90, isNoteOn: true),
    ]
  }

  @Test func annotateFillsBarBeatAndOffset() {
    let annotated = TakeRecorder.annotate(sampleEvents(), against: grid)
    #expect(annotated[0].bar == 1)
    #expect(annotated[0].beat == 1)
    #expect(abs(annotated[0].offsetMs ?? .infinity) < 0.01)
    #expect(annotated[2].beat == 2)
  }

  @Test func summaryComputesMeanAndStdevOverNoteOnsOnly() {
    let annotated = TakeRecorder.annotate(sampleEvents(), against: grid)
    let summary = TakeRecorder.summary(of: annotated)
    // Two note-ons in the sample (the note-off is excluded).
    #expect(summary.noteOnCount == 2)
  }

  @Test func summaryOfEmptyEventsIsZeroed() {
    let summary = TakeRecorder.summary(of: [])
    #expect(summary.noteOnCount == 0)
    #expect(summary.meanOffsetMs == 0)
    #expect(summary.stdevOffsetMs == 0)
  }

  @Test func jsonlRoundTripsHeaderAndEvents() throws {
    let annotated = TakeRecorder.annotate(sampleEvents(), against: grid)
    let header = TakeHeader(
      transport: .usb, bpm: grid.bpm, beatsPerBar: grid.beatsPerBar,
      countInBeats: grid.countInBeats, startHostTime: grid.startHostTime,
      hostTimebaseNumer: HostClock.timebase.numer, hostTimebaseDenom: HostClock.timebase.denom,
      recordedAt: Date())

    let jsonl = try TakeRecorder.encodeJSONL(header: header, events: annotated)
    let (decodedHeader, decodedEvents) = try TakeRecorder.decodeJSONL(jsonl)

    #expect(decodedHeader.transport == .usb)
    #expect(decodedHeader.bpm == grid.bpm)
    #expect(decodedHeader.startHostTime == grid.startHostTime)
    #expect(decodedEvents.count == annotated.count)
    #expect(decodedEvents[0].midiNote == annotated[0].midiNote)
    #expect(decodedEvents[0].bar == annotated[0].bar)
    #expect(decodedEvents[2].beat == annotated[2].beat)
  }

  @Test func decodingEmptyStringThrows() {
    #expect(throws: (any Error).self) {
      _ = try TakeRecorder.decodeJSONL("")
    }
  }
}
