import SwiftUI

/// The switch in a card row, bound straight to a core value so a refused write
/// never shows (#1736). Same shape as `EntrySettingsSheet`'s setting toggle.
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
