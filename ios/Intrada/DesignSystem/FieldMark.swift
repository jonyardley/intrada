import SwiftUI

/// Says a form field was filled from a photographed page rather than typed
/// (#1436). A weak read wears the same mark, dimmed — the read the user most
/// needs to check is the quietest thing on screen visually, so the difference
/// is spoken as well, never left to contrast alone.
struct FieldMark: View {
  let weak: Bool

  var body: some View {
    HStack(spacing: 5) {
      Image(systemName: "doc.viewfinder")
        .font(.system(size: 11, weight: .medium))
      Text("From the photo")
        .font(IntradaFont.micro)
        .fontWeight(.medium)
    }
    .foregroundStyle(weak ? IntradaColor.inkFaint : IntradaColor.inkSecondary)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(
      weak ? "Read from the photo, but not clearly. Worth checking" : "Read from the photo")
  }
}

#if DEBUG
  #Preview {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      FieldMark(weak: false)
      FieldMark(weak: true)
    }
    .padding()
    .background(IntradaColor.cardFill)
  }
#endif
