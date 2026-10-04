import SwiftUI

/// A small bordered action that sits inside a row beside its content, such as
/// "Link" on a Used in row or "Sections" on a related exercise.
struct CapsuleActionButton: View {
  let title: String
  let action: () -> Void

  var body: some View {
    Button(action: action) {
      Text(title)
        .font(IntradaFont.badge)
        .foregroundStyle(IntradaColor.accent)
        // 10/5 are capsule-specific insets, below the token scale floor.
        .padding(.horizontal, 10)
        .padding(.vertical, 5)
        .background(
          RoundedRectangle(cornerRadius: IntradaRadius.badge)
            .fill(IntradaColor.surfaceSunken)
            .stroke(IntradaColor.divider, lineWidth: 1))
    }
    .buttonStyle(.plain)
  }
}
