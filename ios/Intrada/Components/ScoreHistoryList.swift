import SwiftUI

/// Displays a list of practice score history entries with dates and score badges.
struct ScoreHistoryList: View {
    let entries: [ScoreHistoryEntry]

    var body: some View {
        if !entries.isEmpty {
            CardView {
                VStack(alignment: .leading, spacing: 8) {
                    Text("SCORE HISTORY")
                        .fieldLabelStyle()

                    ForEach(entries, id: \.sessionId) { entry in
                        HStack {
                            Text(formatDate(entry.sessionDate))
                                .font(.caption)
                                .foregroundStyle(Color.textFaint)
                            Spacer()
                            scoreBadge(score: entry.score)
                        }
                    }
                }
            }
        }
    }

    private func scoreBadge(score: UInt8) -> some View {
        HStack(spacing: 2) {
            Image(systemName: "star.fill")
                .font(.caption2)
            Text("\(score)")
                .font(.caption)
                .fontWeight(.medium)
        }
        .foregroundStyle(scoreColor(for: score))
        .padding(.horizontal, 8)
        .padding(.vertical, 3)
        .background(scoreColor(for: score).opacity(0.15))
        .clipShape(Capsule())
    }

    private func scoreColor(for score: UInt8) -> Color {
        switch score {
        case 5: .success
        case 4: .accentText
        case 3: .warningText
        case 2: .dangerText
        default: .textFaint
        }
    }
}

#Preview {
    VStack {
        Text("ScoreHistoryList Preview")
            .foregroundStyle(Color.textMuted)
    }
    .padding()
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
