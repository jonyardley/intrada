import SharedTypes
import SwiftUI

/// The player's key picker (#2788). Picking the key already playing is the
/// core's no-op, as for a variation.
struct KeyPickerSheet: View {
  let itemTitle: String
  let choices: [KeyChoiceView]
  let current: Key?
  /// Returns false when the core refused the switch, which keeps the sheet up.
  let onPick: (Key?) -> Bool

  @Environment(\.dismiss) private var dismiss

  var body: some View {
    BottomSheet(title: "Switch key", detents: [.medium, .large]) {
      ScrollView {
        VStack(alignment: .leading, spacing: 0) {
          KeyChoiceRows(
            choices: choices, current: current, hint: "Switches \(itemTitle) to this key",
            identifier: "keyPicker.row"
          ) { key in
            if onPick(key) { dismiss() }
          }
          Text("Switching starts a fresh count of repetitions.")
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .padding(IntradaSpacing.card)
        }
        .padding(.top, IntradaSpacing.controlGap)
      }
    }
  }
}
