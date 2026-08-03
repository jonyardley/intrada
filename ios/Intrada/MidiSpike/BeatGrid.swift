import Foundation

/// The logical click grid, defined purely in host-time ticks. `startHostTime`
/// is bar 1 beat 1 — i.e. the first beat *after* the count-in, not engine
/// start. Bars and beats are 1-indexed to match how a musician counts.
struct BeatGrid {
  let bpm: Double
  let beatsPerBar: Int
  let countInBeats: Int
  let startHostTime: UInt64

  var secondsPerBeat: Double { 60.0 / bpm }

  func hostTime(bar: Int, beat: Int) -> UInt64 {
    let beatsFromStart = (bar - 1) * beatsPerBar + (beat - 1)
    let offsetSeconds = Double(beatsFromStart) * secondsPerBeat
    return startHostTime &+ HostClock.ticks(fromSeconds: offsetSeconds)
  }

  /// Nearest (bar, beat) to `hostTime`, plus the signed offset in
  /// milliseconds (positive = late, negative = early). Bar/beat are clamped
  /// to 1 if `hostTime` falls before the grid start (e.g. during count-in).
  func nearestBeat(for hostTime: UInt64) -> (bar: Int, beat: Int, offsetMs: Double) {
    let elapsedSeconds = HostClock.secondsBetween(hostTime, startHostTime)
    let beatsFromStart = elapsedSeconds / secondsPerBeat
    let nearestBeatIndex = max(0, Int(beatsFromStart.rounded()))

    let bar = nearestBeatIndex / beatsPerBar + 1
    let beat = nearestBeatIndex % beatsPerBar + 1

    let nearestBeatSeconds = Double(nearestBeatIndex) * secondsPerBeat
    let offsetSeconds = elapsedSeconds - nearestBeatSeconds
    return (bar, beat, offsetSeconds * 1000)
  }
}
