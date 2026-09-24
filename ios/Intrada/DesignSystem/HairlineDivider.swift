import SwiftUI

struct HairlineDivider: View {
  var axis: Axis = .horizontal

  var body: some View {
    Rectangle().fill(IntradaColor.hairline)
      .frame(width: axis == .vertical ? 1 : nil, height: axis == .horizontal ? 1 : nil)
  }
}
