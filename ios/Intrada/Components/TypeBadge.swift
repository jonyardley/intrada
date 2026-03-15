import SwiftUI

/// Piece/Exercise type pill matching the web's `TypeBadge` component.
///
///     TypeBadge(itemType: "piece")
///     TypeBadge(itemType: "exercise")
struct TypeBadge: View {

    let itemType: String

    private var backgroundColor: Color {
        switch itemType.lowercased() {
        case "piece": Color.badgePieceBg
        case "exercise": Color.badgeExerciseBg
        default: Color.surfacePrimary
        }
    }

    private var textColor: Color {
        switch itemType.lowercased() {
        case "piece": Color.badgePieceText
        case "exercise": Color.badgeExerciseText
        default: Color.textSecondary
        }
    }

    private var displayText: String {
        switch itemType.lowercased() {
        case "piece": "Piece"
        case "exercise": "Exercise"
        default: itemType
        }
    }

    var body: some View {
        Text(displayText)
            .font(.system(size: 14, weight: .medium))
            .foregroundStyle(textColor)
            .padding(.horizontal, 12)
            .padding(.vertical, 4)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: DesignRadius.badge))
    }
}

#Preview("TypeBadge") {
    HStack(spacing: 12) {
        TypeBadge(itemType: "piece")
        TypeBadge(itemType: "exercise")
        TypeBadge(itemType: "unknown")
    }
    .padding()
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}
