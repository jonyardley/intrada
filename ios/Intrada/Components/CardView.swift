import SwiftUI

/// Glassmorphism container matching the web's `Card` component.
///
/// Wraps content in a frosted-glass card with blur, border, and shadow.
///
///     CardView {
///         Text("Hello")
///     }
///
///     CardView(padding: Spacing.cardCompact) {
///         Text("Compact")
///     }
struct CardView<Content: View>: View {

    var padding: CGFloat = Spacing.card
    @ViewBuilder var content: Content

    var body: some View {
        content
            .glassCard(padding: padding)
    }
}

#Preview("CardView") {
    VStack(spacing: 16) {
        CardView {
            VStack(alignment: .leading, spacing: 8) {
                Text("Standard Card")
                    .sectionTitleStyle()
                Text("Default padding (16pt)")
                    .font(.subheadline)
                    .foregroundStyle(Color.textSecondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }

        CardView(padding: Spacing.cardCompact) {
            Text("Compact Card")
                .font(.subheadline)
                .foregroundStyle(Color.textSecondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }

        CardView(padding: Spacing.cardComfortable) {
            Text("Comfortable Card")
                .font(.subheadline)
                .foregroundStyle(Color.textSecondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
    }
    .padding()
    .background(Color.backgroundApp)
}
