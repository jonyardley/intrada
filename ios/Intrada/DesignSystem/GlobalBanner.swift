import SwiftUI

/// Full-width banner below the status bar for app-level conditions not tied to a
/// sheet. Dismissible when `onDismiss` is set; otherwise a persistent state.
struct GlobalBanner: View {
  let message: String
  var onDismiss: (() -> Void)?

  var body: some View {
    // `.combine` flattens the dismiss button away, so re-expose it as an
    // accessibility action — otherwise VoiceOver can't dismiss the banner.
    if let onDismiss {
      bar.accessibilityElement(children: .combine)
        .accessibilityLabel(message)
        .accessibilityAction(named: "Dismiss", onDismiss)
    } else {
      bar.accessibilityElement(children: .combine)
        .accessibilityLabel(message)
    }
  }

  private var bar: some View {
    HStack(alignment: .top, spacing: 8) {
      Image(systemName: "exclamationmark.triangle.fill")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.danger)
      Text(message)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.danger)
        .frame(maxWidth: .infinity, alignment: .leading)
      if let onDismiss {
        Button(action: onDismiss) {
          Image(systemName: "xmark")
            .font(IntradaFont.metaMedium)
            .foregroundStyle(IntradaColor.danger)
        }
        .buttonStyle(.plain)
      }
    }
    .padding(.horizontal, 16)
    .padding(.vertical, 10)
    .frame(maxWidth: .infinity)
    .background(IntradaColor.danger.opacity(0.12))
  }
}

#if DEBUG
  #Preview {
    VStack(spacing: 0) {
      GlobalBanner(message: "Couldn't delete that item.", onDismiss: {})
      GlobalBanner(message: "Storage unavailable — changes this session won't be saved.")
      Spacer()
    }
    .background(IntradaColor.paperTop)
  }
#endif
