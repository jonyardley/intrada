import SwiftUI

// MARK: - ItemKind Display Extensions

extension ItemKind {

    /// Human-readable display text for the badge.
    var displayText: String {
        switch self {
        case .piece: "Piece"
        case .exercise: "Exercise"
        }
    }

    /// Badge background colour for this item type.
    var badgeBackground: Color {
        switch self {
        case .piece: Color.badgePieceBg
        case .exercise: Color.badgeExerciseBg
        }
    }

    /// Badge text colour for this item type.
    var badgeTextColor: Color {
        switch self {
        case .piece: Color.badgePieceText
        case .exercise: Color.badgeExerciseText
        }
    }
}

/// Piece/Exercise type pill matching the web's `TypeBadge` component.
///
///     TypeBadge(kind: .piece)
///     TypeBadge(kind: .exercise)
struct TypeBadge: View {

    let kind: ItemKind

    var body: some View {
        Text(kind.displayText)
            .font(.system(size: 14, weight: .medium))
            .foregroundStyle(kind.badgeTextColor)
            .padding(.horizontal, 12)
            .padding(.vertical, 4)
            .background(kind.badgeBackground)
            .clipShape(RoundedRectangle(cornerRadius: DesignRadius.badge))
            .accessibilityLabel("\(kind.displayText) type")
    }
}

#Preview("TypeBadge") {
    HStack(spacing: 12) {
        TypeBadge(kind: .piece)
        TypeBadge(kind: .exercise)
    }
    .padding()
    .background(Color.backgroundApp)
}
