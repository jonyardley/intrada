import SwiftUI

/// A themed labelled text field for forms: faint label over an ink input.
/// 16pt input minimum avoids iOS zoom-on-focus (CLAUDE.md iOS rules).
struct FormField: View {
  let label: String
  @Binding var text: String
  var placeholder: String = ""
  var axis: Axis = .horizontal
  var keyboard: UIKeyboardType = .default
  var autocapitalization: TextInputAutocapitalization = .sentences

  var body: some View {
    VStack(alignment: .leading, spacing: 4) {
      Text(label)
        .font(.system(size: 12, weight: .medium))
        .foregroundStyle(IntradaColor.inkFaint)
      TextField(placeholder, text: $text, axis: axis)
        .font(.system(size: 16))
        .foregroundStyle(IntradaColor.ink)
        .keyboardType(keyboard)
        .textInputAutocapitalization(autocapitalization)
    }
    .padding(.vertical, 10)
    .padding(.horizontal, 16)
    .frame(maxWidth: .infinity, alignment: .leading)
  }
}
