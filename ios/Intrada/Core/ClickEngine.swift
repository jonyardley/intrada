import AVFoundation

/// AVAudioEngine metronome. Beats sit on a host-time grid struck once from
/// `scheduledStart`, because a timer-driven click audibly drifts and musicians
/// hear milliseconds. The pulse is unbounded, so the window rolls.
@MainActor
final class ClickEngine {
  private let engine = AVAudioEngine()
  private let playerNode = AVAudioPlayerNode()
  private let clickBuffer: AVAudioPCMBuffer

  /// Headroom for the first `scheduleBuffer` to land. Short because a tempo
  /// change restarts the pulse, and a long gap per stepper tap reads as a stall.
  private let leadInSeconds: Double = 0.2
  // Queue oscillates 24-88 beats, 12s to 44s at 120bpm: too long for a
  // coalesced wakeup to run it dry, short enough that `stop()` isn't fighting it.
  // Polled rather than a sleep per beat, since iOS coalesces long timer wakeups
  // while `AVAudioTime(hostTime:)` audio is immune to that.
  private let windowBeats = 64
  private let topUpBelow = 24
  static let maxLagBeats: Double = 2

  /// The pulse stopped without the shell asking (interruption, route change).
  /// Not a failure: the click is still available, it just isn't sounding.
  var onPulseDied: (() -> Void)?

  private var pendingBeats: [ScheduledBeat] = []
  private var pollTask: Task<Void, Never>?
  private let pollIntervalNanoseconds: UInt64 = 100_000_000  // 100ms

  private var pulse: Pulse?
  private var nextBeat = 0
  private var observers: [NSObjectProtocol] = []

  struct Pulse {
    let bpm: Double
    let scheduledStart: UInt64
    let outputLatencyTicks: UInt64

    var secondsPerBeat: Double { 60.0 / bpm }
  }

  struct ScheduledBeat {
    let hostTime: UInt64
    /// When the player *hears* it — scheduled plus output latency.
    let audibleHostTime: UInt64
  }

  init() throws {
    let format = AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 1)!
    clickBuffer = try Self.synthesizeClick(format: format, frequency: 1000)
    engine.attach(playerNode)
    engine.connect(playerNode, to: engine.mainMixerNode, format: format)
    observeInterruptions()
  }

  /// Explicit rather than a `deinit`, which is nonisolated and so cannot touch
  /// the observers under Swift 6.
  func dispose() {
    stop()
    for observer in observers { NotificationCenter.default.removeObserver(observer) }
    observers = []
  }

  /// Safe to call repeatedly — each call restarts the grid at the new tempo.
  func start(bpm: Double) throws {
    // NaN `secondsPerBeat` trips `HostClock.ticks`' precondition, which is a
    // crash rather than something the caller can route around.
    guard bpm > 0 else { throw ClickEngineError.nonPositiveTempo }
    playerNode.stop()

    let session = AVAudioSession.sharedInstance()
    do {
      // .mixWithOthers so a backing track or tuner keeps playing, and only
      // system events (calls, Siri, alarms) count as an interruption.
      try session.setCategory(.playback, mode: .default, options: [.mixWithOthers])
      try session.setActive(true)
      // A configuration change (headphones in or out) tears the graph's
      // connections down; without this the engine starts clean and plays nothing.
      engine.connect(playerNode, to: engine.mainMixerNode, format: clickBuffer.format)
      if !engine.isRunning {
        try engine.start()
      }
    } catch {
      // The previous pulse's poll task outlives a failed restart otherwise,
      // topping up a window nobody plays for the rest of the session.
      stop()
      throw error
    }
    playerNode.play()

    pulse = Pulse(
      bpm: bpm,
      scheduledStart: HostClock.now() &+ HostClock.ticks(fromSeconds: leadInSeconds),
      outputLatencyTicks: HostClock.ticks(fromSeconds: session.outputLatency))
    nextBeat = 0
    pendingBeats = []
    scheduleWindow()
    startPolling()
  }

  func stop() {
    let wasSounding = pulse != nil
    pollTask?.cancel()
    pollTask = nil
    pendingBeats = []
    pulse = nil
    nextBeat = 0
    playerNode.stop()
    if engine.isRunning {
      engine.stop()
    }
    // Lets a ducked backing-track app resume. Guarded because the session is
    // app-wide: a stop with nothing sounding must not deactivate it.
    guard wasSounding else { return }
    try? AVAudioSession.sharedInstance().setActive(false, options: [.notifyOthersOnDeactivation])
  }

  /// Pure beat layout: touches no audio state, so the grid can be exercised
  /// without standing up a real `AVAudioEngine` (#1282). Every beat is an
  /// offset from `scheduledStart`, never from its predecessor, so topping up
  /// the rolling window cannot accumulate error.
  static func schedule(beats: Range<Int>, pulse: Pulse) -> [ScheduledBeat] {
    beats.map { index in
      let hostTime =
        pulse.scheduledStart &+ HostClock.ticks(fromSeconds: Double(index) * pulse.secondsPerBeat)
      return ScheduledBeat(
        hostTime: hostTime, audibleHostTime: hostTime &+ pulse.outputLatencyTicks)
    }
  }

  private func scheduleWindow() {
    guard let pulse else { return }
    let beats = nextBeat..<(nextBeat + windowBeats)
    let scheduled = Self.schedule(beats: beats, pulse: pulse)
    for beat in scheduled {
      playerNode.scheduleBuffer(clickBuffer, at: AVAudioTime(hostTime: beat.hostTime), options: [])
    }
    pendingBeats.append(contentsOf: scheduled)
    nextBeat = beats.upperBound
  }

  private func startPolling() {
    pollTask?.cancel()
    pollTask = Task { @MainActor [weak self] in
      while let self, !Task.isCancelled {
        let now = HostClock.now()
        if self.clockRanAway(now: now) {
          self.abandonPulse()
          return
        }
        while let next = self.pendingBeats.first, next.audibleHostTime <= now {
          self.pendingBeats.removeFirst()
        }
        if self.pendingBeats.count < self.topUpBelow { self.scheduleWindow() }
        try? await Task.sleep(nanoseconds: self.pollIntervalNanoseconds)
      }
    }
  }

  private func clockRanAway(now: UInt64) -> Bool {
    guard let pulse, let head = pendingBeats.first else { return false }
    return Self.hasLostTheClock(
      head: head.audibleHostTime, now: now, secondsPerBeat: pulse.secondsPerBeat)
  }

  /// A backlog this deep is a suspended app, not a late wakeup: draining it
  /// would fire the whole window at once, so the schedule is abandoned.
  static func hasLostTheClock(head: UInt64, now: UInt64, secondsPerBeat: Double) -> Bool {
    HostClock.secondsBetween(now, head) > maxLagBeats * secondsPerBeat
  }

  // ── Interruptions ──

  private func observeInterruptions() {
    let centre = NotificationCenter.default
    observers.append(
      centre.addObserver(
        forName: AVAudioSession.interruptionNotification, object: nil, queue: .main
      ) { [weak self] note in
        // Unpacked here because `Notification` is not Sendable; the `UInt`s are.
        let type = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt
        MainActor.assumeIsolated { self?.handleInterruption(type: type) }
      })
    // A route change leaves `isRunning` false with nothing to restart it.
    observers.append(
      centre.addObserver(
        forName: .AVAudioEngineConfigurationChange, object: engine, queue: .main
      ) { [weak self] _ in
        MainActor.assumeIsolated { self?.abandonPulse() }
      })
  }

  private func handleInterruption(type: UInt?) {
    guard let type, let kind = AVAudioSession.InterruptionType(rawValue: type) else { return }
    switch kind {
    case .began:
      abandonPulse()
    case .ended:
      // `shouldResume` is the system's opinion about the audio, not the
      // musician's about the click. The pulse stopped; they tap it back on.
      break
    @unknown default:
      abandonPulse()
    }
  }

  /// Silent about a pulse that was not running, and never called from `stop()`,
  /// so a teardown the shell chose is not mistaken for a death.
  private func abandonPulse() {
    guard pulse != nil else { return }
    stop()
    onPulseDied?()
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

  enum ClickEngineError: Error, Equatable {
    case bufferAllocationFailed
    case nonPositiveTempo
  }
}
