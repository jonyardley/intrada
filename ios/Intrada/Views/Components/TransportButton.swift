import SwiftUI

/// A round Focus Player transport control: the ink `primary` advance and the
/// bare `secondary` skip beside it.
struct TransportButton: View {
  enum Prominence {
    case primary, secondary
  }

  let systemImage: String
  let prominence: Prominence
  let label: String
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      switch prominence {
      case .primary:
        Image(systemName: systemImage)
          .iconSize(.transport)
          .foregroundStyle(IntradaColor.onAccent)
          .frame(width: 78, height: 78)
          .background(LinearGradient.inkBar)
          .clipShape(Circle())
          .dropShadow(.transport)
      case .secondary:
        Image(systemName: systemImage)
          .iconSize(.control)
          .foregroundStyle(IntradaColor.inkSecondary)
          .frame(width: 48, height: 48)
      }
    }
    .buttonStyle(PressRebound())
    .accessibilityLabel(label)
  }
}

#if DEBUG
  #Preview {
    HStack(spacing: 32) {
      TransportButton(systemImage: "play.fill", prominence: .primary, label: "Next item") {}
      TransportButton(systemImage: "forward.end", prominence: .secondary, label: "Skip this item") {
      }
    }
    .padding(IntradaSpacing.section)
  }
#endif
