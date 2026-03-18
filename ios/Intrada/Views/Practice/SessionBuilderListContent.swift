import SwiftUI

/// Scrollable library list for the session builder.
///
/// Shows all library items with selection state (accent bar + check icon).
/// Handles empty state and no-match-for-search state.
struct SessionBuilderListContent: View {
    let items: [LibraryItemView]
    let selectedItemIds: Set<String>
    let onToggle: (LibraryItemView) -> Void

    var body: some View {
        if items.isEmpty {
            emptyState
        } else {
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(items, id: \.id) { (item: LibraryItemView) in
                        LibraryQueueRow(
                            item: item,
                            isSelected: selectedItemIds.contains(item.id),
                            onTap: { onToggle(item) }
                        )
                        .padding(.horizontal, 16)

                        Divider()
                            .background(Color.borderDefault)
                            .padding(.leading, 16)
                    }
                }
            }
        }
    }

    private var emptyState: some View {
        VStack(spacing: 16) {
            Spacer()

            Image(systemName: "music.note.list")
                .font(.system(size: 40))
                .foregroundStyle(Color.textFaint)

            Text("No library items")
                .font(.subheadline.weight(.medium))
                .foregroundStyle(Color.textSecondary)

            Text("Add some pieces or exercises to your library first")
                .font(.caption)
                .foregroundStyle(Color.textMuted)
                .multilineTextAlignment(.center)

            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

#Preview {
    SessionBuilderListContent(
        items: [],
        selectedItemIds: [],
        onToggle: { _ in }
    )
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
