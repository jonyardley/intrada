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

  /// #1184: the last count-in click reports 0, so its dot fills.
  @Test func theLastCountInClickReportsZeroBeatsLeft() {
    var countIns: [Int] = []
    var beats: [Int] = []

    let schedule = ClickEngine.buildSchedule(
      beats: 0..<6, pulse: pulse(countInBeats: 4, clickPattern: [true]),
      onCountIn: { countIns.append($0) }, onBeat: { index, _ in beats.append(index) })
    for entry in schedule { entry.fire() }

    #expect(countIns == [3, 2, 1, 0])
    #expect(beats == [0, 1])
  }

  @Test func aScheduleWithNoCountInIsAllBody() {
    var beats: [Int] = []

    let schedule = ClickEngine.buildSchedule(
      beats: 0..<3, pulse: pulse(countInBeats: 0, clickPattern: [true]),
      onBeat: { index, _ in beats.append(index) })
    for entry in schedule { entry.fire() }

    #expect(beats == [0, 1, 2])
  }

  // ── #1224 · the four authored levels, each over a four-bar phrase ──
  // Placement is audio and no automated check can hear it, so these assert the
  // exact voice of every beat rather than a summary.

  @Test func l1SoundsEveryBeatWithAccentedBarDownbeats() {
    let voices = soundedAndReported(clickPattern: [true, true, true, true], bars: 4)

    #expect(voices == repeated([.accent, .click, .click, .click], 4))
  }

  @Test func l2SoundsBeatsTwoAndFourAndNeverAccents() {
    let voices = soundedAndReported(clickPattern: [false, true, false, true], bars: 4)

    #expect(voices == repeated([.silent, .click, .silent, .click], 4))
    #expect(!voices.contains(.accent), "the accented beat is the one this level silences")
  }

  @Test func l3SoundsTheBarDownbeatAlone() {
    let voices = soundedAndReported(clickPattern: [true, false, false, false], bars: 4)

    #expect(voices == repeated([.accent, .silent, .silent, .silent], 4))
  }

  /// The mask's own length drives the cycle, not `beatsPerBar` — otherwise a
  /// phrase of an odd number of bars flips the alternation on every pass.
  @Test func l4SoundsOneDownbeatEveryOtherBar() {
    let voices = soundedAndReported(
      clickPattern: [true, false, false, false, false, false, false, false], bars: 4)

    #expect(
      voices
        == repeated([.accent, .silent, .silent, .silent, .silent, .silent, .silent, .silent], 2)
    )
    #expect(voices.filter { $0 != .silent }.count == 2, "four bars carry exactly two clicks")
  }

  @Test func aThreeBeatBarKeepsTheAccentOnItsOwnDownbeat() {
    let onTheBeat = soundedAndReported(
      clickPattern: [true, true, true], bars: 4, beatsPerBar: 3)
    #expect(onTheBeat == repeated([.accent, .click, .click], 4))

    let twoAndFour = soundedAndReported(
      clickPattern: [false, true, false], bars: 4, beatsPerBar: 3)
    #expect(twoAndFour == repeated([.silent, .click, .silent], 4))
  }

  /// Spelled out here, and enforced for every level above by `soundedAndReported`.
  @Test func silentBeatsStillReportThemselves() {
    var beats: [Int] = []

    let schedule = ClickEngine.buildSchedule(
      beats: 0..<16,
      pulse: pulse(
        countInBeats: 0, clickPattern: [true, false, false, false, false, false, false, false]),
      onBeat: { index, _ in beats.append(index) })
    for entry in schedule { entry.fire() }

    #expect(schedule.filter { $0.voice == .silent }.count == 14, "fourteen of the sixteen")
    #expect(beats == Array(0..<16), "and all sixteen reach the core regardless")
  }

  // ── The stranded clock (#1223 review) ──

  @Test func aPulseSuspendedForFiveMinutesIsStranded() {
    let secondsPerBeat = 0.5
    let head = HostClock.ticks(fromSeconds: 100)
    let fiveMinutesLater = head &+ HostClock.ticks(fromSeconds: 300)

    #expect(
      ClickEngine.hasLostTheClock(
        head: head, now: fiveMinutesLater, secondsPerBeat: secondsPerBeat))
  }

  /// A coalesced wakeup a beat or so late is what the 10ms poll exists to absorb.
  @Test func aWakeupUpToOneBeatLateIsNotStranded() {
    let secondsPerBeat = 0.5
    let head = HostClock.ticks(fromSeconds: 100)

    let onTime = head
    let slightlyLate = head &+ HostClock.ticks(fromSeconds: 0.2)
    let oneBeatLate = head &+ HostClock.ticks(fromSeconds: 0.5)

    #expect(!ClickEngine.hasLostTheClock(head: head, now: onTime, secondsPerBeat: secondsPerBeat))
    #expect(
      !ClickEngine.hasLostTheClock(head: head, now: slightlyLate, secondsPerBeat: secondsPerBeat))
    #expect(
      !ClickEngine.hasLostTheClock(head: head, now: oneBeatLate, secondsPerBeat: secondsPerBeat))
  }

  /// The case an unsigned subtraction would get catastrophically wrong.
  @Test func aBeatStillAheadIsNeverStranded() {
    let now = HostClock.ticks(fromSeconds: 100)
    let head = now &+ HostClock.ticks(fromSeconds: 30)

    #expect(!ClickEngine.hasLostTheClock(head: head, now: now, secondsPerBeat: 0.5))
  }

  @Test func theStrandedThresholdScalesWithTempoNotWallClock() {
    let head = HostClock.ticks(fromSeconds: 100)
    let threeSecondsLate = head &+ HostClock.ticks(fromSeconds: 3)

    // 240bpm: two beats is half a second, so three seconds is long gone.
    #expect(
      ClickEngine.hasLostTheClock(head: head, now: threeSecondsLate, secondsPerBeat: 0.25))
    // 40bpm: two beats is three seconds, so the same lag is still in tolerance.
    #expect(
      !ClickEngine.hasLostTheClock(head: head, now: threeSecondsLate, secondsPerBeat: 1.5))
  }

  /// `HostClock.ticks` traps on the NaN a zero tempo produces, and a trap is a
  /// crash rather than something `unavailable()` can route around. This is the
  /// one test in the file that genuinely needs a real engine instance, since
  /// it is `start()`'s guard being exercised, not `buildSchedule`.
  @Test func aNonPositiveTempoThrowsRatherThanCrashing() throws {
    let engine = try ClickEngine()

    #expect(throws: ClickEngine.ClickEngineError.nonPositiveTempo) {
      try engine.start(bpm: 0, beatsPerBar: 4, countInBeats: 4, clickPattern: [true])
    }
    #expect(throws: ClickEngine.ClickEngineError.nonPositiveTempo) {
      try engine.start(bpm: -120, beatsPerBar: 4, countInBeats: 4, clickPattern: [true])
    }
  }

  /// Asserts every beat reported before returning the voices, so no level can
  /// pass its placement assertion while quietly swallowing beats.
  private func soundedAndReported(clickPattern: [Bool], bars: Int, beatsPerBar: Int = 4)
    -> [ClickEngine.Voice]
  {
    var beats: [Int] = []

    let total = bars * beatsPerBar
    let schedule = ClickEngine.buildSchedule(
      beats: 0..<total,
      pulse: pulse(countInBeats: 0, clickPattern: clickPattern, beatsPerBar: beatsPerBar),
      onBeat: { index, _ in beats.append(index) })
    for entry in schedule { entry.fire() }

    #expect(beats == Array(0..<total), "every beat reports, sounded or silent")
    return schedule.map(\.voice)
  }

  /// The count-in is the one place placement never applies: it clicks every
  /// beat whatever the level, or there is nothing to come in against.
  @Test func theCountInClicksEveryBeatWhateverTheLevel() {
    let schedule = ClickEngine.buildSchedule(
      beats: 0..<6, pulse: pulse(countInBeats: 4, clickPattern: [true, false, false, false]))

    #expect(schedule.prefix(4).allSatisfy { $0.voice == .countIn })
    #expect(Array(schedule.dropFirst(4)).map(\.voice) == [.accent, .silent])
  }

  /// A window past the first keeps the placement aligned to the pulse's first
  /// body beat, so topping up the rolling schedule can't shift where clicks land.
  @Test func alaterWindowKeepsThePlacementAligned() {
    let schedule = ClickEngine.buildSchedule(
      beats: 64..<68, pulse: pulse(countInBeats: 4, clickPattern: [false, true, false, true]))

    // Body beat 60 onwards, and 60 % 4 == 0, so the cycle starts where it did.
    #expect(schedule.map(\.voice) == [.silent, .click, .silent, .click])
  }

  /// Beats do not restart at a rep boundary any more, so a later window keeps
  /// counting on — the core derives the rep from `beat_index % phrase_beats`.
  @Test func beatIndicesCountOnAcrossWindows() {
    var beats: [Int] = []

    let schedule = ClickEngine.buildSchedule(
      beats: 64..<67, pulse: pulse(countInBeats: 4, clickPattern: [true]),
      onBeat: { index, _ in beats.append(index) })
    for entry in schedule { entry.fire() }

    #expect(beats == [60, 61, 62])
  }
}
