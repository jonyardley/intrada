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

  // ── #1224 · the four authored levels, each over a four-bar phrase ──
  //
  // These carry more weight than usual: the sparse click is audio, and it is
  // merging without an ear on it, so this is the only thing between a wrong
  // mask and a shipped one. Each level asserts the exact voice of all sixteen
  // beats, not a summary, and that every one of the sixteen still reports.

  /// l1 — every beat, bar downbeats accented.
  @Test func l1SoundsEveryBeat() throws {
    let voices = try soundedAndReported(clickPattern: [true, true, true, true], bars: 4)

    #expect(voices == repeated([.accent, .click, .click, .click], 4))
  }

  /// l2 — beats 2 and 4, so the bar's downbeat is silent and no beat is accented.
  @Test func l2SoundsBeatsTwoAndFourOnly() throws {
    let voices = try soundedAndReported(clickPattern: [false, true, false, true], bars: 4)

    #expect(voices == repeated([.silent, .click, .silent, .click], 4))
    #expect(!voices.contains(.accent), "the accented beat is the one this level silences")
  }

  /// l3 — the bar's downbeat alone, which is also the accent.
  @Test func l3SoundsTheBarDownbeatOnly() throws {
    let voices = try soundedAndReported(clickPattern: [true, false, false, false], bars: 4)

    #expect(voices == repeated([.accent, .silent, .silent, .silent], 4))
  }

  /// l4 — one click every other bar. The cycle is two bars, which is why the
  /// mask's own length drives it and not `beatsPerBar`; a phrase of an odd
  /// number of bars would otherwise flip the alternation on every pass.
  @Test func l4SoundsEveryOtherBarsDownbeat() throws {
    let voices = try soundedAndReported(
      clickPattern: [true, false, false, false, false, false, false, false], bars: 4)

    #expect(
      voices
        == repeated([.accent, .silent, .silent, .silent, .silent, .silent, .silent, .silent], 2)
    )
    #expect(voices.filter { $0 != .silent }.count == 2, "four bars carry exactly two clicks")
  }

  /// A 3/4 bar: `TwoAndFour` drops its beat 4, and the accent must still land on
  /// the bar line rather than every fourth beat.
  @Test func aThreeBeatBarKeepsTheAccentOnItsOwnDownbeat() throws {
    let onTheBeat = try soundedAndReported(
      clickPattern: [true, true, true], bars: 4, beatsPerBar: 3)
    #expect(onTheBeat == repeated([.accent, .click, .click], 4))

    let twoAndFour = try soundedAndReported(
      clickPattern: [false, true, false], bars: 4, beatsPerBar: 3)
    #expect(twoAndFour == repeated([.silent, .click, .silent], 4))
  }

  /// The one that would break bar and rep tracking silently: a beat the level
  /// silences still reports itself, so only the audio goes quiet. Asserted for
  /// every level above via `soundedAndReported`, and spelled out here.
  @Test func silentBeatsStillReportThemselves() throws {
    let engine = try ClickEngine()
    var beats: [Int] = []
    engine.onBeat = { index, _ in beats.append(index) }

    let schedule = engine.buildSchedule(
      beats: 0..<16,
      pulse: pulse(
        countInBeats: 0, clickPattern: [true, false, false, false, false, false, false, false]))
    for entry in schedule { entry.fire() }

    #expect(schedule.filter { $0.voice == .silent }.count == 14, "fourteen of the sixteen")
    #expect(beats == Array(0..<16), "and all sixteen reach the core regardless")
  }

  /// Builds a phrase at one level and asserts, before returning the voices, that
  /// every beat in it reported itself — so no level can pass its placement
  /// assertion while quietly swallowing beats.
  private func soundedAndReported(clickPattern: [Bool], bars: Int, beatsPerBar: Int = 4) throws
    -> [ClickEngine.Voice]
  {
    let engine = try ClickEngine()
    var beats: [Int] = []
    engine.onBeat = { index, _ in beats.append(index) }

    let total = bars * beatsPerBar
    let schedule = engine.buildSchedule(
      beats: 0..<total,
      pulse: pulse(countInBeats: 0, clickPattern: clickPattern, beatsPerBar: beatsPerBar))
    for entry in schedule { entry.fire() }

    #expect(beats == Array(0..<total), "every beat reports, sounded or silent")
    return schedule.map(\.voice)
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
