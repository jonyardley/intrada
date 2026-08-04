#if DEBUG
  import SwiftUI

  /// Runs `DrillScreen` against the real click so the loop can be felt at a
  /// piano before the coach engine exists — the brief's own reason for shipping
  /// A2/A3 first ("A1 and A4 will be better designed once you have used A2 and
  /// A3 at a real piano").
  ///
  /// DEBUG-only scaffolding, and the sequencing below is exactly the part that
  /// does **not** ship: from Phase 2a the core's session state machine
  /// (`specs/intrada-coach-engine.md` §4) owns counting, gating and what comes
  /// next, and this file goes away.
  @Observable
  @MainActor
  final class DrillLoopHarnessModel {
    private(set) var state = DrillLoopState(
      drillTitle: "Rootless voicings",
      section: "A section",
      destination: "Strasbourg / St. Denis",
      tempoBpm: 120,
      clickLevel: "beats 2 & 4",
      bars: 4,
      elapsedSeconds: 0, ceilingSeconds: 360,
      blockKinds: [.piece, .exercise, .exercise, .piece, .piece], blockIndex: 1,
      gateQuestion: "Clean at 120?",
      gateSummary: "3 clean at 120",
      gateFilled: 0, gateTarget: 3)

    private var click: ClickEngine?
    private var repEnd: DispatchWorkItem?
    private var gateHold: DispatchWorkItem?
    private var ticker: Task<Void, Never>?

    func start() {
      ticker = Task { @MainActor [weak self] in
        while !Task.isCancelled {
          try? await Task.sleep(for: .seconds(1))
          self?.state.elapsedSeconds += 1
        }
      }
      startRep()
    }

    func stop() {
      repEnd?.cancel()
      gateHold?.cancel()
      ticker?.cancel()
      click?.stop()
    }

    func verdict(clean: Bool) {
      if clean { state.gateFilled += 1 }
      if state.gateFilled >= state.gateTarget {
        state.phase = .gateOpen
        let hold = DispatchWorkItem { [weak self] in
          guard let self else { return }
          self.state.gateFilled = 0
          self.startRep()
        }
        gateHold = hold
        DispatchQueue.main.asyncAfter(deadline: .now() + 2, execute: hold)
      } else {
        state.phase = .acknowledged(clean: clean, countInRemaining: state.countInBeats)
        startRep()
      }
    }

    /// The stuck ladder is Phase 2b; here it just drops the tempo, which is the
    /// first rung and enough to prove the target is reachable without looking.
    func stuck() {
      state.tempoBpm = max(60, state.tempoBpm - 20)
      state.gateQuestion = "Clean at \(state.tempoBpm)?"
      state.gateSummary = "\(state.gateTarget) clean at \(state.tempoBpm)"
      startRep()
    }

    private func startRep() {
      repEnd?.cancel()
      let bodyBeats = state.bars * state.beatsPerBar
      do {
        let engine = try click ?? ClickEngine()
        click = engine
        engine.onCountIn = { [weak self] remaining in
          Task { @MainActor in
            guard let self, case .acknowledged(let clean, _) = self.state.phase else { return }
            self.state.phase = .acknowledged(clean: clean, countInRemaining: remaining)
          }
        }
        engine.onBeat = { [weak self] bar, beat, _ in
          Task { @MainActor in
            guard let self else { return }
            self.state.phase = .playing
            self.state.bar = bar
            self.state.beat = beat
          }
        }
        let grid = try engine.start(
          bpm: Double(state.tempoBpm), beatsPerBar: state.beatsPerBar,
          countInBeats: state.countInBeats, bodyBeats: bodyBeats)
        let end = DispatchWorkItem { [weak self] in self?.state.phase = .awaitingVerdict }
        repEnd = end
        DispatchQueue.main.asyncAfter(
          deadline: .now() + Double(bodyBeats) * grid.secondsPerBeat + 0.4, execute: end)
      } catch {
        state.phase = .awaitingVerdict
      }
    }
  }

  struct DrillLoopHarness: View {
    @State private var model = DrillLoopHarnessModel()

    var onClose: () -> Void

    var body: some View {
      DrillScreen(
        state: model.state,
        onVerdict: { model.verdict(clean: $0) },
        onStuck: { model.stuck() },
        onDismiss: onClose
      )
      .onAppear { model.start() }
      .onDisappear { model.stop() }
    }
  }
#endif
