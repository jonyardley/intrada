import SwiftUI

/// Debug-only tab (#if DEBUG in RootView) switching between the two spike
/// screens. Deliberately plain — no ScreenScaffold/design tokens, since this
/// is throwaway debug tooling, not a shipped screen.
struct MidiSpikeScreen: View {
  // The file itself builds in Release (only its call site in RootView is
  // DEBUG-gated), so anything reaching into DEBUG-only code has to be guarded
  // here too — the per-PR CI job builds Debug and won't catch it.
  private enum Mode: String, CaseIterable, Identifiable {
    case capture = "Capture"
    case drill = "Gate Drill"
    #if DEBUG
      case loop = "Drill Loop"
    #endif
    var id: String { rawValue }
  }

  @State private var mode: Mode = .capture
  #if DEBUG
    @State private var runningLoop = false
  #endif

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
      #if DEBUG
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
      #endif
      }
    }
    #if DEBUG
      .fullScreenCover(isPresented: $runningLoop) {
        DrillLoopHarness(onClose: { runningLoop = false })
      }
    #endif
  }
}

#Preview {
  NavigationStack {
    MidiSpikeScreen()
  }
}
