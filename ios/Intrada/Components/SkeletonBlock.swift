import SwiftUI

/// Pulsing rectangular loading placeholder for cards.
/// Matches the web's `SkeletonBlock` component.
///
///     SkeletonBlock()
///     SkeletonBlock(height: 120)
struct SkeletonBlock: View {

    var height: CGFloat = 96

    var body: some View {
        RoundedRectangle(cornerRadius: DesignRadius.card)
            .fill(Color.surfaceSecondary)
            .frame(height: height)
            .pulsing()
            .accessibilityHidden(true)
    }
}

#Preview("SkeletonBlock") {
    VStack(spacing: 12) {
        SkeletonBlock()
        SkeletonBlock(height: 120)
        SkeletonBlock(height: 60)
    }
    .padding()
    .background(Color.backgroundApp)
}
