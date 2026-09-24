import SwiftUI

/// The shared shell every top-level screen is built from. The page title lives
/// in the content, not the native nav bar's title (the locked *Library, Light*
/// header). Every screen forces `.inline` display with an empty native
/// title (#1724, #1822), so the bar is the same fixed height wherever it shows
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
  let trailingContent: Trailing
  let trailingPlacement: TrailingPlacement
  private var reservesSubtitleLine = false
  @ViewBuilder var content: Content
  @Environment(\.navigationBarHiddenAtRoot) private var hiddenAtRoot
  @Environment(\.isPresented) private var isPresented

  struct TrailingAction {
    let label: String
    var systemImage: String = "plus"
    var identifier: String = ""
    let action: () -> Void
  }

  enum TrailingPlacement {
    case toolbar
    case header
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
    self.trailingPlacement = .toolbar
    self.content = content()
  }

  init(
    title: String,
    subtitle: String? = nil,
    trailingPlacement: TrailingPlacement = .toolbar,
    @ViewBuilder trailingContent: () -> Trailing,
    @ViewBuilder content: () -> Content
  ) where Leading == EmptyView {
    self.title = title
    self.subtitle = subtitle
    self.trailing = nil
    self.leadingContent = EmptyView()
    self.trailingContent = trailingContent()
    self.trailingPlacement = trailingPlacement
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
    self.trailingPlacement = .toolbar
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
    self.trailingPlacement = .toolbar
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
      .toolbar {
        if Leading.self != EmptyView.self {
          ToolbarItem(placement: .topBarLeading) { leadingContent }
        }
        if Trailing.self != EmptyView.self, trailingPlacement == .toolbar {
          ToolbarItemGroup(placement: .topBarTrailing) { trailingContent }
        }
      }
      // Forced visible so an item-less bar keeps the iPad split's columns matched
      // (#1868, #1682); a tab's first screen drops its empty band instead (#1912).
      .toolbar(hidesBar ? .hidden : .visible, for: .navigationBar)
  }

  // Unproven before iOS 26.5: an opened screen or sheet may inherit the flag (#1912).
  private var hidesBar: Bool { hiddenAtRoot && !isPresented }

  private var titleText: some View {
    Text(title)
      .font(IntradaFont.pageTitle())
      .foregroundStyle(IntradaColor.ink)
  }

  private var header: some View {
    HStack(alignment: .firstTextBaseline) {
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
        } else if reservesSubtitleLine {
          Text(" ").font(IntradaFont.meta).hidden()
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
        .accessibilityIdentifier(trailing.identifier)
        // Centre the circular button on the title's baseline rather than
        // letting it hang below it.
        .alignmentGuide(.firstTextBaseline) { $0[VerticalAlignment.center] }
      }
      if Trailing.self != EmptyView.self, trailingPlacement == .header {
        trailingContent
          .alignmentGuide(.firstTextBaseline) { $0[VerticalAlignment.center] }
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.controlGap)
  }

  // Keeps the header rule level with an iPad split column that has a subtitle.
  func reservingSubtitleLine(_ reserves: Bool) -> Self {
    var copy = self
    copy.reservesSubtitleLine = reserves
    return copy
  }
}

private struct NavigationBarHiddenAtRootKey: EnvironmentKey {
  static let defaultValue = false
}

extension EnvironmentValues {
  var navigationBarHiddenAtRoot: Bool {
    get { self[NavigationBarHiddenAtRootKey.self] }
    set { self[NavigationBarHiddenAtRootKey.self] = newValue }
  }
}

extension View {
  /// Hides the bar on the first screen of a tab's stack.
  func navigationBarHiddenAtRoot() -> some View {
    environment(\.navigationBarHiddenAtRoot, true)
  }
}

/// The solid circular icon button `ScreenScaffold`'s `trailing` renders.
@MainActor
enum ScreenScaffoldIconButton {
  static func icon(_ systemImage: String) -> some View {
    Image(systemName: systemImage)
      .iconSize(.inline, weight: .semibold)
      .foregroundStyle(IntradaColor.onAccent)
      .frame(width: 30, height: 30)
      .background(IntradaColor.accent, in: Circle())
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
    NavigationStack {
      ScreenScaffold(
        title: "Build session",
        subtitle: "3 items",
        leadingContent: {
          Button("Cancel") {}
        },
        trailing: .init(label: "Add", action: {}),
        content: {
          PlaceholderContent(
            systemImage: "music.note",
            message: "Detail content goes here.")
        }
      )
    }
  }
#endif
