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
