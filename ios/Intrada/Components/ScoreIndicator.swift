import SwiftUI

/// Displays a 1–5 score as filled/empty circles.
struct ScoreIndicator: View {
    let score: UInt8
    var maxScore: UInt8 = 5

    var body: some View {
        HStack(spacing: 3) {
            ForEach(1...maxScore, id: \.self) { i in
                Circle()
                    .fill(i <= score ? Color.indigo : Color.secondary.opacity(0.2))
                    .frame(width: 8, height: 8)
            }
        }
    }
}
