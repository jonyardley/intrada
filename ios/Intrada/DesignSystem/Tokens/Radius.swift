import SwiftUI

// MARK: - Design Tokens: Corner Radius
//
// Maps to the web's radius custom properties.
// iOS values are intentionally scaled up from the web (Tailwind v4 defaults:
// xl=12px, lg=8px, md=6px) to feel native on larger touch targets and
// match Apple HIG corner radius conventions.

enum DesignRadius {
    /// Card corners — 16pt (web: var(--radius-xl) = 12px, scaled +4 for iOS)
    static let card: CGFloat = 16

    /// Button corners — 12pt (web: var(--radius-lg) = 8px, scaled +4 for iOS)
    static let button: CGFloat = 12

    /// Input field corners — 12pt (web: var(--radius-lg) = 8px, scaled +4 for iOS)
    static let input: CGFloat = 12

    /// Badge corners — 8pt (web: var(--radius-md) = 6px, scaled +2 for iOS)
    static let badge: CGFloat = 8

    /// Fully rounded pill — for TypeTabs, tags (web: 9999px)
    static let pill: CGFloat = 9999
}

// MARK: - Preview

#Preview("Radius") {
    HStack(spacing: 16) {
        radiusSample("Card", DesignRadius.card)
        radiusSample("Button", DesignRadius.button)
        radiusSample("Input", DesignRadius.input)
        radiusSample("Badge", DesignRadius.badge)
    }
    .padding()
    .background(Color.backgroundApp)
}

private func radiusSample(_ label: String, _ radius: CGFloat) -> some View {
    VStack(spacing: 4) {
        RoundedRectangle(cornerRadius: radius)
            .fill(Color.surfacePrimary)
            .frame(width: 64, height: 44)
            .overlay(
                RoundedRectangle(cornerRadius: radius)
                    .stroke(Color.borderCard, lineWidth: 1)
            )
        Text(label)
            .font(.system(size: 10))
            .foregroundStyle(Color.textMuted)
        Text("\(Int(radius))pt")
            .font(.system(size: 9))
            .foregroundStyle(Color.textFaint)
    }
}
