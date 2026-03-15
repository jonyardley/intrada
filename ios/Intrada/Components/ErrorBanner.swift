import SwiftUI

/// Persistent error display matching the web's `ErrorBanner` component.
///
///     ErrorBanner(message: "Failed to load items") {
///         // dismiss action
///     }
struct ErrorBanner: View {

    let message: String
    var onDismiss: (() -> Void)? = nil

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Text("Error: \(message)")
                .font(.system(size: 14))
                .foregroundStyle(Color.dangerText)
                .frame(maxWidth: .infinity, alignment: .leading)

            if let onDismiss {
                Button(action: onDismiss) {
                    Text("Dismiss")
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(Color.dangerText)
                }
            }
        }
        .padding(Spacing.card)
        .background(Color.dangerSurface)
        .clipShape(RoundedRectangle(cornerRadius: DesignRadius.badge))
        .overlay(
            RoundedRectangle(cornerRadius: DesignRadius.badge)
                .stroke(Color.dangerText.opacity(0.20), lineWidth: 1)
        )
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Error: \(message)")
        .accessibilityAddTraits(.isStaticText)
    }
}

#Preview("ErrorBanner") {
    VStack(spacing: 16) {
        ErrorBanner(message: "Failed to load items from the server") {
            // dismiss
        }

        ErrorBanner(message: "Network connection lost")
    }
    .padding()
    .background(Color.backgroundApp)
}
