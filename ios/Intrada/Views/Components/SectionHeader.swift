import SwiftUI

/// The uppercase, letter-spaced section label ("eyebrow") used above every
/// section on the refreshed screens. `inkFaint` is the one place that token is
/// allowed — eyebrows only (it fails AA for body text).
struct Eyebrow: View {
  @Environment(\.dynamicTypeSize) private var dynamicTypeSize

  let text: String
  // Defaults to inkFaint; override for an eyebrow on a dark/coloured surface
  // (the Practice hero, the dark summary headline); a trailing `.foregroundStyle`
  // can't override the inner Text, so the tint must be set here.
  var tint: Color = IntradaColor.inkFaint
  init(_ text: String, tint: Color = IntradaColor.inkFaint) {
    self.text = text
    self.tint = tint
  }

  var body: some View {
    // A spaced-out word wraps sooner, so accessibility sizes get tighter
    // tracking to keep it on one line (#1781).
    Text(text.uppercased())
      .font(IntradaFont.eyebrow)
      .tracking(dynamicTypeSize.isAccessibilitySize ? 0.5 : IntradaFont.eyebrowTracking)
      .foregroundStyle(tint)
      .accessibilityLabel(text)
  }
}

/// An eyebrow with an optional trailing caption (e.g. "THIS MONTH" · "best week ·
/// 95 min"). The trailing caption uses `inkSecondary` — real metadata, AA-safe.
/// `caption` sits against the eyebrow instead, for a count that qualifies the
/// title rather than commenting on the section ("USED IN · 3 pieces").
struct SectionHeader: View {
  @Environment(\.dynamicTypeSize) private var dynamicTypeSize

  let title: String
  var caption: String?
  var trailing: String?
  // Trailing action button (e.g. "Edit"/"Done"), matching the disabled/
  // hidden-when-empty pattern used across the header buttons on this screen.
  var actionTitle: String?
  var action: (() -> Void)?
  var actionAccessibilityLabel: String?
  var actionDisabled = false

  var body: some View {
    // The eyebrow breaks mid-word when it shares a line with trailing text (#1781).
    if dynamicTypeSize.isAccessibilitySize {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        if actionTitle != nil {
          HStack(alignment: .firstTextBaseline) {
            Eyebrow(title)
            Spacer(minLength: IntradaSpacing.controlGap)
            actionButton
          }
        } else {
          Eyebrow(title)
        }
        if let caption { meta(caption) }
        if let trailing { meta(trailing) }
      }
    } else {
      HStack(alignment: .firstTextBaseline) {
        Eyebrow(title)
        if let caption { meta("· \(caption)") }
        if let trailing {
          Spacer(minLength: IntradaSpacing.controlGap)
          meta(trailing)
        } else if actionTitle != nil {
          Spacer(minLength: IntradaSpacing.controlGap)
        }
        actionButton
      }
    }
  }

  @ViewBuilder private var actionButton: some View {
    if let actionTitle, let action {
      Button(actionTitle, action: action)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.accent)
        .disabled(actionDisabled)
        .opacity(actionDisabled ? 0 : 1)
        .accessibilityLabel(actionAccessibilityLabel ?? actionTitle)
    }
  }

  private func meta(_ text: String) -> some View {
    Text(text)
      .font(IntradaFont.meta)
      .foregroundStyle(IntradaColor.inkSecondary)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        Eyebrow("Recent mastery")
        SectionHeader(title: "This month", trailing: "best week · 95 min")
        SectionHeader(title: "Used in", caption: "3 pieces")
      }
      .padding(IntradaSpacing.card)
    }
  }

  #Preview("Accessibility size") {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        Eyebrow("Recent mastery")
        SectionHeader(title: "Variations", trailing: "5 of 15 solid")
        SectionHeader(title: "Used in", caption: "3 pieces")
      }
      .padding(IntradaSpacing.card)
    }
    .dynamicTypeSize(.accessibility5)
  }
#endif
