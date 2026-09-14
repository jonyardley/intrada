import SwiftUI

/// The shared shell every top-level screen is built from. The page title lives
/// in the content, not a UIKit nav bar (the locked *Library — Light* header).
/// A pushed or presented screen hides the native nav bar entirely (#1724) and
/// carries its own back/cancel action as `leadingContent`, so the title sits
/// at the same height everywhere rather than dropping below a second bar.
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

/// The solid circular icon button `ScreenScaffold`'s `trailing` renders,
/// shared with `ScreenBackButton` so a header's leading and trailing actions
/// read as the same control (#1724).
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

/// A pushed or presented screen's back action, styled as `ScreenScaffold`'s
/// `trailing` button so leading and trailing read as one language (#1724).
/// Dismisses itself via `\.dismiss`: only fits a screen actually reached by a
/// push or a `navigationDestination`.
struct ScreenBackButton: View {
  @Environment(\.dismiss) private var dismiss

  var body: some View {
    Button(action: { dismiss() }) {
      ScreenScaffoldIconButton.icon("chevron.left")
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Back")
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

  #Preview("With a back button") {
    ScreenScaffold(
      title: "Clair de Lune",
      subtitle: "Claude Debussy",
      leadingContent: { ScreenBackButton() },
      trailing: .init(label: "Add", action: {}),
      content: {
        PlaceholderContent(
          systemImage: "music.note",
          message: "Detail content goes here.")
      }
    )
  }
#endif
