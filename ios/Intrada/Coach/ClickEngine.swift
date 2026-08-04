import AVFoundation

/// AVAudioEngine metronome scheduled via AVAudioTime(hostTime:). Beat callbacks
/// fire at the *audible* instant (scheduled time + AVAudioSession.outputLatency),
/// since that is what the player reacts to.
@MainActor
final class ClickEngine {
  private let engine = AVAudioEngine()
  private let playerNode = AVAudioPlayerNode()
  private let countInBuffer: AVAudioPCMBuffer
  private let clickBuffer: AVAudioPCMBuffer
  private let accentBuffer: AVAudioPCMBuffer

  private let leadInSeconds: Double = 0.5

  /// Fires as each count-in click sounds; `remaining` is beats left *after*
  /// this one, counting down to 0 on the last click (#1184).
  var onCountIn: ((_ remaining: Int) -> Void)?

  /// Fires after each *body* beat's audible host time (Layer 0 UI pip).
  /// `index` is 0-based from bar 1 beat 1; the core turns it into a bar and beat.
  var onBeat: ((_ index: Int, _ hostTime: UInt64) -> Void)?

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
  func start(bpm: Double, beatsPerBar: Int, countInBeats: Int, bodyBeats: Int) throws {
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

    pendingBeats = buildSchedule(
      totalBeats: totalBeats, countInBeats: countInBeats,
      audibleStart: scheduledStart &+ outputLatencyTicks, secondsPerBeat: secondsPerBeat)
    startPolling()
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

  func buildSchedule(
    totalBeats: Int, countInBeats: Int, audibleStart: UInt64, secondsPerBeat: Double
  ) -> [(hostTime: UInt64, fire: () -> Void)] {
    (0..<totalBeats).map { beatIndex in
      let beatHostTime =
        audibleStart &+ HostClock.ticks(fromSeconds: Double(beatIndex) * secondsPerBeat)
      if beatIndex < countInBeats {
        let remaining = countInBeats - beatIndex - 1
        return (beatHostTime, { [weak self] in self?.onCountIn?(remaining) })
      }
      let index = beatIndex - countInBeats
      return (beatHostTime, { [weak self] in self?.onBeat?(index, beatHostTime) })
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
