import SwiftUI

struct PlaceholderAction {
  let title: String
  let identifier: String
  let action: @MainActor () -> Void
}

/// A centred "nothing here yet" body: tinted glyph, one line of muted copy, and
/// the buttons that fill the screen. The first is the marker; any after it are plain.
struct PlaceholderContent: View {
  let systemImage: String
  let message: String
  var glyphTint: Color = IntradaColor.accent.opacity(IntradaOpacity.dimmed)
  var actions: [PlaceholderAction] = []

  // Scrolls only when it cannot fit: at accessibility sizes the buttons would
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
          ForEach(Array(actions.enumerated()), id: \.element.identifier) { index, action in
            if index == 0 {
              MarkerButton(action.title, action: action.action)
                .accessibilityIdentifier(action.identifier)
            } else {
              Button(action: action.action) {
                Text(action.title)
                  .font(IntradaFont.bodyMedium)
                  .foregroundStyle(IntradaColor.accent)
                  .frame(maxWidth: .infinity, minHeight: 44)
                  .contentShape(Rectangle())
              }
              .buttonStyle(.plain)
              .accessibilityIdentifier(action.identifier)
            }
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
          .init(title: "Add piece or exercise", identifier: "preview.add") {},
          .init(title: "Scan a page", identifier: "preview.scan") {},
        ])
    }
  }
#endif
