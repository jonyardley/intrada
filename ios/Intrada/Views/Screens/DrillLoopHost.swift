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
  /// The pulse the click is sounding. The core bumps `pulseSeq` only when the
  /// click must actually restart, so an unchanged key means leave it alone —
  /// including across taps, gates and rep boundaries, which is what makes the
  /// pulse continuous (`specs/intrada-coach-engine.md` §6). Keyed on the
  /// block too: `pulseSeq` is block-scoped.
  @State private var startedPulse: Pulse?

  private struct Pulse: Equatable {
    let block: UInt64
    let seq: UInt32
  }

  private var drill: DrillView? { store.viewModel?.coach.drill }

  var body: some View {
    Group {
      if let drill {
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
      } else {
        Color.clear
      }
    }
    // The loop is a fullScreenCover, so RootView's banner is occluded while it's
    // up. Re-surface viewModel.error here — otherwise "Couldn't save what you
    // just practised." renders nowhere and a lost block is a silent no-op
    // (#846, #1181). On the host rather than the entry point, so the real
    // press-start route into the loop (#1182) inherits it.
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
    .onChange(of: drill == nil) { _, ended in
      // The core closed the block — ceiling, ladder or gate. It has already
      // ended the session, so this only tears down.
      if ended { close() }
    }
    // There is no `UIBackgroundModes: audio`, so a backgrounded app's poll task
    // freezes while its scheduled host times run on. Stop the pulse on the way
    // out rather than let it be resumed stale, and start a fresh one — count-in
    // first — on the way back.
    .onChange(of: scenePhase) { _, phase in
      if phase == .active {
        syncClick()
      } else {
        silencePulse()
      }
    }
  }

  private func run() async {
    // A blob means the last session was cut off mid-block: hand it back rather
    // than starting fresh, or the evidence already banked is discarded (#1181).
    let now = SessionClock.nowRFC3339()
    if let crashed = store.pendingCoachSession() {
      store.send(.coach(.recoverSession(session: crashed, now: now)))
    } else {
      store.send(.coach(.startPlannedSession(now: now)))
    }
    // No block came back — the core could not plan one, and has said why. Now
    // that press-start is a user path (#1182), holding a blank screen instead
    // of returning is the #846 silent no-op.
    guard drill != nil else { return close() }
    syncClick()
    while !Task.isCancelled {
      try? await Task.sleep(for: .seconds(1))
      guard !Task.isCancelled else { return }
      store.send(.coach(.tick(now: SessionClock.nowRFC3339())))
    }
  }

  /// The shell's whole click rule, with no domain reasoning in it.
  private func syncClick() {
    guard let drill else { return }
    guard drill.pulseRunning else { return silencePulse() }
    let pulse = Pulse(block: drill.blockIndex, seq: drill.pulseSeq)
    guard startedPulse != pulse else { return }

    // Only a sounding pulse is a consumed one: bailing out with `startedPulse`
    // set would leave a silent, permanently frozen screen (the #846 class).
    guard let engine = clickEngine() else { return unavailable() }
    guard engine.start(drill: drill, store: store) else { return unavailable() }
    startedPulse = pulse
  }

  /// Stops the click and forgets the key, so no beat is reported while nothing
  /// is sounding. That is what keeps the evidence honest: a core that hears no
  /// beats cannot count passes the player never heard, and the next
  /// `syncClick` starts a fresh pulse rather than resuming a dead one.
  private func silencePulse() {
    click?.stop()
    startedPulse = nil
  }

  private func clickEngine() -> Click? {
    if let click { return click }
    do {
      let created = try Click()
      created.onPulseStopped = { silencePulse() }
      // Only ever back into a foreground app: restarting into an interruption
      // that is still running would fail the session activation and take the
      // whole session down through `unavailable()`.
      created.onPulseShouldRestart = {
        silencePulse()
        if scenePhase == .active { syncClick() }
      }
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
    teardown()
    onClose()
  }

  private func dismiss() {
    store.send(.coach(.leaveSession(now: SessionClock.nowRFC3339())))
    close()
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

/// The view fields the click is driven by. Anything else moving in the drill
/// view — a tap, a gate, a beat — must not reach `syncClick`.
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

  /// The pulse died and cannot be restarted yet — an interruption is running.
  var onPulseStopped: (() -> Void)? {
    get { engine.onPulseStopped }
    set { engine.onPulseStopped = newValue }
  }

  /// The pulse died and a fresh one may start.
  var onPulseShouldRestart: (() -> Void)? {
    get { engine.onPulseShouldRestart }
    set { engine.onPulseShouldRestart = newValue }
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
    do {
      try engine.start(
        bpm: Double(drill.tempoBpm), beatsPerBar: Int(drill.beatsPerBar),
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

  /// Stops for good: unhooks the interruption callbacks, which `stop()`
  /// deliberately leaves live because a stopped pulse is exactly when they still
  /// need to speak.
  func dispose() {
    onPulseStopped = nil
    onPulseShouldRestart = nil
    engine.onCountIn = nil
    engine.onBeat = nil
    engine.dispose()
  }
}
