import Observation
import SharedTypes

/// The Focus Player's click. Tempo, metre and pattern are session state that
/// never writes back to the item; the reflection hand-off reports them as
/// `clickState` and the core rules on what they evidence (#1499).
@MainActor @Observable
final class ClickController {
  private(set) var isRunning = false
  private(set) var bpm = 0
  private(set) var metre = Metre(beats: 4, unit: 4, groups: nil)
  private(set) var sounding: UInt16 = 0b1111
  /// Set only when the engine refused to start: an interruption or route change
  /// stops the pulse without breaking it, and a red row for headphones is a lie.
  private(set) var unavailable = false
  /// A phone left locked would otherwise click until the battery goes (#1399).
  var backgroundGrace: Duration = .seconds(600)
  private var backgroundStop: Task<Void, Never>?
  var backgroundStopArmed: Bool { backgroundStop != nil }

  private var engine: ClickEngine?
  private var limits: LimitsView?
  private var seeded = 0
  private var seededUnit: UInt8 = 4
  /// The item's own bar can hold a grouping the core's table does not list
  /// (4/8 as 2 + 2), so its patterns come with the seed (#2225).
  private var seedMetre: Metre?
  private var seedPresets: [ClickPresetOption] = []
  private var configured = false

  /// A bar change to another beat unit re-reads the same number as a different
  /// tempo, so it leaves the seed even when `bpm` is untouched (#1942).
  var isAtSeededTempo: Bool { bpm == seeded && metre.unit == seededUnit }
  /// False when the item declares no BPM, or one the clamp moved (#1942).
  private(set) var soundsTarget = false
  /// `nil` until the click is started or its bar is changed: the core reads
  /// `Some` as a metre the player chose, and would normalise a tempo against
  /// it (spec § question 2).
  var clickState: ClickState? {
    configured ? ClickState(metre: metre, sounding: sounding) : nil
  }

  var tempoStep: Int { limits?.clickStep ?? 0 }
  var band: ClosedRange<Int> { limits?.clickBand(unit: metre.unit) ?? bpm...bpm }

  var presets: [ClickPresetOption] {
    if let seedMetre, seedMetre.beats == metre.beats, seedMetre.groups == metre.groups {
      return seedPresets
    }
    return limits?.clickPresets(for: metre) ?? []
  }

  var matchingPreset: ClickPreset? {
    presets.first { $0.sounding == sounding }?.preset
  }

  /// Silences the click: its tempo and bar belonged to the item that just
  /// finished. Bar, tempo and pattern open on the core's answer (T19, #2225).
  func reseed(from active: ActiveSessionView, limits: LimitsView) {
    stop()
    unavailable = false
    self.limits = limits
    metre = active.clickSeedMetre
    seedMetre = active.clickSeedMetre
    seedPresets = active.clickSeedPresets
    seeded = Int(active.clickSeedBpm)
    seededUnit = metre.unit
    soundsTarget = active.clickSeedSoundsTarget
    bpm = seeded
    sounding = active.currentClickSounding
    configured = false
  }

  func toggle() {
    if isRunning {
      stop()
    } else {
      start()
    }
  }

  func step(by delta: Int) {
    let stepped = clamped(bpm + delta, unit: metre.unit)
    guard stepped != bpm else { return }
    bpm = stepped
    if isRunning { start() }
  }

  /// The drag gesture's absolute counterpart to `step(by:)` (#1823).
  func setBpm(_ newValue: Int) {
    let next = clamped(newValue, unit: metre.unit)
    guard next != bpm else { return }
    bpm = next
    if isRunning { start() }
  }

  /// A session-local override; the item keeps its own metre. The tempo keeps
  /// its number in the new unit where the band allows it: the pulse the player
  /// is hearing does not change because they named the beat differently.
  func setMetre(_ next: Metre) {
    guard next != metre else { return }
    metre = next
    bpm = clamped(bpm, unit: next.unit)
    sounding = presets.first { $0.preset == .everyBeat }?.sounding ?? 1
    configured = true
    if isRunning { start() }
  }

  func apply(_ preset: ClickPreset) {
    guard let option = presets.first(where: { $0.preset == preset }) else { return }
    setSounding(option.sounding)
  }

  /// Flips one beat; the last sounding beat cannot be silenced, since a click
  /// that sounds nothing is not a click.
  func toggleBeat(_ index: Int) {
    let bit: UInt16 = 1 << UInt16(index)
    let next = sounding ^ bit
    guard next != 0 else { return }
    setSounding(next)
  }

  private func clamped(_ value: Int, unit: UInt8) -> Int {
    limits?.clampClickTempo(value, unit: unit) ?? value
  }

  private func setSounding(_ next: UInt16) {
    guard next != sounding, next != 0 else { return }
    sounding = next
    configured = true
    if isRunning { start() }
  }

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
      configured = true
      unavailable = false
    } catch {
      report(error, "click.start")
      isRunning = false
      unavailable = true
    }
  }

  func stop() {
    backgroundStop?.cancel()
    backgroundStop = nil
    engine?.stop()
    isRunning = false
  }

  func enteredBackground() {
    backgroundStop?.cancel()
    backgroundStop = Task { [weak self, grace = backgroundGrace] in
      guard (try? await Task.sleep(for: grace)) != nil, let self else { return }
      self.stop()
    }
  }

  func enteredForeground() {
    backgroundStop?.cancel()
    backgroundStop = nil
  }

  /// The engine's observers outlive the screen unless it is torn down.
  func dispose() {
    backgroundStop?.cancel()
    backgroundStop = nil
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
