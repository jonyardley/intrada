import AVFoundation

/// AVAudioEngine metronome. Beats fire at the *audible* instant (scheduled +
/// outputLatency); the pulse is unbounded, so this tops up a rolling window
/// rather than scheduling one rep's worth (`specs/intrada-coach-engine.md` §6).
@MainActor
final class ClickEngine {
  private let engine = AVAudioEngine()
  private let playerNode = AVAudioPlayerNode()
  private let countInBuffer: AVAudioPCMBuffer
  private let clickBuffer: AVAudioPCMBuffer
  private let accentBuffer: AVAudioPCMBuffer

  private let leadInSeconds: Double = 0.5
  // Queue oscillates 24-88 beats, 12s to 44s at 120bpm: too long for a
  // coalesced wakeup to run it dry, short enough that `stop()` isn't fighting it.
  //
  // Polled rather than one `Task.sleep` per beat, since iOS coalesces long
  // timer wakeups while `AVAudioTime(hostTime:)` audio is immune to that.
  private let windowBeats = 64
  private let topUpBelow = 24
  static let maxLagBeats: Double = 2

  /// `remaining` is beats left *after* this one, down to 0 on the last (#1184).
  var onCountIn: ((_ remaining: Int) -> Void)?

  /// The pulse died and the shell did not choose to stop it. Reports the fact
  /// and nothing else; what it costs the block is the core's call.
  var onPulseDied: (() -> Void)?

  /// `index` counts on across reps from the pulse's first body beat. A beat the
  /// placement silences still fires — only the audio goes quiet (#1224).
  var onBeat: ((_ index: Int, _ hostTime: UInt64) -> Void)?

  private var pendingBeats: [ScheduledBeat] = []
  private var pollTask: Task<Void, Never>?
  private let pollIntervalNanoseconds: UInt64 = 10_000_000  // 10ms

  private var pulse: Pulse?
  private var nextBeat = 0
  private var observers: [NSObjectProtocol] = []

  struct Pulse {
    let bpm: Double
    let beatsPerBar: Int
    let countInBeats: Int
    /// Never empty or all-silent: the body would have no audible pulse at all.
    let clickPattern: [Bool]
    let scheduledStart: UInt64
    let outputLatencyTicks: UInt64

    var secondsPerBeat: Double { 60.0 / bpm }
  }

  enum Voice {
    case countIn
    case accent
    case click
    case silent
  }

  struct ScheduledBeat {
    let voice: Voice
    let hostTime: UInt64
    /// When the player *hears* it, and so when `fire` runs.
    let audibleHostTime: UInt64
    let fire: () -> Void
  }

  init() throws {
    let format = AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 1)!
    // Lower-pitched so the count-in is audibly distinct from the click.
    countInBuffer = try Self.synthesizeClick(format: format, frequency: 600)
    clickBuffer = try Self.synthesizeClick(format: format, frequency: 1000)
    accentBuffer = try Self.synthesizeClick(format: format, frequency: 1500)
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

  /// Safe to call repeatedly — each call restarts from the count-in.
  func start(bpm: Double, beatsPerBar: Int, countInBeats: Int, clickPattern: [Bool]) throws {
    // NaN `secondsPerBeat` trips `HostClock.ticks`' precondition, which is a
    // crash rather than something `unavailable()` can route around.
    guard bpm > 0 else { throw ClickEngineError.nonPositiveTempo }
    playerNode.stop()

    let session = AVAudioSession.sharedInstance()
    try session.setCategory(.playback, mode: .default, options: [])
    try session.setActive(true)

    if !engine.isRunning {
      try engine.start()
    }
    playerNode.play()

    pulse = Pulse(
      bpm: bpm, beatsPerBar: beatsPerBar, countInBeats: countInBeats,
      clickPattern: clickPattern,
      scheduledStart: HostClock.now() &+ HostClock.ticks(fromSeconds: leadInSeconds),
      outputLatencyTicks: HostClock.ticks(fromSeconds: session.outputLatency))
    nextBeat = 0
    pendingBeats = []
    scheduleWindow()
    startPolling()
  }

  func stop() {
    pollTask?.cancel()
    pollTask = nil
    pendingBeats = []
    pulse = nil
    nextBeat = 0
    playerNode.stop()
    if engine.isRunning {
      engine.stop()
    }
  }

  /// Silent beats keep their `fire` — the core counts beats, not clicks — so a
  /// placement level can never shift bar or rep tracking.
  func buildSchedule(beats: Range<Int>, pulse: Pulse) -> [ScheduledBeat] {
    beats.map { beatIndex in
      let offset = HostClock.ticks(fromSeconds: Double(beatIndex) * pulse.secondsPerBeat)
      let hostTime = pulse.scheduledStart &+ offset
      let audible = hostTime &+ pulse.outputLatencyTicks
      if beatIndex < pulse.countInBeats {
        let remaining = pulse.countInBeats - beatIndex - 1
        return ScheduledBeat(
          voice: .countIn, hostTime: hostTime, audibleHostTime: audible,
          fire: { [weak self] in self?.onCountIn?(remaining) })
      }
      let index = beatIndex - pulse.countInBeats
      return ScheduledBeat(
        voice: voice(bodyBeat: index, pulse: pulse), hostTime: hostTime,
        audibleHostTime: audible,
        fire: { [weak self] in self?.onBeat?(index, audible) })
    }
  }

  private func voice(bodyBeat index: Int, pulse: Pulse) -> Voice {
    guard !pulse.clickPattern.isEmpty, pulse.clickPattern[index % pulse.clickPattern.count] else {
      return .silent
    }
    guard pulse.beatsPerBar > 0 else { return .click }
    return index % pulse.beatsPerBar == 0 ? .accent : .click
  }

  private func scheduleWindow() {
    guard let pulse else { return }
    let beats = nextBeat..<(nextBeat + windowBeats)
    let scheduled = buildSchedule(beats: beats, pulse: pulse)
    for beat in scheduled {
      guard let buffer = buffer(for: beat.voice) else { continue }
      playerNode.scheduleBuffer(buffer, at: AVAudioTime(hostTime: beat.hostTime), options: [])
    }
    pendingBeats.append(contentsOf: scheduled)
    nextBeat = beats.upperBound
  }

  private func buffer(for voice: Voice) -> AVAudioPCMBuffer? {
    switch voice {
    case .countIn: countInBuffer
    case .accent: accentBuffer
    case .click: clickBuffer
    case .silent: nil
    }
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
          next.fire()
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

  /// A backlog this deep is a suspended app, not a late wakeup. Draining it
  /// would hand the core hundreds of beats at once, so the schedule is
  /// abandoned rather than fast-forwarded.
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
        let options = note.userInfo?[AVAudioSessionInterruptionOptionKey] as? UInt
        MainActor.assumeIsolated { self?.handleInterruption(type: type, options: options) }
      })
    // A route change (the headphones case) tears the graph down, leaving
    // `isRunning` false with nothing to restart it.
    observers.append(
      centre.addObserver(
        forName: .AVAudioEngineConfigurationChange, object: engine, queue: .main
      ) { [weak self] _ in
        MainActor.assumeIsolated { self?.abandonPulse() }
      })
  }

  private func handleInterruption(type: UInt?, options: UInt?) {
    guard let type, let kind = AVAudioSession.InterruptionType(rawValue: type) else { return }
    switch kind {
    case .began:
      abandonPulse()
    case .ended:
      // `shouldResume` is the system's opinion about the audio, not the coach's
      // about the block. The core parked it; the player taps Start.
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
