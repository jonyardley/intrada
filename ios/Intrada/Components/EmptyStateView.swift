import SwiftUI

/// Reusable empty state with icon, message, and optional CTA.
/// Uses design tokens — no raw SwiftUI colours.
struct EmptyStateView<AdditionalContent: View>: View {
    let icon: String
    let title: String
    var message: String? = nil
    var actionTitle: String? = nil
    var action: (() -> Void)? = nil
    var additionalContent: AdditionalContent

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

            additionalContent

            if let actionTitle, let action {
                ButtonView(actionTitle, variant: .primary, action: action)
                    .frame(maxWidth: 200)
            }
        }
        .padding(Spacing.cardComfortable)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

extension EmptyStateView {
    init(
        icon: String,
        title: String,
        message: String? = nil,
        actionTitle: String? = nil,
        action: (() -> Void)? = nil,
        @ViewBuilder additionalContent: () -> AdditionalContent
    ) {
        self.icon = icon
        self.title = title
        self.message = message
        self.actionTitle = actionTitle
        self.action = action
        self.additionalContent = additionalContent()
    }
}

extension EmptyStateView where AdditionalContent == EmptyView {
    init(
        icon: String,
        title: String,
        message: String? = nil,
        actionTitle: String? = nil,
        action: (() -> Void)? = nil
    ) {
        self.icon = icon
        self.title = title
        self.message = message
        self.actionTitle = actionTitle
        self.action = action
        self.additionalContent = EmptyView()
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
            title: "No sessions",
            message: "Start practising to see your sessions here."
        )
    }
    .background(Color.backgroundApp)
}
