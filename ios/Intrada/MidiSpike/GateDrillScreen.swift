import SwiftUI

/// Holds the engine/capture state as a reference type so closures (onBeat,
/// onNoteEvent) can safely mutate it without capturing a SwiftUI view value.
/// Not routed through the Crux Store — this is throwaway debug tooling with
/// no persistence.
@Observable
@MainActor
final class GateDrillModel {
  enum Phase {
    case countIn(remaining: Int)
    case playing
    case revealing(RepResult)
    case cleared
  }

  private(set) var phase: Phase = .countIn(remaining: GatePhrase.countInBeats)
  private(set) var passes = 0
  private(set) var beatFlashID = 0

  /// Spec decision 7 (transport-tiered scoring): Bluetooth's jitter doesn't
  /// support a fine timing verdict, so it's set explicitly rather than
  /// inferred — the drill needs to know before it scores a rep.
  var transport: TransportType = .usb

  private var click: ClickEngine?
  private let midi = MidiCaptureService()
  private var grid: BeatGrid?
  private var capturedNotes: [NoteEvent] = []
  private var repEndWorkItem: DispatchWorkItem?
  private var revealWorkItem: DispatchWorkItem?

  /// Extra beats of room *after* the phrase's last expected beat before a
  /// rep is scored, so a slightly late final note is scored as "dragging"
  /// rather than silently dropped. The old cutoff (the beat right after the
  /// phrase, i.e. bar 3 beat 1) gave a bare single beat of slack, which
  /// wasn't enough — a note landing even a bit behind that missed the
  /// window entirely and read as a mysteriously "wrong" note.
  private let repEndGraceBeats: Double = 2

  func start() {
    // Gated to .playing only — otherwise noodling during the count-in (or
    // during the ~1s reveal before the next rep) leaks notes into the next
    // rep's candidate pool and can steal a match slot from the real,
    // on-time note (GatePhrase.evaluate matches by pitch class, not a time
    // window).
    midi.onNoteEvent = { [weak self] event in
      Task { @MainActor in
        guard let self, case .playing = self.phase else { return }
        self.capturedNotes.append(event)
      }
    }
    do {
      try midi.start()
    } catch {
      // Non-fatal for the spike — the drill still runs, just with no notes
      // to score, which shows up honestly as "wrong notes" every rep.
    }
    startRep()
  }

  func stop() {
    repEndWorkItem?.cancel()
    revealWorkItem?.cancel()
    click?.stop()
    midi.stop()
  }

  private func startRep() {
    capturedNotes = []
    phase = .countIn(remaining: GatePhrase.countInBeats)

    do {
      let engine = try click ?? ClickEngine()
      click = engine
      engine.onCountIn = { [weak self] remaining in
        Task { @MainActor in self?.phase = .countIn(remaining: remaining) }
      }
      engine.onBeat = { [weak self] _, _, _ in
        Task { @MainActor in
          self?.phase = .playing
          self?.beatFlashID += 1
        }
      }
      let bodyBeats = GatePhrase.expected.count
      let newGrid = try engine.start(
        bpm: GatePhrase.bpm, beatsPerBar: GatePhrase.beatsPerBar,
        countInBeats: GatePhrase.countInBeats, bodyBeats: bodyBeats)
      grid = newGrid

      let lastBeatOfPhrase = GatePhrase.expected.last!
      let phraseEndHostTime = newGrid.hostTime(
        bar: lastBeatOfPhrase.bar, beat: lastBeatOfPhrase.beat)
      let graceTicks = HostClock.ticks(fromSeconds: repEndGraceBeats * newGrid.secondsPerBeat)
      let repEndHostTime = phraseEndHostTime &+ graceTicks
      let delaySeconds = max(0, HostClock.secondsBetween(repEndHostTime, HostClock.now()))
      let workItem = DispatchWorkItem { [weak self] in self?.finishRep() }
      repEndWorkItem = workItem
      DispatchQueue.main.asyncAfter(deadline: .now() + delaySeconds, execute: workItem)
    } catch {
      // Spike tooling: surfacing this as a wrong-notes rep is enough signal
      // to notice the engine failed to start.
      phase = .revealing(RepResult(verdict: .wrongNotes(count: GatePhrase.expected.count)))
    }
  }

  private func finishRep() {
    guard let grid else { return }
    let result = GatePhrase.evaluate(notes: capturedNotes, against: grid, transport: transport)
    phase = .revealing(result)
    passes = result.isPass ? passes + 1 : 0

    // Layer 1 is a ~1s glance, then hands never leave the keys — the next
    // count-in starts on its own. Tracked in `revealWorkItem` (like
    // `repEndWorkItem`) so `stop()` can cancel it — otherwise navigating
    // away during this window doesn't stop the drill, it just delays a
    // surprise restart.
    let workItem = DispatchWorkItem { [weak self] in
      guard let self else { return }
      if self.passes >= GatePhrase.gateTargetPasses {
        self.phase = .cleared
      } else {
        self.startRep()
      }
    }
    revealWorkItem = workItem
    DispatchQueue.main.asyncAfter(deadline: .now() + 1.0, execute: workItem)
  }

  func replay() {
    passes = 0
    startRep()
  }
}

struct GateDrillScreen: View {
  @State private var model = GateDrillModel()
  @State private var flashOpacity = 0.15

  var body: some View {
    VStack(spacing: 32) {
      Picker("Transport", selection: $model.transport) {
        Text("USB").tag(TransportType.usb)
        Text("Bluetooth").tag(TransportType.bluetooth)
      }
      .pickerStyle(.segmented)

      if model.transport == .bluetooth {
        // Spec decision 7: never issue a precision verdict the input can't
        // support — and say why, on screen, not just quietly withhold it.
        Text("Bluetooth: notes are scored, timing isn't — the connection's jitter is too coarse.")
          .font(.caption2)
          .foregroundStyle(.secondary)
      }

      VStack(spacing: 4) {
        Text("Play these 8 single notes in order — not a chord:")
          .font(.caption)
          .foregroundStyle(.secondary)
        Text(GatePhrase.displaySequence)
          .font(.system(.body, design: .monospaced))
          .multilineTextAlignment(.center)
        Text("One at a time, either hand, any octave. Strike each fresh on its beat.")
          .font(.caption2)
          .foregroundStyle(.secondary)
        Text("\(Int(GatePhrase.bpm)) bpm, crotchets")
          .font(.caption2)
          .foregroundStyle(.secondary)
      }
      gateDots
      Spacer()
      content
      Spacer()
    }
    .padding()
    .navigationTitle("Gate Drill")
    .onAppear { model.start() }
    .onDisappear { model.stop() }
    .onChange(of: model.beatFlashID) {
      // A one-shot flash-and-decay per beat, not a toggle — a toggle only
      // alternates between two brightness levels beat to beat, which reads
      // as random rather than "in time with the click".
      flashOpacity = 0.9
      withAnimation(.easeOut(duration: 0.3)) {
        flashOpacity = 0.15
      }
    }
  }

  // Monochrome throughout (design brief: "mastery is monochrome; the count
  // carries meaning" — no green/red, since colour must never be the sole,
  // or even the primary, carrier of a calm-vs-shaming distinction).
  private var gateDots: some View {
    HStack(spacing: 12) {
      ForEach(0..<GatePhrase.gateTargetPasses, id: \.self) { index in
        Circle()
          .fill(index < model.passes ? Color.primary : Color.primary.opacity(0.15))
          .frame(width: 20, height: 20)
      }
      Text("\(min(model.passes, GatePhrase.gateTargetPasses)) of \(GatePhrase.gateTargetPasses)")
        .font(.headline)
        .padding(.leading, 8)
    }
  }

  @ViewBuilder
  private var content: some View {
    switch model.phase {
    case .countIn(let remaining):
      // No animation/contentTransition here on purpose — a smoothing
      // transition on the number delays when it visually lands, which
      // reads as "out of sync" even though the state change itself fires
      // within ~10ms of the actual click (ClickEngine's poll loop). A
      // straight cut to the new digit is what "on the beat" looks like.
      VStack(spacing: 12) {
        Text("Get ready…").font(.headline).foregroundStyle(.secondary)
        Text("\(remaining)")
          .font(.system(size: 64, weight: .bold, design: .rounded))
      }
    case .playing:
      // Layer 0 — silence: a passive beat flash, no wrong-note indication,
      // no score counting up.
      VStack(spacing: 12) {
        Text("Play now").font(.headline).foregroundStyle(.secondary)
        Circle()
          .fill(Color.primary.opacity(flashOpacity))
          .frame(width: 80, height: 80)
      }
    case .revealing(let result):
      // Shape carries the pass/fail meaning (check vs cross); colour stays
      // monochrome so a miss reads as a calm fact, not a red shaming flash.
      VStack(spacing: 12) {
        Image(systemName: result.isPass ? "checkmark.circle.fill" : "xmark.circle.fill")
          .font(.system(size: 64))
          .foregroundStyle(result.isPass ? Color.primary : Color.primary.opacity(0.6))
        Text(result.fact)
          .font(.title2)
      }
    case .cleared:
      VStack(spacing: 16) {
        Text("Gate cleared").font(.largeTitle.bold())
        Button("Play again") { model.replay() }
      }
    }
  }
}

#Preview {
  NavigationStack {
    GateDrillScreen()
  }
}
