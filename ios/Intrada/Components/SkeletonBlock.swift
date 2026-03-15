import SwiftUI

/// Pulsing rectangular loading placeholder for cards.
/// Matches the web's `SkeletonBlock` component.
///
///     SkeletonBlock()
///     SkeletonBlock(height: 120)
struct SkeletonBlock: View {

    var height: CGFloat = 96

    @State private var isAnimating = false

    var body: some View {
        RoundedRectangle(cornerRadius: DesignRadius.card)
            .fill(Color.surfaceSecondary)
            .frame(height: height)
            .opacity(isAnimating ? 0.3 : 1.0)
            .animation(
                .easeInOut(duration: 1.0).repeatForever(autoreverses: true),
                value: isAnimating
            )
            .onAppear { isAnimating = true }
    }
}

#Preview("SkeletonBlock") {
    VStack(spacing: 12) {
        SkeletonBlock()
        SkeletonBlock(height: 120)
        SkeletonBlock(height: 60)
    }
    .padding()
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}
