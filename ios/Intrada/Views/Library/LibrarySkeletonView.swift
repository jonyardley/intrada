import SwiftUI

/// Loading skeleton for the library list, showing placeholder rows
/// that match the layout of LibraryItemRow.
struct LibrarySkeletonView: View {
    var body: some View {
        VStack(spacing: 0) {
            ForEach(0..<5, id: \.self) { _ in
                skeletonRow
                Divider()
                    .overlay(Color.borderDefault)
            }
        }
    }

    private var skeletonRow: some View {
        VStack(alignment: .leading, spacing: 6) {
            // Title + badge row
            HStack {
                SkeletonLine(width: 180, height: 18)
                Spacer()
                SkeletonLine(width: 60, height: 22)
            }

            // Composer
            SkeletonLine(width: 120, height: 14)

            // Metadata
            HStack(spacing: 12) {
                SkeletonLine(width: 70, height: 12)
                SkeletonLine(width: 90, height: 12)
            }

            // Tags
            HStack(spacing: 6) {
                SkeletonLine(width: 50, height: 20)
                SkeletonLine(width: 40, height: 20)
            }
        }
        .padding(.vertical, 8)
        .padding(.horizontal, Spacing.card)
    }
}

/// Loading skeleton for the item detail view.
struct DetailSkeletonView: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            // Title
            SkeletonLine(width: 220, height: 28)
            // Subtitle
            SkeletonLine(width: 160, height: 18)
            // Badge
            SkeletonLine(width: 60, height: 24)

            // Metadata card
            SkeletonBlock(height: 80)

            // Tags
            HStack(spacing: 8) {
                SkeletonLine(width: 60, height: 24)
                SkeletonLine(width: 50, height: 24)
                SkeletonLine(width: 70, height: 24)
            }

            // Notes card
            SkeletonBlock(height: 100)

            // Practice summary
            SkeletonBlock(height: 80)
        }
        .padding(Spacing.card)
    }
}

#Preview {
    ScrollView {
        LibrarySkeletonView()
    }
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
