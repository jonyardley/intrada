import AVFoundation

/// AVAudioEngine metronome scheduled via AVAudioTime(hostTime:) — the same
/// mach_absolute_time clock CoreMIDI stamps with. `start`'s `BeatGrid.startHostTime`
/// is the *audible* instant of bar 1 beat 1 (scheduled time +
/// AVAudioSession.outputLatency), since that's what the player reacts to.
@MainActor
final class ClickEngine {
  private let engine = AVAudioEngine()
  private let playerNode = AVAudioPlayerNode()
  private let countInBuffer: AVAudioPCMBuffer
  private let clickBuffer: AVAudioPCMBuffer
  private let accentBuffer: AVAudioPCMBuffer

  private let leadInSeconds: Double = 0.5

  /// Beats remaining, counting down to 1 — distinct from `onBeat` since the
  /// count-in needs its own audible tone and on-screen countdown.
  var onCountIn: ((_ remaining: Int) -> Void)?

  /// Fires after each *body* beat's audible host time (Layer 0 UI pip).
  var onBeat: ((_ bar: Int, _ beat: Int, _ hostTime: UInt64) -> Void)?

  /// Pending (hostTime, action) pairs, polled on a short interval rather
  /// than one `Task.sleep` per beat — iOS coalesces long timer wakeups to
  /// save power, so a several-second sleep can fire tens to hundreds of ms
  /// late, while the audio itself (scheduled via `AVAudioTime(hostTime:)`)
  /// renders sample-accurately and is immune to that.
  private var pendingBeats: [(hostTime: UInt64, fire: () -> Void)] = []
  private var pollTask: Task<Void, Never>?
  private let pollIntervalNanoseconds: UInt64 = 10_000_000  // 10ms

  init() throws {
    let format = AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 1)!
    // Lower-pitched so the count-in is audibly distinct from the click.
    countInBuffer = try Self.synthesizeClick(format: format, frequency: 600)
    clickBuffer = try Self.synthesizeClick(format: format, frequency: 1000)
    accentBuffer = try Self.synthesizeClick(format: format, frequency: 1500)
    engine.attach(playerNode)
    engine.connect(playerNode, to: engine.mainMixerNode, format: format)
  }

  /// Schedules `countInBeats` clicks then `bodyBeats` phrase beats (bar
  /// downbeats accented). Safe to call repeatedly — each call reschedules.
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

  /// Synthesized inline (mirrors ios/Reference/BackgroundAudioPlugin.swift's
  /// silent-WAV trick) rather than bundled, to avoid a resource dependency.
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
      let envelope = 1.0 - t / durationSeconds  // linear decay, avoids a pop
      channel[frame] = Float(sin(2 * Double.pi * frequency * t) * envelope * 0.5)
    }
    return buffer
  }

  enum ClickEngineError: Error {
    case bufferAllocationFailed
  }
}
