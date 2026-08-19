import Testing

@testable import Intrada

@MainActor
struct ClickEngineTests {
  private func pulse(bpm: Double, latencySeconds: Double = 0) -> ClickEngine.Pulse {
    ClickEngine.Pulse(
      bpm: bpm, scheduledStart: 0,
      outputLatencyTicks: HostClock.ticks(fromSeconds: latencySeconds))
  }

  @Test func everyBeatSitsOnTheGridStruckFromTheStart() {
    let beats = ClickEngine.schedule(beats: 0..<9, pulse: pulse(bpm: 120))

    for (index, beat) in beats.enumerated() {
      let seconds = HostClock.seconds(fromTicks: beat.hostTime)
      #expect(abs(seconds - Double(index) * 0.5) < 0.000_001)
    }
  }

  /// The window is topped up thousands of beats into a session, so beat N is
  /// N × the period from the *start*, never one period past its predecessor —
  /// per-beat accumulation would round its way off the grid.
  @Test func aBeatThousandsInIsStillOnTheOriginalGrid() {
    let secondsPerBeat = 60.0 / 132.0
    let beats = ClickEngine.schedule(beats: 5000..<5001, pulse: pulse(bpm: 132))

    let seconds = HostClock.seconds(fromTicks: beats[0].hostTime)
    #expect(abs(seconds - 5000 * secondsPerBeat) < 0.000_001)
  }

  @Test func aLaterWindowPicksUpExactlyOnePeriodAfterTheOneBefore() {
    let first = ClickEngine.schedule(beats: 0..<64, pulse: pulse(bpm: 120))
    let second = ClickEngine.schedule(beats: 64..<128, pulse: pulse(bpm: 120))

    let gap = HostClock.secondsBetween(second[0].hostTime, first[63].hostTime)
    #expect(abs(gap - 0.5) < 0.000_001)
  }

  @Test func aBeatIsHeardOneOutputLatencyAfterItIsScheduled() {
    let latency = HostClock.ticks(fromSeconds: 0.012)
    let beats = ClickEngine.schedule(beats: 0..<4, pulse: pulse(bpm: 90, latencySeconds: 0.012))

    for beat in beats {
      #expect(beat.audibleHostTime == beat.hostTime &+ latency)
    }
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
  /// crash rather than something the caller can route around. The one test here
  /// that genuinely needs a real engine, since it exercises `start()`'s guard.
  @Test func aNonPositiveTempoThrowsRatherThanCrashing() throws {
    let engine = try ClickEngine()

    #expect(throws: ClickEngine.ClickEngineError.nonPositiveTempo) {
      try engine.start(bpm: 0)
    }
    #expect(throws: ClickEngine.ClickEngineError.nonPositiveTempo) {
      try engine.start(bpm: -120)
    }
    engine.dispose()
  }
}
