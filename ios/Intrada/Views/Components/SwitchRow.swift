import SwiftUI

/// The switch in a card row; bind it to the core's value when the setting is
/// one, so a refused write never shows (#1736).
struct SwitchRow: View {
  let title: String
  let isOn: Binding<Bool>
  let identifier: String

  var body: some View {
    Toggle(isOn: isOn) {
      FieldLabel(title)
    }
    .tint(IntradaColor.accent)
    .accessibilityIdentifier(identifier)
  }
}
