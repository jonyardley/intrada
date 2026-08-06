import Testing

@testable import Intrada

@MainActor
struct ClickEngineTests {
  private func pulse(countInBeats: Int, clickPattern: [Bool], beatsPerBar: Int = 4)
    -> ClickEngine.Pulse
  {
    ClickEngine.Pulse(
      bpm: 120, beatsPerBar: beatsPerBar, countInBeats: countInBeats,
      clickPattern: clickPattern, scheduledStart: 0, outputLatencyTicks: 0)
  }

  private func repeated(_ pattern: [ClickEngine.Voice], _ times: Int) -> [ClickEngine.Voice] {
    Array(repeating: pattern, count: times).flatMap { $0 }
  }

  /// #1184: the last count-in click must report 0, so its dot fills.
  @Test func countInReportsBeatsLeftAfterEachClick() throws {
    let engine = try ClickEngine()
    var countIns: [Int] = []
    var beats: [Int] = []
    engine.onCountIn = { countIns.append($0) }
    engine.onBeat = { index, _ in beats.append(index) }

    let schedule = engine.buildSchedule(
      beats: 0..<6, pulse: pulse(countInBeats: 4, clickPattern: [true]))
    for entry in schedule { entry.fire() }

    #expect(countIns == [3, 2, 1, 0])
    #expect(beats == [0, 1])
  }

  @Test func aScheduleWithNoCountInIsAllBody() throws {
    let engine = try ClickEngine()
    var countIns: [Int] = []
    var beats: [Int] = []
    engine.onCountIn = { countIns.append($0) }
    engine.onBeat = { index, _ in beats.append(index) }

    let schedule = engine.buildSchedule(
      beats: 0..<3, pulse: pulse(countInBeats: 0, clickPattern: [true]))
    for entry in schedule { entry.fire() }

    #expect(countIns.isEmpty)
    #expect(beats == [0, 1, 2])
  }

  /// #1224 l1: every beat sounds, bar downbeats accented.
  @Test func everyBeatSoundsWithAccentedDownbeats() throws {
    let engine = try ClickEngine()
    let schedule = engine.buildSchedule(
      beats: 0..<8, pulse: pulse(countInBeats: 0, clickPattern: [true, true, true, true]))

    #expect(schedule.map(\.voice) == repeated([.accent, .click, .click, .click], 2))
  }

  /// #1224 l2: beats 2 and 4 of the bar, so the downbeat is silent.
  @Test func twoAndFourSoundsOnlyTheOffbeats() throws {
    let engine = try ClickEngine()
    let schedule = engine.buildSchedule(
      beats: 0..<8, pulse: pulse(countInBeats: 0, clickPattern: [false, true, false, true]))

    #expect(schedule.map(\.voice) == repeated([.silent, .click, .silent, .click], 2))
  }

  /// #1224 l3: one click a bar. #1224 l4: one every other bar, which is why the
  /// cycle is the pattern's own length and not the bar's.
  @Test func sparseLevelsSoundOnlyTheirOwnBeats() throws {
    let engine = try ClickEngine()
    let barDownbeat = engine.buildSchedule(
      beats: 0..<8, pulse: pulse(countInBeats: 0, clickPattern: [true, false, false, false]))
    #expect(barDownbeat.map(\.voice) == repeated([.accent, .silent, .silent, .silent], 2))

    let everyOtherBar = engine.buildSchedule(
      beats: 0..<16,
      pulse: pulse(
        countInBeats: 0,
        clickPattern: [true, false, false, false, false, false, false, false]))
    #expect(everyOtherBar.map(\.voice).filter { $0 != .silent } == [.accent, .accent])
    #expect(everyOtherBar[0].voice == .accent)
    #expect(everyOtherBar[8].voice == .accent)
  }

  /// The one that would break bar and rep tracking silently: a beat the level
  /// silences still reports itself, so only the audio goes quiet.
  @Test func silentBeatsStillReportThemselves() throws {
    let engine = try ClickEngine()
    var beats: [Int] = []
    engine.onBeat = { index, _ in beats.append(index) }

    let schedule = engine.buildSchedule(
      beats: 0..<8, pulse: pulse(countInBeats: 0, clickPattern: [false, true, false, true]))
    for entry in schedule { entry.fire() }

    #expect(beats == [0, 1, 2, 3, 4, 5, 6, 7])
  }

  /// The count-in is the one place placement never applies: it clicks every
  /// beat whatever the level, or there is nothing to come in against.
  @Test func theCountInClicksEveryBeatWhateverTheLevel() throws {
    let engine = try ClickEngine()
    let schedule = engine.buildSchedule(
      beats: 0..<6, pulse: pulse(countInBeats: 4, clickPattern: [true, false, false, false]))

    #expect(schedule.prefix(4).allSatisfy { $0.voice == .countIn })
    #expect(Array(schedule.dropFirst(4)).map(\.voice) == [.accent, .silent])
  }

  /// A window past the first keeps the placement aligned to the pulse's first
  /// body beat, so topping up the rolling schedule can't shift where clicks land.
  @Test func alaterWindowKeepsThePlacementAligned() throws {
    let engine = try ClickEngine()
    let schedule = engine.buildSchedule(
      beats: 64..<68, pulse: pulse(countInBeats: 4, clickPattern: [false, true, false, true]))

    // Body beat 60 onwards, and 60 % 4 == 0, so the cycle starts where it did.
    #expect(schedule.map(\.voice) == [.silent, .click, .silent, .click])
  }

  /// Beats do not restart at a rep boundary any more, so a later window keeps
  /// counting on — the core derives the rep from `beat_index % phrase_beats`.
  @Test func beatIndicesCountOnAcrossWindows() throws {
    let engine = try ClickEngine()
    var beats: [Int] = []
    engine.onBeat = { index, _ in beats.append(index) }

    let schedule = engine.buildSchedule(
      beats: 64..<67, pulse: pulse(countInBeats: 4, clickPattern: [true]))
    for entry in schedule { entry.fire() }

    #expect(beats == [60, 61, 62])
  }
}
