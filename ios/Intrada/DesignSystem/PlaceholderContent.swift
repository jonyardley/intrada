import SwiftUI

/// A centred "nothing here yet" body for screens whose real UI lands in the
/// screen-by-screen rewrite (Phase C). Tinted glyph + one line of muted copy.
struct PlaceholderContent: View {
  let systemImage: String
  let message: String
  @ScaledMetric(relativeTo: .largeTitle) private var glyphSize: CGFloat = 40

  var body: some View {
    VStack(spacing: 12) {
      Image(systemName: systemImage)
        .font(.system(size: glyphSize, weight: .regular))
        .foregroundStyle(IntradaColor.accent.opacity(0.55))
      Text(message)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkFaint)
        .multilineTextAlignment(.center)
    }
    .padding(32)
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .accessibilityElement(children: .combine)
    .accessibilityLabel(message)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      PlaceholderContent(
        systemImage: "music.note",
        message: "Start a focused practice session here.")
    }
  }
#endif
