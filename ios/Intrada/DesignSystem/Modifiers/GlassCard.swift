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
            .shadow(color: .black.opacity(0.15), radius: 8, y: 4)
    }
}

/// Active card variant — accent border with glow.
/// Matches the web's `glass-card-active` utility.
struct GlassCardActiveModifier: ViewModifier {

    var padding: CGFloat = Spacing.card

    func body(content: Content) -> some View {
        content
            .padding(padding)
            .background(.ultraThinMaterial)
            .clipShape(RoundedRectangle(cornerRadius: DesignRadius.card))
            .overlay(
                RoundedRectangle(cornerRadius: DesignRadius.card)
                    .stroke(Color.accentFocus, lineWidth: 2)
            )
            .shadow(color: Color.accentFocus.opacity(0.20), radius: 12, y: 0)
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

    /// Apply active glassmorphism card styling (accent border + glow).
    ///
    /// - Parameter padding: Internal padding. Defaults to `Spacing.card` (16pt).
    func glassCardActive(padding: CGFloat = Spacing.card) -> some View {
        modifier(GlassCardActiveModifier(padding: padding))
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

        VStack(alignment: .leading, spacing: 8) {
            Text("Active Glass Card")
                .font(.headline)
                .foregroundStyle(Color.textPrimary)
            Text("Accent border with glow — used for currently practicing item.")
                .font(.subheadline)
                .foregroundStyle(Color.textSecondary)
        }
        .glassCardActive()
    }
    .padding()
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}
