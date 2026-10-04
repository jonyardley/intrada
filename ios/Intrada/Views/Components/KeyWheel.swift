import SharedTypes
import SwiftUI

/// The two-ring circle of fifths, major outside and minor inside. `chosen`
/// answers the spelling lit on a spoke, or nil; the caller owns what a tap
/// means, so one wheel serves the single key and the set of keys (#2247).
struct KeyWheel<Hub: View>: View {
  let chosen: (_ ring: Int, _ mode: Modality) -> String?
  let onTap: (_ ring: Int, _ mode: Modality) -> Void
  @ViewBuilder let hub: Hub

  var body: some View {
    ZStack {
      ForEach(0..<12, id: \.self) { ring in
        wedge(ring: ring, mode: .major)
        wedge(ring: ring, mode: .minor)
      }
      hubDisc
      ForEach(0..<12, id: \.self) { ring in
        majorLabel(ring: ring)
        minorLabel(ring: ring)
      }
    }
    .frame(width: 300, height: 300)
    // The geometry is fixed, so its labels stop growing where they still fit.
    .dynamicTypeSize(...DynamicTypeSize.xxLarge)
  }

  private func isSelected(ring: Int, mode: Modality) -> Bool {
    chosen(ring, mode) != nil
  }

  private func wedge(ring: Int, mode: Modality) -> some View {
    let isMajor = mode == .major
    let center = 270.0 + 30.0 * Double(ring)
    let shape = RingWedge(
      innerRadius: isMajor ? 105 : 60,
      outerRadius: isMajor ? 150 : 105,
      startAngle: .degrees(center - 15),
      endAngle: .degrees(center + 15))
    let selected = isSelected(ring: ring, mode: mode)
    let restFill = isMajor ? IntradaColor.cardFill : IntradaColor.surfaceSunken
    return
      shape
      .fill(selected ? IntradaColor.accent : restFill)
      .overlay(shape.stroke(IntradaColor.hairline, lineWidth: 1))
      .contentShape(shape)
      .onTapGesture { onTap(ring, mode) }
      .accessibilityElement()
      .accessibilityLabel(KeyHelper.wedgeAccessibilityLabel(ring: ring, mode: mode))
      .accessibilityAddTraits(selected ? [.isButton, .isSelected] : .isButton)
  }

  private var hubDisc: some View {
    ZStack {
      Circle()
        .fill(IntradaColor.cardFill)
        .overlay(Circle().stroke(IntradaColor.hairline, lineWidth: 1))
        .cardShadow()
      hub
    }
    .frame(width: 120, height: 120)
    .allowsHitTesting(false)
  }

  private func majorLabel(ring: Int) -> some View {
    let selected = isSelected(ring: ring, mode: .major)
    let primary = KeyHelper.primary(ring: ring, mode: .major)
    let point = point(radius: 127.5, ring: ring)
    return Group {
      if let alt = KeyHelper.enharmonicAlt(ring: ring, mode: .major) {
        let pair = displayedPair(ring: ring, mode: .major, primary: primary, alt: alt)
        VStack(spacing: 1) {
          Text(KeyHelper.prettify(pair.top))
            .font(IntradaFont.segment)
            .foregroundStyle(selected ? IntradaColor.onAccent : IntradaColor.ink)
          Text("\u{21C5} \(KeyHelper.prettify(pair.bottom))")  // ⇅
            .font(IntradaFont.small)
            .foregroundStyle(selected ? IntradaColor.onAccent : IntradaColor.inkSecondary)
        }
      } else {
        Text(KeyHelper.prettify(primary))
          .font(IntradaFont.segment)
          .foregroundStyle(selected ? IntradaColor.onAccent : IntradaColor.ink)
      }
    }
    .position(point)
    .allowsHitTesting(false)
  }

  private func minorLabel(ring: Int) -> some View {
    let selected = isSelected(ring: ring, mode: .minor)
    let spelling = chosen(ring, .minor) ?? KeyHelper.primary(ring: ring, mode: .minor)
    let label = "\(KeyHelper.prettify(spelling))m"
    let color = selected ? IntradaColor.onAccent : IntradaColor.inkSecondary
    return Group {
      // ⇅ stacked above the label so it fits the narrow inner wedge.
      if KeyHelper.enharmonicAlt(ring: ring, mode: .minor) != nil {
        VStack(spacing: 0) {
          Text("\u{21C5}").font(IntradaFont.small)  // ⇅
          Text(label).font(IntradaFont.secondary)
        }
      } else {
        Text(label).font(IntradaFont.secondary)
      }
    }
    .foregroundStyle(color)
    .position(point(radius: 82.5, ring: ring))
    .allowsHitTesting(false)
  }

  /// On an enharmonic spoke, the chosen spelling leads when selected; otherwise
  /// the circle's default spelling is on top.
  private func displayedPair(
    ring: Int, mode: Modality, primary: String, alt: String
  ) -> (top: String, bottom: String) {
    chosen(ring, mode) == alt ? (alt, primary) : (primary, alt)
  }

  /// Point at `radius` on the wheel for spoke `ring` (C at top, clockwise).
  private func point(radius: CGFloat, ring: Int) -> CGPoint {
    let radians = (270.0 + 30.0 * Double(ring)) * .pi / 180
    return CGPoint(
      x: 150 + radius * CGFloat(cos(radians)), y: 150 + radius * CGFloat(sin(radians)))
  }
}

private struct RingWedge: Shape {
  let innerRadius: CGFloat
  let outerRadius: CGFloat
  let startAngle: Angle
  let endAngle: Angle

  func path(in rect: CGRect) -> Path {
    let center = CGPoint(x: rect.midX, y: rect.midY)
    var path = Path()
    path.addArc(
      center: center, radius: outerRadius, startAngle: startAngle, endAngle: endAngle,
      clockwise: false)
    path.addArc(
      center: center, radius: innerRadius, startAngle: endAngle, endAngle: startAngle,
      clockwise: true)
    path.closeSubpath()
    return path
  }
}
