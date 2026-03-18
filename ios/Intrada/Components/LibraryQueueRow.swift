import SwiftUI

/// A tappable row in the session builder library list.
///
/// Shows a library item with toggle state:
/// - **Unselected**: + icon on the right
/// - **Selected**: Accent left bar + check-circle icon on the right
///
/// Tap toggles the item in/out of the setlist.
struct LibraryQueueRow: View {
    let item: LibraryItemView
    let isSelected: Bool
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 12) {
                // Accent bar (selected indicator)
                RoundedRectangle(cornerRadius: 2)
                    .fill(isSelected ? Color.accent : Color.clear)
                    .frame(width: 3, height: 40)

                // Item info
                VStack(alignment: .leading, spacing: 2) {
                    Text(item.title)
                        .font(.body)
                        .fontWeight(isSelected ? .semibold : .medium)
                        .foregroundStyle(Color.textPrimary)
                        .lineLimit(1)

                    if !item.subtitle.isEmpty {
                        Text(item.subtitle)
                            .font(.caption)
                            .foregroundStyle(Color.textMuted)
                            .lineLimit(1)
                    }
                }

                Spacer()

                // Type badge
                TypeBadge(kind: item.itemType)

                // Toggle icon
                Image(systemName: isSelected ? "checkmark.circle.fill" : "plus")
                    .font(.body)
                    .foregroundStyle(isSelected ? Color.accentText : Color.textMuted)
                    .frame(width: 20, height: 20)
            }
            .padding(.vertical, 12)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel("\(item.title), \(isSelected ? "selected" : "not selected")")
        .accessibilityHint(isSelected ? "Double tap to remove from setlist" : "Double tap to add to setlist")
    }
}

#Preview {
    VStack(spacing: 0) {
        Text("Selected")
            .font(.caption)
            .foregroundStyle(Color.textMuted)
            .padding(.top, 16)

        LibraryQueueRow(
            item: LibraryItemView(
                id: "1", itemType: .piece, title: "Clair de Lune",
                subtitle: "Debussy", key: "D♭", tempo: "72 BPM",
                notes: nil, tags: [], createdAt: "", updatedAt: "",
                practice: nil, latestAchievedTempo: nil
            ),
            isSelected: true,
            onTap: {}
        )
        .padding(.horizontal, 16)

        Divider().background(Color.borderDefault)

        Text("Unselected")
            .font(.caption)
            .foregroundStyle(Color.textMuted)
            .padding(.top, 8)

        LibraryQueueRow(
            item: LibraryItemView(
                id: "2", itemType: .piece, title: "Moonlight Sonata",
                subtitle: "Beethoven", key: "C# minor", tempo: nil,
                notes: nil, tags: [], createdAt: "", updatedAt: "",
                practice: nil, latestAchievedTempo: nil
            ),
            isSelected: false,
            onTap: {}
        )
        .padding(.horizontal, 16)
    }
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
