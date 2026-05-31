import SwiftUI

/// The shared shell every top-level screen is built from: paper background, a
/// serif page title with optional subtitle and a trailing action, a hairline
/// rule, then the screen's content. Matches the locked *Library — Light* header
/// (the title lives in the content, not a UIKit nav bar).
struct ScreenScaffold<Content: View>: View {
  let title: String
  var subtitle: String?
  var trailing: TrailingAction?
  @ViewBuilder var content: Content

  struct TrailingAction {
    let label: String
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
          .padding(.top, 12)
        content
          .frame(maxWidth: .infinity, maxHeight: .infinity)
      }
    }
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
        Button(trailing.label, action: trailing.action)
          .font(.system(size: 14, weight: .medium))
          .foregroundStyle(IntradaColor.accent)
      }
    }
    .padding(.horizontal, 16)
    .padding(.top, 8)
  }
}

#if DEBUG
  #Preview {
    ScreenScaffold(
      title: "Library",
      subtitle: "12 items",
      trailing: .init(label: "+ Add", action: {})
    ) {
      PlaceholderContent(
        systemImage: "books.vertical",
        message: "Your pieces and exercises will live here.")
    }
  }
#endif
