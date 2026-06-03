import SwiftUI

/// The shared shell every top-level screen is built from. The page title lives
/// in the content, not a UIKit nav bar (the locked *Library — Light* header).
struct ScreenScaffold<Content: View>: View {
  let title: String
  var subtitle: String?
  var trailing: TrailingAction?
  @ViewBuilder var content: Content

  struct TrailingAction {
    let label: String
    var systemImage: String = "plus"
    let action: () -> Void
  }

  init(
    title: String,
    subtitle: String? = nil,
    trailing: TrailingAction? = nil,
    @ViewBuilder content: () -> Content
  ) {
    self.title = title
    self.subtitle = subtitle
    self.trailing = trailing
    self.content = content()
  }

  var body: some View {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: 0) {
        header
        Rectangle()
          .fill(IntradaColor.divider)
          .frame(height: 1)
          .padding(.top, IntradaSpacing.cardCompact)
        content
          .frame(maxWidth: .infinity, maxHeight: .infinity)
      }
    }
    // Clamp the floor (avoid sub-readable text) but allow the full accessibility
    // range now that the filter tabs scroll instead of wrapping (#810).
    .dynamicTypeSize(.xSmall ... .accessibility5)
  }

  private var header: some View {
    HStack(alignment: .firstTextBaseline) {
      VStack(alignment: .leading, spacing: 3) {
        Text(title)
          .font(IntradaFont.pageTitle())
          .foregroundStyle(IntradaColor.ink)
        if let subtitle {
          Text(subtitle)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkFaint)
        }
      }
      // Combine only the title block so the trailing action stays its own
      // VoiceOver element rather than being merged into the heading.
      .accessibilityElement(children: .combine)
      .accessibilityLabel(subtitle.map { "\(title), \($0)" } ?? title)
      Spacer(minLength: 12)
      if let trailing {
        Button(action: trailing.action) {
          Image(systemName: trailing.systemImage)
            .font(.system(size: 16, weight: .semibold))
            .foregroundStyle(IntradaColor.onAccent)
            .frame(width: 30, height: 30)
            .background(IntradaColor.accent, in: Circle())
            .frame(width: 44, height: 44)
            .contentShape(Circle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(trailing.label)
        // Centre the circular button on the title's baseline rather than
        // letting it hang below it.
        .alignmentGuide(.firstTextBaseline) { $0[VerticalAlignment.center] }
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.controlGap)
  }
}

#if DEBUG
  #Preview {
    ScreenScaffold(
      title: "Library",
      subtitle: "12 items",
      trailing: .init(label: "Add", action: {})
    ) {
      PlaceholderContent(
        systemImage: "books.vertical",
        message: "Your pieces and exercises will live here.")
    }
  }
#endif
