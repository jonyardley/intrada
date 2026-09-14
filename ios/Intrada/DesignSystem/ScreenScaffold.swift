import SwiftUI

/// The shared shell every top-level screen is built from. The page title lives
/// in the content, not the native nav bar's title (the locked *Library, Light*
/// header). Every screen forces `.inline` display with an empty native
/// title (#1724, #1822), so the bar is the same fixed ~44pt height everywhere
/// rather than a large title on some screens and none on others; on a pushed
/// screen this also keeps the real native back chevron and its edge-swipe
/// gesture, which a custom leading view could never reproduce. `leadingContent`
/// stays for a screen whose leading action is not "back" (Build session's
/// Cancel, which must intercept an unsaved plan rather than pop silently).
struct ScreenScaffold<Content: View, Leading: View, Trailing: View>: View {
  let title: String
  var subtitle: String?
  var trailing: TrailingAction?
  let leadingContent: Leading
  /// A trailing view of the screen's own (the Practice profile badge, #1692);
  /// it sits where `trailing`'s circular button would.
  let trailingContent: Trailing
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
  ) where Leading == EmptyView, Trailing == EmptyView {
    self.title = title
    self.subtitle = subtitle
    self.trailing = trailing
    self.leadingContent = EmptyView()
    self.trailingContent = EmptyView()
    self.content = content()
  }

  init(
    title: String,
    subtitle: String? = nil,
    @ViewBuilder trailingContent: () -> Trailing,
    @ViewBuilder content: () -> Content
  ) where Leading == EmptyView {
    self.title = title
    self.subtitle = subtitle
    self.trailing = nil
    self.leadingContent = EmptyView()
    self.trailingContent = trailingContent()
    self.content = content()
  }

  init(
    title: String,
    subtitle: String? = nil,
    @ViewBuilder leadingContent: () -> Leading,
    trailing: TrailingAction? = nil,
    @ViewBuilder content: () -> Content
  ) where Trailing == EmptyView {
    self.title = title
    self.subtitle = subtitle
    self.trailing = trailing
    self.leadingContent = leadingContent()
    self.trailingContent = EmptyView()
    self.content = content()
  }

  init(
    title: String,
    subtitle: String? = nil,
    @ViewBuilder leadingContent: () -> Leading,
    @ViewBuilder trailingContent: () -> Trailing,
    @ViewBuilder content: () -> Content
  ) {
    self.title = title
    self.subtitle = subtitle
    self.trailing = nil
    self.leadingContent = leadingContent()
    self.trailingContent = trailingContent()
    self.content = content()
  }

  var body: some View {
    // The background sizes and the content floats: a ZStack used to size to an
    // un-shrinkable child, and every ancestor then centred it (#1470, #1481).
    PaperBackground()
      .overlay(alignment: .topLeading) {
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
      // Blank inline title: a fixed small bar on every screen, native back
      // chevron and edge-swipe intact on a pushed one, nothing to look at (#1822).
      .navigationTitle("")
      .navigationBarTitleDisplayMode(.inline)
  }

  private var titleText: some View {
    Text(title)
      .font(IntradaFont.pageTitle())
      .foregroundStyle(IntradaColor.ink)
  }

  private var header: some View {
    HStack(alignment: .firstTextBaseline) {
      // A child view, even an empty one, still claims the HStack's default
      // inter-item spacing, so a screen with no leading action must omit
      // the subview entirely rather than render an empty one (#1724).
      if Leading.self != EmptyView.self {
        leadingContent
          .alignmentGuide(.firstTextBaseline) { $0[VerticalAlignment.center] }
      }
      VStack(alignment: .leading, spacing: 3) {
        // A swipe behind a wrapped title would land under its last line only.
        ViewThatFits(in: .horizontal) {
          titleText.lineLimit(1).markerSwipe()
          titleText
        }
        if let subtitle {
          Text(subtitle)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      // Combine only the title block so the trailing action stays its own
      // VoiceOver element rather than being merged into the heading.
      .accessibilityElement(children: .combine)
      .accessibilityLabel(
        subtitle.map { "\(title), \($0.replacingOccurrences(of: " · ", with: ", "))" } ?? title)
      Spacer(minLength: 12)
      if let trailing {
        Button(action: trailing.action) {
          ScreenScaffoldIconButton.icon(trailing.systemImage)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(trailing.label)
        // Centre the circular button on the title's baseline rather than
        // letting it hang below it.
        .alignmentGuide(.firstTextBaseline) { $0[VerticalAlignment.center] }
      }
      trailingContent
        .alignmentGuide(.firstTextBaseline) { $0[VerticalAlignment.center] }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.controlGap)
  }
}

/// The solid circular icon button `ScreenScaffold`'s `trailing` renders.
enum ScreenScaffoldIconButton {
  static func icon(_ systemImage: String) -> some View {
    Image(systemName: systemImage)
      .font(.system(size: 16, weight: .semibold))
      .foregroundStyle(IntradaColor.onAccent)
      .frame(width: 30, height: 30)
      // Reads as a solid dark button in the mock, not one of the accent's
      // allowed jobs (#1723).
      .background(IntradaColor.ink, in: Circle())
      .frame(width: 44, height: 44)
      .contentShape(Circle())
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
        message: "Pieces and exercises will live here.")
    }
  }

  #Preview("With a leading action") {
    ScreenScaffold(
      title: "Build session",
      subtitle: "3 items",
      leadingContent: {
        Button("Cancel") {}
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
      },
      trailing: .init(label: "Add", action: {}),
      content: {
        PlaceholderContent(
          systemImage: "music.note",
          message: "Detail content goes here.")
      }
    )
  }
#endif
