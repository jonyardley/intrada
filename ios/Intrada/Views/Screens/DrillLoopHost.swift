import SharedTypes
import SwiftUI

/// Runs `DrillScreen` against the real click. Replaces the Swift-sequencing
/// harness this file grew out of (#1176): counting, gating, the escalation
/// ladder and what-comes-next all live in the core's session state machine
/// (`specs/intrada-coach-engine.md` §4). What is left here is I/O — schedule
/// the clicks the core asked for, report the ones that sounded, report taps and
/// seconds. No decisions.
struct DrillLoopHost: View {
  @Environment(Store.self) private var store

  var onClose: () -> Void

  @State private var click: Click?
  /// Last `repSeq` the click was started for, so a re-render mid-rep doesn't
  /// restart it and a new rep always does.
  @State private var startedRep: UInt32?

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
    .onChange(of: drill?.repSeq) { _, _ in startClickIfNeeded() }
    .onChange(of: drill == nil) { _, ended in
      // The core closed the block — ceiling, ladder or gate. Nothing left to draw.
      if ended { dismiss() }
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
    guard let drill, startedRep != drill.repSeq else { return }
    startedRep = drill.repSeq

    let engine: Click
    if let click {
      engine = click
    } else {
      do {
        engine = try Click()
      } catch {
        report(error, "drill-click")
        return
      }
      click = engine
    }
    engine.start(drill: drill, store: store)
  }

  private func dismiss() {
    store.send(.coach(.endBlock(now: SessionClock.nowRFC3339())))
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

  func start(drill: DrillView, store: Store) {
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
    } catch {
      report(error, "drill-click-start")
    }
  }

  func stop() {
    engine.onCountIn = nil
    engine.onBeat = nil
    engine.stop()
  }
}
