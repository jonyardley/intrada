import SwiftUI

/// Pill badge showing item type (Piece or Exercise).
struct TypeBadge: View {
    let itemType: String

    private var isPiece: Bool { itemType.lowercased() == "piece" }

    var body: some View {
        Text(itemType.capitalized)
            .font(.caption2)
            .fontWeight(.semibold)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(isPiece ? Color.indigo.opacity(0.2) : Color.teal.opacity(0.2))
            .foregroundStyle(isPiece ? .indigo : .teal)
            .clipShape(Capsule())
    }
}
