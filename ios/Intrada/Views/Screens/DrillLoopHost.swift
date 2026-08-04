import SharedTypes
import SwiftUI

/// Runs `DrillScreen` against the real click. Pure I/O: schedule the clicks the
/// core asked for, report the ones that sounded, report taps and seconds. Every
/// decision — counting, gating, the escalation ladder, what comes next — is in
/// the core's session state machine (`specs/intrada-coach-engine.md` §4).
struct DrillLoopHost: View {
  @Environment(Store.self) private var store

  var onClose: () -> Void

  @State private var click: Click?
  /// Which rep the click is running, so a re-render mid-rep doesn't restart it
  /// and a new one always does. Keyed on the block too: `repSeq` restarts at 1
  /// in each block, so on its own it can miss a boundary.
  @State private var startedRep: Rep?

  private struct Rep: Equatable {
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
          onDismiss: dismiss)
      } else {
        Color.clear
      }
    }
    .task { await run() }
    .onDisappear(perform: teardown)
    .onChange(of: drill.map { Rep(block: $0.blockIndex, seq: $0.repSeq) }) { _, _ in
      startClickIfNeeded()
    }
    .onChange(of: drill == nil) { _, ended in
      // The core closed the block — ceiling, ladder or gate. It has already
      // ended the session, so this only tears down.
      if ended { close() }
    }
  }

  private func run() async {
    store.send(.coach(.startDrillLoop(now: SessionClock.nowRFC3339())))
    startClickIfNeeded()
    while !Task.isCancelled {
      try? await Task.sleep(for: .seconds(1))
      guard !Task.isCancelled else { return }
      store.send(.coach(.tick(now: SessionClock.nowRFC3339())))
    }
  }

  private func startClickIfNeeded() {
    guard let drill else { return }
    let rep = Rep(block: drill.blockIndex, seq: drill.repSeq)
    guard startedRep != rep else { return }

    // Only a started rep is a consumed one: bailing out with `startedRep` set
    // would leave a silent, permanently frozen screen (the #846 class).
    guard let engine = clickEngine() else { return unavailable() }
    guard engine.start(drill: drill, store: store) else { return unavailable() }
    startedRep = rep
  }

  private func clickEngine() -> Click? {
    if let click { return click }
    do {
      let created = try Click()
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
    click?.stop()
    click = nil
    startedRep = nil
  }
}

/// A reference wrapper so the callbacks below outlive a SwiftUI body pass, and
/// so the AVAudioEngine survives a re-render mid-rep.
@MainActor
private final class Click {
  private let engine: ClickEngine

  init() throws {
    engine = try ClickEngine()
  }

  /// `false` = the click could not be scheduled, so this rep never sounded.
  func start(drill: DrillView, store: Store) -> Bool {
    engine.onCountIn = { remaining in
      store.send(.coach(.countInBeat(remaining: UInt8(max(0, remaining)))))
    }
    engine.onBeat = { index, _, _, _ in
      store.send(.coach(.beat(beatIndex: UInt32(max(0, index)))))
    }
    do {
      _ = try engine.start(
        bpm: Double(drill.tempoBpm), beatsPerBar: Int(drill.beatsPerBar),
        countInBeats: Int(drill.countInBeats), bodyBeats: Int(drill.clickBeats))
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
}
