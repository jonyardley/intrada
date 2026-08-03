import CoreAudioKit
import SwiftUI

/// Bluetooth MIDI (BLE-MIDI) peripherals never appear in iOS's system
/// Settings -> Bluetooth list — Apple only exposes pairing for them through
/// an app-hosted `CABTMIDICentralViewController`. This wraps that system
/// panel so the debug screen doesn't depend on GarageBand or another app
/// happening to be installed to establish the pairing.
struct BluetoothMIDIPairingSheet: View {
  @Environment(\.dismiss) private var dismiss

  var body: some View {
    NavigationStack {
      BluetoothMIDICentralView()
        .navigationTitle("Bluetooth MIDI Devices")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
          ToolbarItem(placement: .confirmationAction) {
            Button("Done") { dismiss() }
          }
        }
    }
  }
}

private struct BluetoothMIDICentralView: UIViewControllerRepresentable {
  func makeUIViewController(context: Context) -> CABTMIDICentralViewController {
    CABTMIDICentralViewController()
  }

  func updateUIViewController(_ uiViewController: CABTMIDICentralViewController, context: Context) {
  }
}
