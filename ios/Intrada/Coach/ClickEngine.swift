import AVFoundation

/// AVAudioEngine metronome scheduled via AVAudioTime(hostTime:). Beat callbacks
/// fire at the *audible* instant (scheduled time + AVAudioSession.outputLatency),
/// since that is what the player reacts to.
///
/// The pulse is unbounded: the core says "keep clicking" by leaving the pulse key
/// alone, so this tops up a rolling window of beats rather than scheduling one
/// rep's worth (`specs/intrada-coach-engine.md` §6, "The pulse, and how the
/// shell knows what to do with it").
@MainActor
final class ClickEngine {
  private let engine = AVAudioEngine()
  private let playerNode = AVAudioPlayerNode()
  private let countInBuffer: AVAudioPCMBuffer
  private let clickBuffer: AVAudioPCMBuffer
  private let accentBuffer: AVAudioPCMBuffer

  private let leadInSeconds: Double = 0.5
  /// Beats scheduled ahead, and the level the poll tops up at, so the queue
  /// oscillates between 24 and 88 beats — 12s to 44s at 120bpm. Far enough that
  /// a coalesced wakeup can't run it dry, short enough that `stop()` is not
  /// fighting a long queue.
  private let windowBeats = 64
  private let topUpBelow = 24
  /// Past this much lag the queue is not late, it is stranded — see
  /// `hasLostTheClock`.
  static let maxLagBeats: Double = 2

  /// Fires as each count-in click sounds; `remaining` is beats left *after*
  /// this one, counting down to 0 on the last click (#1184).
  var onCountIn: ((_ remaining: Int) -> Void)?

  /// The pulse died and the shell did not choose to stop it — an interruption
  /// began, the engine's configuration changed under it, or the clock ran away.
  /// Reports the fact and nothing else: no further beats are reported, and what
  /// it costs the block is the core's call, not this one's.
  var onPulseDied: (() -> Void)?

  /// Fires after each *body* beat's audible host time (Layer 0 UI pip).
  /// `index` is 0-based from the pulse's first body beat and counts on across
  /// reps; the core turns it into a bar, a beat and a rep. A beat the placement
  /// silences still fires — only the audio goes quiet (#1224).
  var onBeat: ((_ index: Int, _ hostTime: UInt64) -> Void)?

  /// Pending (hostTime, action) pairs, polled on a short interval rather
  /// than one `Task.sleep` per beat — iOS coalesces long timer wakeups to
  /// save power, so a several-second sleep can fire tens to hundreds of ms
  /// late, while the audio itself (scheduled via `AVAudioTime(hostTime:)`)
  /// renders sample-accurately and is immune to that.
  private var pendingBeats: [ScheduledBeat] = []
  private var pollTask: Task<Void, Never>?
  private let pollIntervalNanoseconds: UInt64 = 10_000_000  // 10ms

  private var pulse: Pulse?
  /// The first beat not yet scheduled, count-in included.
  private var nextBeat = 0
  private var observers: [NSObjectProtocol] = []

  struct Pulse {
    let bpm: Double
    let beatsPerBar: Int
    let countInBeats: Int
    /// One cycle of the click placement, from the first body beat. An empty or
    /// all-silent pattern would leave a body with no audible pulse at all, so
    /// callers pass the core's `clickPattern`, which always sounds something.
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
    /// When the buffer is handed to the player node.
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

  /// Unregisters the interruption observers. Explicit rather than a `deinit`,
  /// which is nonisolated and so cannot touch them under Swift 6; the host calls
  /// it when it lets the engine go.
  func dispose() {
    stop()
    for observer in observers { NotificationCenter.default.removeObserver(observer) }
    observers = []
  }

  /// Starts a pulse: `countInBeats` clicks on every beat, then body beats that
  /// sound where `clickPattern` says, on and on until `stop()`. Safe to call
  /// repeatedly — each call restarts from the count-in.
  func start(bpm: Double, beatsPerBar: Int, countInBeats: Int, clickPattern: [Bool]) throws {
    // `secondsPerBeat` would be infinite or NaN, and `HostClock.ticks`'
    // precondition on NaN is a crash rather than something `unavailable()` can
    // route around. Core-controlled today; one line so it cannot become a trap.
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

  /// One window's worth of beats: which voice each carries, when it sounds, and
  /// what it reports. Silent beats keep their `fire` — the core counts beats,
  /// not clicks — so a placement level can never shift bar or rep tracking.
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

  /// A backlog deeper than a couple of beats is not a late wakeup — the app was
  /// suspended (there is no `UIBackgroundModes: audio`, so the poll task simply
  /// freezes) or the main thread stalled for seconds. Every queued host time is
  /// then in the past, and draining them would hand the core hundreds of beats
  /// in one iteration: phrase boundaries the player never played, verdict
  /// windows opening and dropping, and a burst of clicks from buffers reaching
  /// a live node with a past host time. So the schedule is abandoned and a
  /// fresh pulse starts, count-in first, rather than fast-forwarded.
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
        // Read the raw values out here: `Notification` is not Sendable, so
        // carrying it across the isolation boundary is a data race, while the
        // two `UInt`s inside it are fine.
        let type = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt
        let options = note.userInfo?[AVAudioSessionInterruptionOptionKey] as? UInt
        MainActor.assumeIsolated { self?.handleInterruption(type: type, options: options) }
      })
    // The engine tears its graph down on a route or configuration change (the
    // headphones case), leaving `isRunning` false with nothing to restart it.
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
      // Deliberately nothing. `shouldResume` is the system's opinion about the
      // audio, not the coach's about the block: the core has parked the block
      // on its card, and it restarts when the player taps Start.
      break
    @unknown default:
      abandonPulse()
    }
  }

  /// Stops everything and reports it. Silent about a pulse that was not running,
  /// so a route change between blocks is not an event — and silent from `stop()`
  /// itself, so a teardown the shell chose is never mistaken for a death.
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
