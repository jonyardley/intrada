import Observation

/// The Focus Player's click: the tempo on screen plus the engine's lifecycle.
/// Shell-only by design — a metronome setting is UI interaction state, not
/// domain data, so nothing here crosses the bridge and nothing is persisted.
@MainActor @Observable
final class ClickController {
  /// Neutral practice tempo for an item that declares none. Matches
  /// `ReflectionSheet`'s fallback so the two tempo surfaces agree.
  static let defaultBpm = 96

  private(set) var isRunning = false
  private(set) var bpm = defaultBpm
  /// Set when the engine refused to start or the pulse died under us, so the
  /// screen never shows a running click that makes no sound.
  private(set) var unavailable = false

  private var engine: ClickEngine?
  private var seeded = defaultBpm

  /// False once the musician has stepped the click off what this item seeded,
  /// which is when the row stops speaking for the item's declared tempo.
  var isAtSeededTempo: Bool { bpm == seeded }

  static func seedBpm(from target: UInt16?) -> Int {
    TempoStepper.clamp(target.map(Int.init) ?? defaultBpm)
  }

  /// Silences whatever was running: the tempo the click was keeping belonged to
  /// the item that just finished.
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

  /// Tears the audio graph down when the player goes away — the observers the
  /// engine holds outlive the screen otherwise.
  func dispose() {
    engine?.dispose()
    engine = nil
    isRunning = false
  }

  private func makeEngine() throws -> ClickEngine {
    let engine = try ClickEngine()
    engine.onPulseDied = { [weak self] in
      self?.isRunning = false
      self?.unavailable = true
    }
    self.engine = engine
    return engine
  }
}
