import AVFoundation

/// AVAudioEngine metronome scheduled via AVAudioTime(hostTime:) — the same
/// mach_absolute_time clock CoreMIDI stamps with. `start` returns a
/// `BeatGrid` whose `startHostTime` is the *audible* instant of bar 1 beat 1
/// (scheduled time + AVAudioSession.outputLatency), since that's what the
/// player actually reacts to.
@MainActor
final class ClickEngine {
  private let engine = AVAudioEngine()
  private let playerNode = AVAudioPlayerNode()
  private let countInBuffer: AVAudioPCMBuffer
  private let clickBuffer: AVAudioPCMBuffer
  private let accentBuffer: AVAudioPCMBuffer

  /// Lead time before the count-in starts, to give the engine time to spin
  /// up before the first scheduled buffer's host time arrives.
  private let leadInSeconds: Double = 0.5

  /// Fires once per count-in beat, with beats remaining (counting down to
  /// 1) — the count-in is otherwise indistinguishable from the click that
  /// follows it, both audibly (same instrument) and on screen.
  var onCountIn: ((_ remaining: Int) -> Void)?

  /// Fires just after each *body* beat's audible host time (count-in beats
  /// are reported via `onCountIn`, not this), for the passive UI beat
  /// indicator (Layer 0 — no information beyond "here is the beat").
  var onBeat: ((_ bar: Int, _ beat: Int, _ hostTime: UInt64) -> Void)?

  /// Pending (hostTime, action) pairs, earliest first — consumed by
  /// `pollTask`. A queue polled on a short interval, rather than one
  /// independently-scheduled `Task.sleep` per beat: iOS coalesces long timer
  /// wakeups to save power, so a several-second sleep can fire tens to
  /// hundreds of ms late. The audio clicks themselves are scheduled via
  /// `AVAudioTime(hostTime:)` and rendered sample-accurately by the audio
  /// engine, immune to that coalescing — which is exactly why the UI could
  /// visibly drift out of sync with the audio despite matching host-time
  /// math: the click and the UI were being kept on time by two different
  /// mechanisms with two different precision guarantees.
  private var pendingBeats: [(hostTime: UInt64, fire: () -> Void)] = []
  private var pollTask: Task<Void, Never>?
  private let pollIntervalNanoseconds: UInt64 = 10_000_000  // 10ms

  init() throws {
    let format = AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 1)!
    // A distinct, lower-pitched tone for the count-in so it's audibly
    // different from the click that follows — otherwise a count-in reads
    // as "no count-in", just a run of identical clicks.
    countInBuffer = try Self.synthesizeClick(format: format, frequency: 600)
    clickBuffer = try Self.synthesizeClick(format: format, frequency: 1000)
    accentBuffer = try Self.synthesizeClick(format: format, frequency: 1500)
    engine.attach(playerNode)
    engine.connect(playerNode, to: engine.mainMixerNode, format: format)
  }

  /// Schedules a count-in of `countInBeats` clicks followed by `bodyBeats`
  /// phrase beats (beat 1 of each bar accented), then returns the resulting
  /// `BeatGrid`. Safe to call repeatedly — each call tears down and
  /// reschedules (used by the gate drill's auto count-in per rep).
  func start(bpm: Double, beatsPerBar: Int, countInBeats: Int, bodyBeats: Int) throws -> BeatGrid {
    playerNode.stop()

    let session = AVAudioSession.sharedInstance()
    try session.setCategory(.playback, mode: .default, options: [])
    try session.setActive(true)

    if !engine.isRunning {
      try engine.start()
    }
    playerNode.play()

    let secondsPerBeat = 60.0 / bpm
    let scheduledStart = HostClock.now() &+ HostClock.ticks(fromSeconds: leadInSeconds)
    let outputLatencyTicks = HostClock.ticks(fromSeconds: session.outputLatency)

    let totalBeats = countInBeats + bodyBeats
    for beatIndex in 0..<totalBeats {
      let scheduledHostTime =
        scheduledStart &+ HostClock.ticks(fromSeconds: Double(beatIndex) * secondsPerBeat)
      let buffer: AVAudioPCMBuffer
      if beatIndex < countInBeats {
        buffer = countInBuffer
      } else {
        let isDownbeat = (beatIndex - countInBeats) % beatsPerBar == 0
        buffer = isDownbeat ? accentBuffer : clickBuffer
      }
      playerNode.scheduleBuffer(buffer, at: AVAudioTime(hostTime: scheduledHostTime), options: [])
    }

    let audibleStart = scheduledStart &+ outputLatencyTicks
    let gridStart =
      audibleStart &+ HostClock.ticks(fromSeconds: Double(countInBeats) * secondsPerBeat)
    let grid = BeatGrid(
      bpm: bpm, beatsPerBar: beatsPerBar, countInBeats: countInBeats, startHostTime: gridStart)

    pendingBeats = buildSchedule(
      totalBeats: totalBeats, audibleStart: audibleStart, secondsPerBeat: secondsPerBeat, grid: grid
    )
    startPolling()

    return grid
  }

  func stop() {
    pollTask?.cancel()
    pollTask = nil
    pendingBeats = []
    playerNode.stop()
    if engine.isRunning {
      engine.stop()
    }
  }

  private func buildSchedule(
    totalBeats: Int, audibleStart: UInt64, secondsPerBeat: Double, grid: BeatGrid
  ) -> [(hostTime: UInt64, fire: () -> Void)] {
    let countInBeats = grid.countInBeats
    return (0..<totalBeats).map { beatIndex in
      let beatHostTime =
        audibleStart &+ HostClock.ticks(fromSeconds: Double(beatIndex) * secondsPerBeat)
      if beatIndex < countInBeats {
        let remaining = countInBeats - beatIndex
        return (beatHostTime, { [weak self] in self?.onCountIn?(remaining) })
      } else {
        let (bar, beat, _) = grid.nearestBeat(for: beatHostTime)
        return (beatHostTime, { [weak self] in self?.onBeat?(bar, beat, beatHostTime) })
      }
    }
  }

  /// Consumes `pendingBeats` as their host times pass, polling on a short
  /// fixed interval rather than sleeping until each one individually — see
  /// the comment on `pendingBeats` for why that matters here.
  private func startPolling() {
    pollTask?.cancel()
    pollTask = Task { @MainActor [weak self] in
      while let self, !Task.isCancelled, !self.pendingBeats.isEmpty {
        let now = HostClock.now()
        while let next = self.pendingBeats.first, next.hostTime <= now {
          self.pendingBeats.removeFirst()
          next.fire()
        }
        try? await Task.sleep(nanoseconds: self.pollIntervalNanoseconds)
      }
    }
  }

  /// A short sine-burst click, synthesized inline rather than shipped as a
  /// bundled asset — mirrors the inline-WAV trick in
  /// ios/Reference/BackgroundAudioPlugin.swift, adapted to an audible tone.
  private static func synthesizeClick(format: AVAudioFormat, frequency: Double) throws
    -> AVAudioPCMBuffer
  {
    let durationSeconds = 0.03
    let sampleRate = format.sampleRate
    let frameCount = AVAudioFrameCount(sampleRate * durationSeconds)
    guard let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: frameCount) else {
      throw ClickEngineError.bufferAllocationFailed
    }
    buffer.frameLength = frameCount
    let channel = buffer.floatChannelData![0]
    for frame in 0..<Int(frameCount) {
      let t = Double(frame) / sampleRate
      // Linear decay envelope so the click doesn't pop.
      let envelope = 1.0 - t / durationSeconds
      channel[frame] = Float(sin(2 * Double.pi * frequency * t) * envelope * 0.5)
    }
    return buffer
  }

  enum ClickEngineError: Error {
    case bufferAllocationFailed
  }
}
