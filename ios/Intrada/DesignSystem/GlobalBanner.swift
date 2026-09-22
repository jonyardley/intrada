import SwiftUI

/// Full-width banner below the status bar for app-level conditions not tied to a
/// sheet. Dismissible when `onDismiss` is set; otherwise a persistent state.
struct GlobalBanner: View {
  /// `.danger` is a refusal or a failed write; `.notice` is a true outcome that
  /// is not a failure (#1325).
  enum Tone {
    case danger
    case notice
  }

  let message: String
  var tone: Tone = .danger
  var onDismiss: (() -> Void)?

  var body: some View {
    // `.combine` flattens the dismiss button away, so re-expose it as an
    // accessibility action — otherwise VoiceOver can't dismiss the banner.
    if let onDismiss {
      bar.accessibilityElement(children: .combine)
        .accessibilityLabel(accessibilityText)
        .accessibilityAction(named: "Dismiss", onDismiss)
    } else {
      bar.accessibilityElement(children: .combine)
        .accessibilityLabel(accessibilityText)
    }
  }

  private var bar: some View {
    HStack(alignment: .top, spacing: IntradaSpacing.controlGap) {
      Image(systemName: glyph)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(ink)
      Text(message)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(ink)
        .frame(maxWidth: .infinity, alignment: .leading)
      if let onDismiss {
        Button(action: onDismiss) {
          Image(systemName: "xmark")
            .font(IntradaFont.metaMedium)
            .foregroundStyle(ink)
        }
        .buttonStyle(.plain)
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.vertical, 10)
    .frame(maxWidth: .infinity)
    .background(fill)
    .overlay(alignment: .bottom) {
      if tone == .notice {
        HairlineDivider()
      }
    }
  }

  private var accessibilityText: String {
    switch tone {
    case .danger: message
    case .notice: "Note, " + message
    }
  }

  private var glyph: String {
    switch tone {
    case .danger: "exclamationmark.triangle.fill"
    case .notice: "info.circle"
    }
  }

  private var ink: Color {
    switch tone {
    case .danger: IntradaColor.danger
    case .notice: IntradaColor.inkSecondary
    }
  }

  private var fill: Color {
    switch tone {
    case .danger: IntradaColor.dangerBanner
    case .notice: IntradaColor.cardFill
    }
  }
}

#if DEBUG
  #Preview {
    VStack(spacing: 0) {
      GlobalBanner(message: "Couldn't delete that item.", onDismiss: {})
      GlobalBanner(message: "Storage unavailable · changes this session won't be saved.")
      GlobalBanner(
        message: "That metronome setting doesn't give a crotchet tempo, so this play has none.",
        tone: .notice, onDismiss: {})
      Spacer()
    }
    .background(IntradaColor.paperTop)
  }
#endif
