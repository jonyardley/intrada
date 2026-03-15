import SwiftUI

// MARK: - Design Tokens: Corner Radius
//
// Matches the web's radius custom properties.

enum DesignRadius {
    /// Card corners — 16pt (web: var(--radius-xl))
    static let card: CGFloat = 16

    /// Button corners — 12pt (web: var(--radius-lg))
    static let button: CGFloat = 12

    /// Input field corners — 12pt (web: var(--radius-lg))
    static let input: CGFloat = 12

    /// Badge/pill corners — 8pt (web: var(--radius-md))
    static let badge: CGFloat = 8
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
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
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
