import Testing

@testable import Intrada

@MainActor
struct ClickEngineTests {
  /// #1184: the last count-in click must report 0, so its dot fills.
  @Test func countInReportsBeatsLeftAfterEachClick() throws {
    let engine = try ClickEngine()
    var countIns: [Int] = []
    var beats: [Int] = []
    engine.onCountIn = { countIns.append($0) }
    engine.onBeat = { index, _ in beats.append(index) }

    let schedule = engine.buildSchedule(
      totalBeats: 6, countInBeats: 4, audibleStart: 0, secondsPerBeat: 0.5)
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
      totalBeats: 3, countInBeats: 0, audibleStart: 0, secondsPerBeat: 0.5)
    for entry in schedule { entry.fire() }

    #expect(countIns.isEmpty)
    #expect(beats == [0, 1, 2])
  }
}
