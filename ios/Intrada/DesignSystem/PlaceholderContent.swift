import SwiftUI

struct PlaceholderAction {
  let title: String
  let identifier: String
  let action: @MainActor () -> Void
}

/// A centred "nothing here yet" body: tinted glyph, one line of muted copy, and
/// a marker button for each way to fill the screen.
struct PlaceholderContent: View {
  let systemImage: String
  let message: String
  var glyphTint: Color = IntradaColor.accent.opacity(IntradaOpacity.dimmed)
  var actions: [PlaceholderAction] = []

  // Scrolls only when it cannot fit: at large text sizes the buttons would
  // otherwise squeeze the message and their own labels into truncation.
  var body: some View {
    GeometryReader { proxy in
      ScrollView {
        stack.frame(minHeight: proxy.size.height)
      }
      .scrollBounceBehavior(.basedOnSize)
    }
  }

  private var stack: some View {
    VStack(spacing: IntradaSpacing.section) {
      VStack(spacing: IntradaSpacing.cardCompact) {
        Image(systemName: systemImage)
          .iconSize(.hero)
          .foregroundStyle(glyphTint)
        Text(message)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
      }
      .accessibilityElement(children: .combine)
      .accessibilityLabel(message)
      if !actions.isEmpty {
        VStack(spacing: IntradaSpacing.cardCompact) {
          ForEach(actions, id: \.identifier) { action in
            MarkerButton(action.title, action: action.action)
              .accessibilityIdentifier(action.identifier)
          }
        }
      }
    }
    .padding(32)
    .frame(maxWidth: .infinity)
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

  #Preview("With actions") {
    ZStack {
      PaperBackground()
      PlaceholderContent(
        systemImage: "books.vertical",
        message: "Pieces and exercises will live here.",
        actions: [
          .init(title: "Add piece or exercise", identifier: "preview.add") {}
        ])
    }
  }
#endif
