import SwiftUI

/// The uppercase, letter-spaced section label ("eyebrow") used above every
/// section on the refreshed screens. `inkFaint` is the one place that token is
/// allowed — eyebrows only (it fails AA for body text).
struct Eyebrow: View {
  let text: String
  // Defaults to inkFaint; override for an eyebrow on a dark/coloured surface
  // (the Practice hero, the gold summary headline) — a trailing `.foregroundStyle`
  // can't override the inner Text, so the tint must be set here.
  var tint: Color = IntradaColor.inkFaint
  init(_ text: String, tint: Color = IntradaColor.inkFaint) {
    self.text = text
    self.tint = tint
  }

  var body: some View {
    Text(text.uppercased())
      .font(IntradaFont.eyebrow)
      .tracking(1.5)
      .foregroundStyle(tint)
      .accessibilityLabel(text)
  }
}

/// An eyebrow with an optional trailing caption (e.g. "THIS MONTH" · "best week ·
/// 95 min"). The trailing caption uses `inkSecondary` — real metadata, AA-safe.
struct SectionHeader: View {
  let title: String
  var trailing: String?

  var body: some View {
    HStack(alignment: .firstTextBaseline) {
      Eyebrow(title)
      if let trailing {
        Spacer(minLength: IntradaSpacing.controlGap)
        Text(trailing)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        Eyebrow("Recent mastery")
        SectionHeader(title: "This month", trailing: "best week · 95 min")
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
