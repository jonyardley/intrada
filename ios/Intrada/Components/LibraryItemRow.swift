import SwiftUI

/// A list row displaying a library item's key information.
/// Shows title, composer, type badge, key, tempo, and tags.
struct LibraryItemRow: View {
    let item: LibraryItemView

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            // Title row with type badge
            HStack(alignment: .top) {
                Text(item.title)
                    .font(.body)
                    .fontWeight(.semibold)
                    .foregroundStyle(Color.textPrimary)
                    .lineLimit(1)

                Spacer()

                TypeBadge(kind: item.itemType)
            }

            // Composer
            if !item.subtitle.isEmpty {
                Text(item.subtitle)
                    .font(.subheadline)
                    .foregroundStyle(Color.textMuted)
                    .lineLimit(1)
            }

            // Metadata: key + tempo
            let metadataItems = buildMetadata()
            if !metadataItems.isEmpty {
                HStack(spacing: 12) {
                    ForEach(metadataItems, id: \.self) { text in
                        Text(text)
                            .font(.caption)
                            .foregroundStyle(Color.textFaint)
                    }
                }
            }

            // Tags
            if !item.tags.isEmpty {
                HStack(spacing: 6) {
                    ForEach(item.tags, id: \.self) { tag in
                        Text(tag)
                            .font(.caption2)
                            .foregroundStyle(Color.textMuted)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 3)
                            .background(Color.surfaceSecondary)
                            .clipShape(Capsule())
                            .overlay(
                                Capsule()
                                    .strokeBorder(Color.borderDefault, lineWidth: 1)
                            )
                    }
                }
            }
        }
        .padding(.vertical, 8)
        .padding(.horizontal, Spacing.card)
        .contentShape(Rectangle())
    }

    private func buildMetadata() -> [String] {
        var parts: [String] = []

        if let key: String = item.key, !key.isEmpty {
            parts.append("♯ \(key)")
        }

        if let tempo: String = item.tempo, !tempo.isEmpty {
            if let achieved: UInt16 = item.latestAchievedTempo {
                let (_, targetBpm) = parseTempoDisplay(tempo)
                if !targetBpm.isEmpty {
                    parts.append("♩ \(achieved) / \(targetBpm) BPM")
                } else {
                    parts.append("♩ \(achieved) BPM")
                }
            } else {
                parts.append("♩ \(tempo)")
            }
        }

        return parts
    }
}

#Preview {
    VStack(spacing: 0) {
        Text("LibraryItemRow Preview")
            .foregroundStyle(Color.textMuted)
            .padding()
    }
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
