import SwiftUI

// MARK: - Design Tokens: Spacing
//
// All values match the web's spacing custom properties.
// Based on a 4pt grid.

enum Spacing {
    /// Stat cards, compact elements — 12pt
    static let cardCompact: CGFloat = 12

    /// Standard card padding — 16pt
    static let card: CGFloat = 16

    /// Comfortable card padding — 24pt
    static let cardComfortable: CGFloat = 24

    /// Between page sections — 48pt
    static let section: CGFloat = 48

    /// Major section breaks — 64pt
    static let sectionLarge: CGFloat = 64
}

// MARK: - Preview

#Preview("Spacing") {
    VStack(alignment: .leading, spacing: 16) {
        spacingRow("Card Compact", Spacing.cardCompact)
        spacingRow("Card", Spacing.card)
        spacingRow("Card Comfortable", Spacing.cardComfortable)
        spacingRow("Section", Spacing.section)
        spacingRow("Section Large", Spacing.sectionLarge)
    }
    .padding()
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}

private func spacingRow(_ label: String, _ value: CGFloat) -> some View {
    HStack {
        Text(label)
            .font(.caption)
            .foregroundStyle(Color.textSecondary)
            .frame(width: 120, alignment: .leading)

        Rectangle()
            .fill(Color.accent.opacity(0.5))
            .frame(width: value, height: 20)

        Text("\(Int(value))pt")
            .font(.caption)
            .foregroundStyle(Color.textMuted)
    }
}
