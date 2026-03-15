import SwiftUI

/// Pulsing text-width loading placeholder.
/// Matches the web's `SkeletonLine` component.
///
///     SkeletonLine()
///     SkeletonLine(width: 120, height: 12)
struct SkeletonLine: View {

    var width: CGFloat? = nil
    var height: CGFloat = 16

    @State private var isAnimating = false

    var body: some View {
        RoundedRectangle(cornerRadius: 4)
            .fill(Color.surfaceSecondary)
            .frame(width: width, height: height)
            .frame(maxWidth: width == nil ? .infinity : nil, alignment: .leading)
            .opacity(isAnimating ? 0.3 : 1.0)
            .animation(
                .easeInOut(duration: 1.0).repeatForever(autoreverses: true),
                value: isAnimating
            )
            .onAppear { isAnimating = true }
    }
}

#Preview("SkeletonLine") {
    VStack(alignment: .leading, spacing: 12) {
        SkeletonLine()
        SkeletonLine(width: 200)
        SkeletonLine(width: 140, height: 12)
        SkeletonLine(width: 80, height: 10)
    }
    .padding()
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}
