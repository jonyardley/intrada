import SwiftUI

struct FormErrorBanner: View {
  let message: String

  var body: some View {
    HStack(alignment: .top, spacing: IntradaSpacing.controlGap) {
      Image(systemName: "exclamationmark.triangle.fill")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.danger)
      Text(message)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.danger)
        .frame(maxWidth: .infinity, alignment: .leading)
    }
    .padding(IntradaSpacing.cardCompact)
    .background(IntradaColor.danger.opacity(0.10), in: RoundedRectangle(cornerRadius: IntradaRadius.card))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card).strokeBorder(IntradaColor.danger.opacity(0.25))
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Error: \(message)")
  }
}

#if DEBUG
  #Preview {
    FormErrorBanner(message: "A piece needs a composer.")
      .padding()
      .background(IntradaColor.paperTop)
  }
#endif
