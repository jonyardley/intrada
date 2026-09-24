import SwiftUI

struct HairlineDivider: View {
  var axis: Axis = .horizontal
  var colour: Color = IntradaColor.hairline

  var body: some View {
    Rectangle().fill(colour)
      .frame(width: axis == .vertical ? 1 : nil, height: axis == .horizontal ? 1 : nil)
  }
}
