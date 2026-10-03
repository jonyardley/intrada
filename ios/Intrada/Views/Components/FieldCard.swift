import SwiftUI

/// A field's name, inside its card above the value (T35 in
/// `docs/design-principles.md`).
struct FieldLabel: View {
  let text: String
  var tint: Color = IntradaColor.inkSecondary
  init(_ text: String, tint: Color = IntradaColor.inkSecondary) {
    self.text = text
    self.tint = tint
  }

  var body: some View {
    Text(text)
      .font(IntradaFont.label)
      .foregroundStyle(tint)
  }
}

/// A labelled control on a form or sheet: one card, the label above whatever
/// sets the value.
struct FieldCard<Content: View>: View {
  let label: String
  let content: Content
  init(_ label: String, @ViewBuilder content: () -> Content) {
    self.label = label
    self.content = content()
  }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      FieldLabel(label)
      content
    }
    .padding(.vertical, 10)
    .padding(.horizontal, IntradaSpacing.card)
    .frame(maxWidth: .infinity, alignment: .leading)
    .cardSurface()
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.section) {
        FieldCard("Beats in the bar") {
          Text("4").font(IntradaFont.body).foregroundStyle(IntradaColor.ink)
        }
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
