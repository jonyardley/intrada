import SharedTypes
import SwiftUI

/// Full-width Piece/Exercise selector — the accent-pill language of
/// `SegmentedPills`, shared by the add and edit forms.
struct KindSegment: View {
  @Binding var selection: ItemKind

  var body: some View {
    SegmentedPills(
      options: [.piece, .exercise], selection: $selection, label: \.label,
      font: IntradaFont.segment, unselectedColor: IntradaColor.inkSecondary,
      layout: .fullWidthTrack)
  }
}
