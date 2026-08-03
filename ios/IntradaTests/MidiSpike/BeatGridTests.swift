import Testing

@testable import Intrada

struct BeatGridTests {
  private func makeGrid(bpm: Double = 120, beatsPerBar: Int = 4, startHostTime: UInt64 = 0)
    -> BeatGrid
  {
    BeatGrid(bpm: bpm, beatsPerBar: beatsPerBar, countInBeats: 4, startHostTime: startHostTime)
  }

  @Test func hostTimeForBar1Beat1MatchesGridStart() {
    let grid = makeGrid(startHostTime: 1_000_000)
    #expect(grid.hostTime(bar: 1, beat: 1) == 1_000_000)
  }

  @Test func hostTimeAdvancesOneBeatAtATime() {
    let grid = makeGrid(bpm: 120, startHostTime: 0)  // 500ms per beat at 120bpm
    let beat2 = grid.hostTime(bar: 1, beat: 2)
    let expectedTicks = HostClock.ticks(fromSeconds: 0.5)
    #expect(beat2 == expectedTicks)
  }

  @Test func hostTimeRollsOverToNextBar() {
    let grid = makeGrid(bpm: 120, beatsPerBar: 4, startHostTime: 0)
    let bar2Beat1 = grid.hostTime(bar: 2, beat: 1)
    let expectedTicks = HostClock.ticks(fromSeconds: 4 * 0.5)  // 4 beats @ 500ms
    #expect(bar2Beat1 == expectedTicks)
  }

  @Test func nearestBeatOnGridIsZeroOffset() {
    let grid = makeGrid(bpm: 120, startHostTime: 0)
    let target = grid.hostTime(bar: 1, beat: 3)
    let (bar, beat, offsetMs) = grid.nearestBeat(for: target)
    #expect(bar == 1)
    #expect(beat == 3)
    #expect(abs(offsetMs) < 0.01)
  }

  @Test func nearestBeatDetectsLateNote() {
    let grid = makeGrid(bpm: 120, startHostTime: 0)
    let onGrid = grid.hostTime(bar: 1, beat: 2)
    let lateHostTime = onGrid &+ HostClock.ticks(fromSeconds: 0.03)  // 30ms late
    let (bar, beat, offsetMs) = grid.nearestBeat(for: lateHostTime)
    #expect(bar == 1)
    #expect(beat == 2)
    #expect(offsetMs > 25 && offsetMs < 35)
  }

  @Test func nearestBeatDetectsEarlyNote() {
    let grid = makeGrid(bpm: 120, startHostTime: 0)
    let onGrid = grid.hostTime(bar: 1, beat: 3)
    let earlyHostTime = onGrid &- HostClock.ticks(fromSeconds: 0.02)  // 20ms early
    let (bar, beat, offsetMs) = grid.nearestBeat(for: earlyHostTime)
    #expect(bar == 1)
    #expect(beat == 3)
    #expect(offsetMs < -15 && offsetMs > -25)
  }

  @Test func nearestBeatBeforeGridStartClampsToBarOneBeatOne() {
    let grid = makeGrid(bpm: 120, startHostTime: 1_000_000_000)
    let (bar, beat, _) = grid.nearestBeat(for: 0)
    #expect(bar == 1)
    #expect(beat == 1)
  }
}
