import SwiftUI

/// Names a section, sentence case, above its card (T35 in
/// `docs/design-principles.md`).
struct SectionTitle: View {
  let text: String
  // Override for a section title on a dark or coloured surface (the Practice
  // and Up next heroes); a trailing `.foregroundStyle` can't
  // override the inner Text, so the tint must be set here.
  var tint: Color = IntradaColor.inkSecondary
  init(_ text: String, tint: Color = IntradaColor.inkSecondary) {
    self.text = text
    self.tint = tint
  }

  var body: some View {
    FieldLabel(text, tint: tint)
  }
}

/// A section title with an optional trailing caption ("This month", "best week ·
/// 95 min"). `caption` sits against the title instead, for a count that
/// qualifies it rather than commenting on the section ("Used in · 3 pieces").
struct SectionHeader: View {
  struct Action {
    let title: String
    var accessibilityLabel: String?
    var isDisabled = false
    var identifier: String?
    let perform: () -> Void
  }

  @Environment(\.dynamicTypeSize) private var dynamicTypeSize

  let title: String
  var caption: String?
  // VoiceOver already hears the list's length, so a bare count is noise there.
  var captionAccessibilityHidden = false
  var trailing: String?
  var action: Action?

  var body: some View {
    // The title breaks mid-word when it shares a line with trailing text (#1781).
    if dynamicTypeSize.isAccessibilitySize {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        if action != nil {
          HStack(alignment: .firstTextBaseline) {
            SectionTitle(title)
            Spacer(minLength: IntradaSpacing.controlGap)
            actionButton
          }
        } else {
          SectionTitle(title)
        }
        if let caption { captionView(caption) }
        if let trailing { meta(trailing) }
      }
    } else {
      HStack(alignment: .firstTextBaseline) {
        SectionTitle(title)
        if let caption { captionView("· \(caption)") }
        if let trailing {
          Spacer(minLength: IntradaSpacing.controlGap)
          meta(trailing)
        } else if action != nil {
          Spacer(minLength: IntradaSpacing.controlGap)
        }
        actionButton
      }
    }
  }

  @ViewBuilder private func captionView(_ text: String) -> some View {
    if captionAccessibilityHidden {
      meta(text).accessibilityHidden(true)
    } else {
      meta(text)
    }
  }

  @ViewBuilder private var actionButton: some View {
    if let action {
      Button(action.title, action: action.perform)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.accent)
        .disabled(action.isDisabled)
        .opacity(action.isDisabled ? 0 : 1)
        .accessibilityLabel(action.accessibilityLabel ?? action.title)
        .accessibilityHidden(action.isDisabled)
        .accessibilityIdentifier(action.identifier ?? "")
    }
  }

  private func meta(_ text: String) -> some View {
    Text(text)
      .font(IntradaFont.secondary)
      .foregroundStyle(IntradaColor.inkSecondary)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        SectionTitle("Recent mastery")
        SectionHeader(title: "This month", trailing: "best week · 95 min")
        SectionHeader(title: "Used in", caption: "3 pieces")
        SectionHeader(title: "Chord chart", action: .init(title: "Edit", perform: {}))
      }
      .padding(IntradaSpacing.card)
    }
  }

  #Preview("Accessibility size") {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        SectionTitle("Recent mastery")
        SectionHeader(title: "Variations", trailing: "5 of 15 solid")
        SectionHeader(title: "Used in", caption: "3 pieces")
        SectionHeader(title: "Chord chart", action: .init(title: "Edit", perform: {}))
      }
      .padding(IntradaSpacing.card)
    }
    .dynamicTypeSize(.accessibility5)
  }
#endif
