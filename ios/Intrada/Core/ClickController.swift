import Observation
import SharedTypes

/// The Focus Player's click. Tempo, metre and pattern are session state that
/// never writes back to the item; the reflection hand-off reports them as
/// `clickState` and the core rules on what they evidence (#1499).
@MainActor @Observable
final class ClickController {
  private(set) var isRunning = false
  private(set) var bpm = TempoScale.defaultBpm
  private(set) var metre = Metre(beats: 4, unit: 4, groups: nil)
  private(set) var sounding: UInt16 = 0b1111
  /// Set only when the engine refused to start: an interruption or route change
  /// stops the pulse without breaking it, and a red row for headphones is a lie.
  private(set) var unavailable = false

  private var engine: ClickEngine?
  private var seeded = TempoScale.defaultBpm

  var isAtSeededTempo: Bool { bpm == seeded }
  var clickState: ClickState { ClickState(metre: metre, sounding: sounding) }

  static func seedBpm(from target: UInt16?) -> Int {
    TempoScale.clamp(target.map(Int.init) ?? TempoScale.defaultBpm)
  }

  /// Silences the click: its tempo and bar belonged to the item that just
  /// finished. The metre opens with the piece's answer in it (T19).
  func reseed(target: UInt16?, metre itemMetre: Metre?) {
    stop()
    unavailable = false
    seeded = Self.seedBpm(from: target)
    bpm = seeded
    metre = itemMetre ?? Metre(beats: 4, unit: 4, groups: nil)
    sounding = ClickPattern.everyBeat(of: metre)
  }

  func toggle() {
    if isRunning {
      stop()
    } else {
      start()
    }
  }

  func step(by delta: Int) {
    let stepped = TempoScale.stepped(from: bpm, by: delta)
    guard stepped != bpm else { return }
    bpm = stepped
    if isRunning { start() }
  }

  /// A session-local override; the item keeps its own metre.
  func setMetre(_ next: Metre) {
    guard next != metre else { return }
    metre = next
    sounding = ClickPattern.everyBeat(of: next)
    if isRunning { start() }
  }

  func apply(_ pattern: ClickPattern) {
    setSounding(pattern.mask(for: metre))
  }

  /// Flips one beat; the last sounding beat cannot be silenced, since a click
  /// that sounds nothing is not a click.
  func toggleBeat(_ index: Int) {
    let bit: UInt16 = 1 << UInt16(index)
    let next = sounding ^ bit
    guard next != 0 else { return }
    setSounding(next)
  }

  private func setSounding(_ next: UInt16) {
    guard next != sounding, next != 0 else { return }
    sounding = next
    if isRunning { start() }
  }

  /// The beat being heard now, for the travelling indicator; nil when silent.
  func currentBeat() -> Int? {
    guard isRunning else { return nil }
    return engine?.currentBeat()
  }

  func start() {
    do {
      let engine = try engine ?? makeEngine()
      try engine.start(
        bpm: Double(bpm),
        pattern: ClickEngine.BeatPattern(beats: Int(metre.beats), sounding: sounding))
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
