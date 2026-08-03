import SwiftUI

@Observable
@MainActor
final class MidiDebugModel {
  var transport: TransportType = .usb
  var bpm: Double = 92
  private(set) var isRunning = false
  private(set) var recentOffsets: [String] = []
  private(set) var summary: TakeSummary?
  private(set) var exportURL: URL?
  private(set) var errorMessage: String?

  private var click: ClickEngine?
  private let midi = MidiCaptureService()
  private var grid: BeatGrid?
  private var capturedEvents: [NoteEvent] = []

  /// Long enough to cover a few minutes of free playing without rescheduling.
  private let bodyBeats = 480

  func start() {
    errorMessage = nil
    summary = nil
    exportURL = nil
    recentOffsets = []
    capturedEvents = []

    do {
      let engine = try click ?? ClickEngine()
      click = engine
      let newGrid = try engine.start(
        bpm: bpm, beatsPerBar: 4, countInBeats: 4, bodyBeats: bodyBeats)
      grid = newGrid

      midi.onNoteEvent = { [weak self] event in
        Task { @MainActor in self?.recordLive(event) }
      }
      try midi.start()
      isRunning = true
    } catch {
      errorMessage = "\(error)"
    }
  }

  func stop() {
    click?.stop()
    midi.stop()
    isRunning = false

    guard let grid else { return }
    let annotated = TakeRecorder.annotate(capturedEvents, against: grid)
    summary = TakeRecorder.summary(of: annotated)

    let header = TakeHeader(
      transport: transport, bpm: grid.bpm, beatsPerBar: grid.beatsPerBar,
      countInBeats: grid.countInBeats, startHostTime: grid.startHostTime,
      hostTimebaseNumer: HostClock.timebase.numer, hostTimebaseDenom: HostClock.timebase.denom,
      recordedAt: Date())
    do {
      exportURL = try TakeRecorder.write(header: header, events: annotated)
    } catch {
      errorMessage = "Export failed: \(error)"
    }
  }

  private func recordLive(_ event: NoteEvent) {
    capturedEvents.append(event)
    guard event.isNoteOn, let grid else { return }
    let nearest = grid.nearestBeat(for: event.hostTime)
    let sign = nearest.offsetMs >= 0 ? "+" : ""
    let line =
      "note \(event.midiNote) — bar \(nearest.bar) beat \(nearest.beat) — \(sign)\(Int(nearest.offsetMs))ms"
    recentOffsets.insert(line, at: 0)
    if recentOffsets.count > 20 {
      recentOffsets.removeLast()
    }
  }
}

struct MidiDebugScreen: View {
  @State private var model = MidiDebugModel()
  @State private var showingBluetoothPairing = false

  var body: some View {
    VStack(alignment: .leading, spacing: 16) {
      Picker("Transport", selection: $model.transport) {
        Text("USB").tag(TransportType.usb)
        Text("Bluetooth").tag(TransportType.bluetooth)
      }
      .pickerStyle(.segmented)
      .disabled(model.isRunning)

      if model.transport == .bluetooth {
        // BLE-MIDI peripherals never appear in Settings -> Bluetooth; they're
        // paired through this app-hosted system panel instead.
        Button("Pair Bluetooth MIDI…") { showingBluetoothPairing = true }
          .disabled(model.isRunning)
      }

      HStack {
        Text("BPM")
        Slider(value: $model.bpm, in: 40...200, step: 1)
        Text("\(Int(model.bpm))")
      }
      .disabled(model.isRunning)

      Button(model.isRunning ? "Stop" : "Start") {
        model.isRunning ? model.stop() : model.start()
      }
      .buttonStyle(.borderedProminent)

      if let errorMessage = model.errorMessage {
        Text(errorMessage).foregroundStyle(.red).font(.caption)
      }

      if let summary = model.summary {
        VStack(alignment: .leading, spacing: 4) {
          Text("Take summary").font(.headline)
          Text(
            "\(summary.noteOnCount) notes — mean \(String(format: "%.1f", summary.meanOffsetMs))ms, stdev \(String(format: "%.1f", summary.stdevOffsetMs))ms"
          )
          .font(.caption)
        }
        if let exportURL = model.exportURL {
          ShareLink(item: exportURL) {
            Label("Export take", systemImage: "square.and.arrow.up")
          }
        }
      }

      Text("Live offsets").font(.headline)
      List(model.recentOffsets, id: \.self) { line in
        Text(line).font(.system(.body, design: .monospaced))
      }
      .listStyle(.plain)
    }
    .padding()
    .navigationTitle("MIDI Capture")
    .sheet(isPresented: $showingBluetoothPairing) {
      BluetoothMIDIPairingSheet()
    }
  }
}

#Preview {
  NavigationStack {
    MidiDebugScreen()
  }
}
