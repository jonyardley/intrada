import SwiftUI

// MARK: - Glassmorphism Card Modifier
//
// Matches the web's `glass-card` utility:
//   background: ultraThinMaterial (blur) with solid fallback
//   border: 1px borderCard
//   radius: DesignRadius.card
//   shadow: subtle drop shadow

struct GlassCardModifier: ViewModifier {

    var padding: CGFloat = Spacing.card

    func body(content: Content) -> some View {
        content
            .padding(padding)
            .background(.ultraThinMaterial)
            .clipShape(RoundedRectangle(cornerRadius: DesignRadius.card))
            .overlay(
                RoundedRectangle(cornerRadius: DesignRadius.card)
                    .stroke(Color.borderCard, lineWidth: 1)
            )
            .shadow(color: .shadowSubtle, radius: 8, y: 4)
    }
}

// MARK: - View Extension

extension View {
    /// Apply glassmorphism card styling.
    ///
    /// - Parameter padding: Internal padding. Defaults to `Spacing.card` (16pt).
    func glassCard(padding: CGFloat = Spacing.card) -> some View {
        modifier(GlassCardModifier(padding: padding))
    }
}

// MARK: - Preview

#Preview("Glass Card") {
    VStack(spacing: 24) {
        VStack(alignment: .leading, spacing: 8) {
            Text("Standard Glass Card")
                .font(.headline)
                .foregroundStyle(Color.textPrimary)
            Text("This is a glassmorphism container matching the web's glass-card utility.")
                .font(.subheadline)
                .foregroundStyle(Color.textSecondary)
        }
        .glassCard()

        VStack(alignment: .leading, spacing: 8) {
            Text("Compact Glass Card")
                .font(.headline)
                .foregroundStyle(Color.textPrimary)
            Text("Uses compact padding.")
                .font(.subheadline)
                .foregroundStyle(Color.textSecondary)
        }
        .glassCard(padding: Spacing.cardCompact)
    }
    .padding()
    .background(Color.backgroundApp)
}
