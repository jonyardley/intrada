import SwiftUI

// MARK: - Pulsing Animation Modifier
//
// Shared skeleton animation used by SkeletonLine and SkeletonBlock.
// Fades between full and 30% opacity on a 1-second loop.

struct PulsingModifier: ViewModifier {

    @State private var isAnimating = false

    func body(content: Content) -> some View {
        content
            .opacity(isAnimating ? 0.3 : 1.0)
            .animation(
                .easeInOut(duration: 1.0).repeatForever(autoreverses: true),
                value: isAnimating
            )
            .onAppear { isAnimating = true }
    }
}

extension View {
    /// Apply a pulsing skeleton animation.
    func pulsing() -> some View {
        modifier(PulsingModifier())
    }
}
