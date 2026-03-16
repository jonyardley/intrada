import SwiftUI

/// A flow layout that wraps children to the next line when they exceed available width.
/// Used for tag chips and other variable-width content that needs to wrap.
struct FlowLayout: Layout {
    var spacing: CGFloat = 6

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let result = computeLayout(proposal: proposal, subviews: subviews)
        return result.size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let result = computeLayout(proposal: proposal, subviews: subviews)
        for (index, position) in result.positions.enumerated() {
            subviews[index].place(
                at: CGPoint(x: bounds.minX + position.x, y: bounds.minY + position.y),
                proposal: .unspecified
            )
        }
    }

    private struct LayoutResult {
        var size: CGSize
        var positions: [CGPoint]
    }

    private func computeLayout(proposal: ProposedViewSize, subviews: Subviews) -> LayoutResult {
        let maxWidth = proposal.width ?? .infinity
        var positions: [CGPoint] = []
        var currentX: CGFloat = 0
        var currentY: CGFloat = 0
        var lineHeight: CGFloat = 0
        var totalHeight: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if currentX + size.width > maxWidth && currentX > 0 {
                currentX = 0
                currentY += lineHeight + spacing
                lineHeight = 0
            }
            positions.append(CGPoint(x: currentX, y: currentY))
            lineHeight = max(lineHeight, size.height)
            currentX += size.width + spacing
            totalHeight = currentY + lineHeight
        }

        return LayoutResult(
            size: CGSize(width: maxWidth, height: totalHeight),
            positions: positions
        )
    }
}

#Preview {
    FlowLayout(spacing: 8) {
        ForEach(["romantic", "exam", "grade 8", "recital", "scales", "warm-up"], id: \.self) { tag in
            Text(tag)
                .font(.caption)
                .foregroundStyle(Color.textSecondary)
                .padding(.horizontal, 10)
                .padding(.vertical, 4)
                .background(Color.surfaceSecondary)
                .clipShape(Capsule())
        }
    }
    .padding()
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
