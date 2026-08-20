import Observation

/// The Focus Player's click. Shell-only by design: a metronome setting is UI
/// state, so nothing here crosses the bridge and nothing is persisted.
@MainActor @Observable
final class ClickController {
  /// Neutral tempo for an item declaring none; matches `ReflectionSheet` (#1402).
  static let defaultBpm = 96

  private(set) var isRunning = false
  private(set) var bpm = defaultBpm
  /// Set only when the engine refused to start: an interruption or route change
  /// stops the pulse without breaking it, and a red row for headphones is a lie.
  private(set) var unavailable = false

  private var engine: ClickEngine?
  private var seeded = defaultBpm

  var isAtSeededTempo: Bool { bpm == seeded }

  static func seedBpm(from target: UInt16?) -> Int {
    TempoStepper.clamp(target.map(Int.init) ?? defaultBpm)
  }

  /// Silences the click: its tempo belonged to the item that just finished.
  func reseed(target: UInt16?) {
    stop()
    unavailable = false
    seeded = Self.seedBpm(from: target)
    bpm = seeded
  }

  func toggle() {
    if isRunning {
      stop()
    } else {
      start()
    }
  }

  func step(by delta: Int) {
    let stepped = TempoStepper.stepped(from: bpm, by: delta)
    guard stepped != bpm else { return }
    bpm = stepped
    if isRunning { start() }
  }

  func start() {
    do {
      let engine = try engine ?? makeEngine()
      try engine.start(bpm: Double(bpm))
      isRunning = true
      unavailable = false
    } catch {
      report(error, "click.start")
      isRunning = false
      unavailable = true
    }
  }

  func stop() {
    engine?.stop()
    isRunning = false
  }

  /// The engine's observers outlive the screen unless it is torn down.
  func dispose() {
    engine?.dispose()
    engine = nil
    isRunning = false
  }

  private func makeEngine() throws -> ClickEngine {
    let engine = try ClickEngine()
    engine.onPulseDied = { [weak self] in self?.isRunning = false }
    self.engine = engine
    return engine
  }
}
