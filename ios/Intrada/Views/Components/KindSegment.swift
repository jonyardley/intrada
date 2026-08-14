import SharedTypes
import SwiftUI

struct KindSegment: View {
  @Binding var selection: ItemKind

  var body: some View {
    VStack(spacing: IntradaSpacing.controlGap) {
      SegmentedPills(
        options: [.piece, .exercise], selection: $selection, label: \.label,
        hint: \.caption, font: IntradaFont.segment,
        unselectedColor: IntradaColor.inkSecondary, layout: .fullWidthTrack)

      Text(selection.caption)
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.inkSecondary)
        .multilineTextAlignment(.center)
        .accessibilityHidden(true)  // spoken as the focused pill's hint instead
    }
  }
}
