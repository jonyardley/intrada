import SharedTypes
import SwiftUI

/// Full-width Piece/Exercise selector — the accent-pill language of
/// `SegmentedPills`, shared by the add and edit forms, over a caption naming
/// what the selected type is for (#1361).
struct KindSegment: View {
  @Binding var selection: ItemKind

  var body: some View {
    VStack(spacing: IntradaSpacing.controlGap) {
      SegmentedPills(
        options: [.piece, .exercise], selection: $selection, label: \.label,
        font: IntradaFont.segment, unselectedColor: IntradaColor.inkSecondary,
        layout: .fullWidthTrack)

      Text(selection.caption)
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.inkSecondary)
        .multilineTextAlignment(.center)
        .frame(maxWidth: .infinity)
    }
  }
}
