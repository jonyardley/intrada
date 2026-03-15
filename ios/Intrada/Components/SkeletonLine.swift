import SwiftUI

/// Pulsing text-width loading placeholder.
/// Matches the web's `SkeletonLine` component.
///
///     SkeletonLine()
///     SkeletonLine(width: 120, height: 12)
struct SkeletonLine: View {

    var width: CGFloat? = nil
    var height: CGFloat = 16

    var body: some View {
        RoundedRectangle(cornerRadius: 4)
            .fill(Color.surfaceSecondary)
            .frame(width: width, height: height)
            .frame(maxWidth: width == nil ? .infinity : nil, alignment: .leading)
            .pulsing()
            .accessibilityHidden(true)
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
    .background(Color.backgroundApp)
}
