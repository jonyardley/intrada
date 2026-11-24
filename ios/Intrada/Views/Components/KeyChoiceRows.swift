import SharedTypes
import SwiftUI

/// The key rows Build session and the player's key chip share, so the two
/// lists cannot drift apart (#2788).
struct KeyChoiceRows: View {
  let choices: [KeyChoiceView]
  let current: Key?
  let hint: String
  let identifier: String
  let onPick: (Key?) -> Void

  var body: some View {
    VStack(spacing: 0) {
      ForEach(Array(choices.enumerated()), id: \.element.label) { index, choice in
        PickerRow(
          label: choice.label, caption: choice.caption, isCurrent: choice.key == current,
          hint: hint, identifier: identifier
        ) {
          onPick(choice.key)
        }
        if index < choices.count - 1 {
          HairlineDivider().padding(.leading, IntradaSpacing.card)
        }
      }
    }
  }
}

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
