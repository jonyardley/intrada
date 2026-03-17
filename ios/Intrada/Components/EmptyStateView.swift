import SwiftUI

/// Reusable empty state with icon, message, and optional CTA.
/// Uses design tokens — no raw SwiftUI colours.
struct EmptyStateView: View {
    let icon: String
    let title: String
    var message: String? = nil
    var actionTitle: String? = nil
    var action: (() -> Void)? = nil

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: icon)
                .font(.system(size: 40))
                .foregroundStyle(Color.textFaint)

            Text(title)
                .font(.headline)
                .foregroundStyle(Color.textSecondary)

            if let message {
                Text(message)
                    .font(.subheadline)
                    .foregroundStyle(Color.textMuted)
                    .multilineTextAlignment(.center)
            }

            if let actionTitle, let action {
                ButtonView(actionTitle, variant: .primary, action: action)
                    .frame(maxWidth: 200)
            }
        }
        .padding(Spacing.cardComfortable)
        .frame(maxWidth: .infinity)
    }
}

#Preview("EmptyStateView") {
    VStack(spacing: 24) {
        EmptyStateView(
            icon: "books.vertical",
            title: "No items yet",
            message: "Add pieces and exercises to build your library.",
            actionTitle: "Add Item",
            action: { }
        )

        EmptyStateView(
            icon: "play.circle",
            title: "No practices",
            message: "Start practising to see your practices here."
        )
    }
    .background(Color.backgroundApp)
}
