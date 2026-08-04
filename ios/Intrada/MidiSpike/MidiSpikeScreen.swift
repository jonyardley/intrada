import SwiftUI

/// Debug-only tab (#if DEBUG in RootView) switching between the two spike
/// screens. Deliberately plain — no ScreenScaffold/design tokens, since this
/// is throwaway debug tooling, not a shipped screen.
struct MidiSpikeScreen: View {
  private enum Mode: String, CaseIterable, Identifiable {
    case capture = "Capture"
    case drill = "Gate Drill"
    case loop = "Drill Loop"
    var id: String { rawValue }
  }

  @State private var mode: Mode = .capture
  @State private var runningLoop = false

  var body: some View {
    VStack(spacing: 0) {
      Picker("Mode", selection: $mode) {
        ForEach(Mode.allCases) { mode in
          Text(mode.rawValue).tag(mode)
        }
      }
      .pickerStyle(.segmented)
      .padding()

      switch mode {
      case .capture:
        MidiDebugScreen()
      case .drill:
        GateDrillScreen()
      case .loop:
        VStack(spacing: 12) {
          Text("A2 / A3 full-screen, driven by the real click.")
            .font(.footnote)
            .foregroundStyle(.secondary)
            .multilineTextAlignment(.center)
          Button("Run the drill loop") { runningLoop = true }
            .buttonStyle(.borderedProminent)
          Spacer()
        }
        .padding()
      }
    }
    .fullScreenCover(isPresented: $runningLoop) {
      DrillLoopHarness(onClose: { runningLoop = false })
    }
  }
}

#Preview {
  NavigationStack {
    MidiSpikeScreen()
  }
}
