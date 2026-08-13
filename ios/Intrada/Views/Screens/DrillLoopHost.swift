import SharedTypes
import SwiftUI

/// Runs `DrillScreen` against the real click. Pure I/O: schedule the clicks the
/// core asked for, report the ones that sounded, report taps and seconds. Every
/// decision — counting, gating, the escalation ladder, what comes next — is in
/// the core's session state machine (`specs/intrada-coach-engine.md` §4).
struct DrillLoopHost: View {
  @Environment(Store.self) private var store
  @Environment(\.scenePhase) private var scenePhase

  var onClose: () -> Void

  @State private var click: Click?
  /// The core bumps `pulseSeq` only when the click must actually restart, so
  /// an unchanged key means leave it alone (spec §6).
  @State private var startedPulse: Pulse?

  private struct Pulse: Equatable {
    let block: UInt64
    let seq: UInt32
  }

  private var drill: DrillView? { store.viewModel?.coach.drill }
  private var feel: FeelPrompt? { store.viewModel?.built.feel }
  private var reflection: Bool { store.viewModel?.built.reflection ?? false }
  private var landing: LandingView? { store.viewModel?.coach.landing }
  private var stage: LoopStage {
    LoopStage.of(drill: drill, feel: feel, reflection: reflection, landing: landing)
  }

  var body: some View {
    Group {
      switch stage {
      case .feel(let feel):
        FeelScreen(
          prompt: feel,
          onFeel: { chosen in
            store.send(
              .builtSession(.recordFeel(blockId: feel.blockId, feel: chosen)),
              onSuccess: .impact)
          },
          onSkip: { store.send(.builtSession(.skipFeel), onSuccess: .selection) })
      case .drill(let drill):
        DrillScreen(
          state: drill,
          onVerdict: { clean in
            store.send(.coach(.tap(clean: clean, now: SessionClock.nowRFC3339())))
          },
          onStuck: { store.send(.coach(.stuck(now: SessionClock.nowRFC3339()))) },
          // `onSuccess` fires only once the core has accepted the event, so a
          // rejected one can never feel like it landed.
          onDiscard: {
            store.send(
              .coach(.discardAttempt(now: SessionClock.nowRFC3339())), onSuccess: .impact)
          },
          onStart: {
            store.send(.coach(.startBlock(now: SessionClock.nowRFC3339())), onSuccess: .impact)
          },
          onSkip: {
            store.send(.coach(.skipBlock(now: SessionClock.nowRFC3339())), onSuccess: .impact)
          },
          onDismiss: dismiss)
      case .reflection:
        SessionReflectionHost()
      case .landing(let landing):
        SoftLandingScreen(
          state: landing,
          onDone: { store.send(.coach(.closeLanding), onSuccess: .selection) })
      case .idle:
        Color.clear
      }
    }
    // The loop is a fullScreenCover, so RootView's banner is occluded and a
    // lost block would be a silent no-op (#846, #1181).
    .safeAreaInset(edge: .top, spacing: 0) {
      if let error = store.viewModel?.error {
        GlobalBanner(message: error) { store.send(.clearError) }
      }
    }
    .task { await run() }
    .onDisappear(perform: teardown)
    .onChange(of: drill.map { PulseState(from: $0) }) { _, _ in
      syncClick()
    }
    .onChange(of: stage.isIdle) { _, idle in
      // The core closed the block — ceiling, ladder or gate — and the session
      // has since been landed and acknowledged, so this only tears down.
      if idle { close() }
    }
    // No `UIBackgroundModes: audio`, so a suspended app's poll task freezes
    // while its host times run on: the pulse is dead the moment we background.
    .onChange(of: scenePhase) { _, phase in
      switch phase {
      case .background:
        pulseDied()
      // Control Centre, the app switcher, a banner: never suspended, audio
      // still playing. Stopping would cost a block for an idle gesture.
      case .active, .inactive:
        break
      @unknown default:
        break
      }
    }
  }

  private func run() async {
    // An offer the core is still holding means the last session was cut off
    // mid-block: take it rather than starting fresh, or the evidence already
    // banked is discarded (#1181). Pressing start is the second door onto it —
    // the launch prompt on Practice is the first (#1305).
    let now = SessionClock.nowRFC3339()
    if store.viewModel?.coach.recovery != nil {
      store.send(
        .coach(
          .acceptRecovery(
            now: now, utcOffsetMinutes: SessionClock.utcOffsetMinutes())))
    } else {
      store.send(
        .coach(
          .startPlannedSession(
            now: now, utcOffsetMinutes: SessionClock.utcOffsetMinutes())))
    }
    // No block came back — the core could not plan one, and has said why. Now
    // that press-start is a user path (#1182), holding a blank screen instead
    // of returning is the #846 silent no-op.
    guard drill != nil else { return closeIfNothingLeftToAsk() }
    syncClick()
    while !Task.isCancelled {
      try? await Task.sleep(for: .seconds(1))
      guard !Task.isCancelled else { return }
      store.send(.coach(.tick(now: SessionClock.nowRFC3339())))
    }
  }

  /// The shell's whole click rule, with no domain reasoning in it.
  private func syncClick() {
    // No drill is no pulse: the cover now outlives the session, so teardown is
    // no longer what stops the last block's click.
    guard let drill else { return silencePulse() }
    guard drill.pulseRunning else { return silencePulse() }
    let pulse = Pulse(block: drill.blockIndex, seq: drill.pulseSeq)
    guard startedPulse != pulse else { return }

    // Only a sounding pulse is a consumed one: bailing out with `startedPulse`
    // set would leave a silent, permanently frozen screen (the #846 class).
    guard let engine = clickEngine() else { return unavailable() }
    guard engine.start(drill: drill, store: store) else { return unavailable() }
    startedPulse = pulse
  }

  /// No beat is reported while nothing sounds, so the core cannot count passes
  /// nobody played.
  private func silencePulse() {
    click?.stop()
    startedPulse = nil
  }

  /// What it costs is the core's call. Guarded on a live drill so a death
  /// during teardown cannot park a closed block.
  private func pulseDied() {
    guard drill != nil else { return }
    silencePulse()
    store.send(.coach(.clickInterrupted(now: SessionClock.nowRFC3339())))
  }

  private func clickEngine() -> Click? {
    if let click { return click }
    do {
      let created = try Click()
      created.onPulseDied = { pulseDied() }
      click = created
      return created
    } catch {
      report(error, "drill-click")
      return nil
    }
  }

  /// No click means no drill, so the core ends the session and surfaces why —
  /// rather than the screen sitting still until the ceiling fires.
  private func unavailable() {
    store.send(.coach(.clickUnavailable(now: SessionClock.nowRFC3339())))
    closeIfNothingLeftToAsk()
  }

  private func dismiss() {
    store.send(.coach(.leaveSession(now: SessionClock.nowRFC3339())))
    closeIfNothingLeftToAsk()
  }

  /// Every exit runs through here: all three close the block that was running,
  /// and a cover torn down on the way out takes its questions and its landing
  /// with it (#1323).
  private func closeIfNothingLeftToAsk() {
    if stage.isIdle { close() }
  }

  private func close() {
    teardown()
    onClose()
  }

  private func teardown() {
    click?.dispose()
    click = nil
    startedPulse = nil
  }
}

/// The only fields the click is driven by: a tap, a gate or a beat must not
/// reach `syncClick`.
private struct PulseState: Equatable {
  let block: UInt64
  let seq: UInt32
  let running: Bool

  init(from drill: DrillView) {
    block = drill.blockIndex
    seq = drill.pulseSeq
    running = drill.pulseRunning
  }
}

/// A reference wrapper so the callbacks below outlive a SwiftUI body pass, and
/// so the AVAudioEngine survives a re-render mid-block.
@MainActor
private final class Click {
  private let engine: ClickEngine

  var onPulseDied: (() -> Void)? {
    get { engine.onPulseDied }
    set { engine.onPulseDied = newValue }
  }

  init() throws {
    engine = try ClickEngine()
  }

  /// `false` = the click could not be scheduled, so this pulse never sounded.
  func start(drill: DrillView, store: Store) -> Bool {
    engine.onCountIn = { remaining in
      store.send(.coach(.countInBeat(remaining: UInt8(max(0, remaining)))))
    }
    engine.onBeat = { index, _ in
      store.send(.coach(.beat(beatIndex: UInt32(max(0, index)))))
    }
    // The core sends no tempo at l0, and holds `pulseRunning` false there, so
    // this is unreachable rather than a state to invent a bpm for.
    guard let bpm = drill.tempoBpm else { return false }
    do {
      try engine.start(
        bpm: Double(bpm), beatsPerBar: Int(drill.beatsPerBar),
        countInBeats: Int(drill.countInBeats), clickPattern: drill.clickPattern)
      return true
    } catch {
      report(error, "drill-click-start")
      return false
    }
  }

  func stop() {
    engine.onCountIn = nil
    engine.onBeat = nil
    engine.stop()
  }

  /// Unhooks the callbacks that `stop()` deliberately leaves live, a stopped
  /// pulse being exactly when they still need to speak.
  func dispose() {
    onPulseDied = nil
    engine.onCountIn = nil
    engine.onBeat = nil
    engine.dispose()
  }
}
