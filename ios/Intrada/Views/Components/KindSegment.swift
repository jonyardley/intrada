import SharedTypes
import SwiftUI

/// Full-width Piece/Exercise selector — the accent-pill language of
/// `SegmentedPills`, shared by the add and edit forms. The caption lives here
/// rather than on the create screen because the edit form flips type too.
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
