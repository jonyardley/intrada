import SharedTypes
import SwiftUI

/// Inline circle-of-fifths key selector: a collapsed row expands a two-ring
/// wheel in place (the iOS date-picker pattern). Binds the structured tonic
/// (`key`) + `modality`. All selection logic lives in `KeyHelper`.
struct KeyPicker: View {
  let label: String
  @Binding var key: String
  @Binding var modality: Modality?

  @State private var expanded: Bool
  @Environment(\.accessibilityReduceMotion) private var reduceMotion

  /// `initiallyExpanded` is for previews/snapshot tests only — the wheel is
  /// otherwise driven by the row tap.
  init(
    label: String, key: Binding<String>, modality: Binding<Modality?>,
    initiallyExpanded: Bool = false
  ) {
    self.label = label
    self._key = key
    self._modality = modality
    self._expanded = State(initialValue: initiallyExpanded)
  }

  private var selection: KeyHelper.Selection? {
    KeyHelper.selection(key: key, modality: modality)
  }

  var body: some View {
    VStack(spacing: 0) {
      row
        // Card→clear row background masks the wheel as it slides behind on open.
        .background(
          LinearGradient(
            stops: [
              .init(color: IntradaColor.cardFill, location: 0),
              .init(color: IntradaColor.cardFill, location: 0.85),
              .init(color: IntradaColor.cardFill.opacity(0), location: 1),
            ],
            startPoint: .top, endPoint: .bottom)
        )
        .zIndex(1)
      if expanded {
        HStack {
          Spacer(minLength: 0)
          wheel
          Spacer(minLength: 0)
        }
        .padding(.top, IntradaSpacing.cardCompact)
        .padding(.bottom, 20)
        .transition(.move(edge: .top).combined(with: .opacity))
      }
    }
    // Clip the reveal so the expanding wheel can't bleed over the rows above.
    .clipped()
  }

  // ── Collapsed row ──

  private var row: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      VStack(alignment: .leading, spacing: 4) {
        Text(label)
          .font(IntradaFont.metaMedium)
          .foregroundStyle(IntradaColor.inkSecondary)
        if let display = KeyHelper.display(key: key, modality: modality) {
          Text(display)
            .font(IntradaFont.field)
            .foregroundStyle(IntradaColor.accent)
        } else {
          Text("Select a key")
            .font(IntradaFont.field)
            .foregroundStyle(IntradaColor.inkFaint)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)

      if !key.isEmpty {
        Button {
          key = ""
          modality = nil
          UIImpactFeedbackGenerator(style: .light).impactOccurred()
        } label: {
          Image(systemName: "xmark.circle.fill")
            .foregroundStyle(IntradaColor.inkFaint)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Clear key")
      }

      Image(systemName: "chevron.down")
        .font(IntradaFont.metaMedium)
        .foregroundStyle(IntradaColor.inkFaint)
        .rotationEffect(.degrees(expanded ? 180 : 0))
        // Keep clear distance from the clear (×) button so a tap aimed at the
        // chevron doesn't land on clear and wipe the key.
        .padding(.leading, IntradaSpacing.cardCompact)
    }
    .padding(.vertical, 10)
    .padding(.horizontal, IntradaSpacing.card)
    .contentShape(Rectangle())
    .onTapGesture {
      withAnimation(reduceMotion ? nil : .spring(response: 0.35, dampingFraction: 0.8)) {
        expanded.toggle()
      }
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
    .accessibilityElement(children: .combine)
    .accessibilityAddTraits(.isButton)
    .accessibilityLabel(rowAccessibilityLabel)
    .accessibilityHint(expanded ? "Collapses the key wheel" : "Expands the key wheel")
  }

  private var rowAccessibilityLabel: String {
    if let sel = selection {
      return "\(label), \(KeyHelper.accessibilityLabel(sel.spelling, mode: sel.mode))"
    }
    if key.isEmpty {
      return "\(label), no key selected"
    }
    return "\(label), \(key)"
  }

  // ── Wheel ──

  private var wheel: some View {
    ZStack {
      ForEach(0..<12, id: \.self) { ring in
        wedge(ring: ring, mode: .major)
        wedge(ring: ring, mode: .minor)
      }
      hub
      ForEach(0..<12, id: \.self) { ring in
        majorLabel(ring: ring)
        minorLabel(ring: ring)
      }
    }
    .frame(width: 300, height: 300)
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
      .onTapGesture { tap(ring: ring, mode: mode) }
      .accessibilityElement()
      .accessibilityLabel(KeyHelper.wedgeAccessibilityLabel(ring: ring, mode: mode))
      .accessibilityAddTraits(selected ? [.isButton, .isSelected] : .isButton)
  }

  private var hub: some View {
    ZStack {
      Circle()
        .fill(IntradaColor.cardFill)
        .overlay(Circle().stroke(IntradaColor.hairline, lineWidth: 1))
        .cardShadow()
      if let sel = selection {
        VStack(spacing: 0) {
          Text(KeyHelper.prettify(sel.spelling))
            .font(IntradaFont.cardTitle(30))
            .foregroundStyle(IntradaColor.ink)
          Text(KeyHelper.modeWord(sel.mode))
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      } else {
        VStack(spacing: 2) {
          Text("\u{266A}")  // ♪
            .font(IntradaFont.cardTitle(26))
            .foregroundStyle(IntradaColor.inkFaint)
          Text("Select a key")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkFaint)
        }
      }
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
            .font(IntradaFont.cardTitle(15))
            .foregroundStyle(selected ? IntradaColor.onAccent : IntradaColor.ink)
          Text("\u{21C5} \(KeyHelper.prettify(pair.bottom))")  // ⇅
            .font(IntradaFont.micro)
            .foregroundStyle(selected ? IntradaColor.onAccent : IntradaColor.inkFaint)
        }
      } else {
        Text(KeyHelper.prettify(primary))
          .font(IntradaFont.cardTitle(15))
          .foregroundStyle(selected ? IntradaColor.onAccent : IntradaColor.ink)
      }
    }
    .position(point)
    .allowsHitTesting(false)
  }

  private func minorLabel(ring: Int) -> some View {
    let selected = isSelected(ring: ring, mode: .minor)
    let spelling =
      (selection.flatMap { $0.ring == ring && $0.mode == .minor ? $0.spelling : nil })
      ?? KeyHelper.primary(ring: ring, mode: .minor)
    let label = "\(KeyHelper.prettify(spelling))m"
    let color = selected ? IntradaColor.onAccent : IntradaColor.inkSecondary
    return Group {
      // ⇅ stacked above the label so it fits the narrow inner wedge.
      if KeyHelper.enharmonicAlt(ring: ring, mode: .minor) != nil {
        VStack(spacing: 0) {
          Text("\u{21C5}").font(IntradaFont.micro)  // ⇅
          Text(label).font(IntradaFont.meta)
        }
      } else {
        Text(label).font(IntradaFont.meta)
      }
    }
    .foregroundStyle(color)
    .position(point(radius: 82.5, ring: ring))
    .allowsHitTesting(false)
  }

  // ── Helpers ──

  private func isSelected(ring: Int, mode: Modality) -> Bool {
    selection.map { $0.ring == ring && $0.mode == mode } ?? false
  }

  /// On an enharmonic spoke, the chosen spelling leads when selected; otherwise
  /// the circle's default spelling is on top.
  private func displayedPair(
    ring: Int, mode: Modality, primary: String, alt: String
  ) -> (top: String, bottom: String) {
    if let sel = selection, sel.ring == ring, sel.mode == mode, sel.spelling == alt {
      return (alt, primary)
    }
    return (primary, alt)
  }

  /// Point at `radius` on the wheel for spoke `ring` (C at top, clockwise).
  private func point(radius: CGFloat, ring: Int) -> CGPoint {
    let radians = (270.0 + 30.0 * Double(ring)) * .pi / 180
    return CGPoint(
      x: 150 + radius * CGFloat(cos(radians)), y: 150 + radius * CGFloat(sin(radians)))
  }

  private func tap(ring: Int, mode: Modality) {
    let result = KeyHelper.nextOnTap(
      currentKey: key, currentModality: modality, ring: ring, mode: mode)
    key = result.tonic
    modality = result.modality
    if result.flipped {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    } else {
      UISelectionFeedbackGenerator().selectionChanged()
    }
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

#if DEBUG
  #Preview {
    struct Demo: View {
      @State private var emptyKey = ""
      @State private var emptyModality: Modality? = nil
      @State private var minorKey = "A"
      @State private var minorModality: Modality? = .minor
      @State private var enhKey = "Gb"
      @State private var enhModality: Modality? = .major
      var body: some View {
        ZStack {
          PaperBackground()
          ScrollView {
            VStack(spacing: IntradaSpacing.card) {
              VStack(spacing: 0) {
                KeyPicker(label: "Key", key: $emptyKey, modality: $emptyModality)
              }.cardSurface()
              VStack(spacing: 0) {
                KeyPicker(label: "Key", key: $minorKey, modality: $minorModality)
              }.cardSurface()
              VStack(spacing: 0) {
                KeyPicker(label: "Key", key: $enhKey, modality: $enhModality)
              }.cardSurface()
            }
            .padding(IntradaSpacing.card)
          }
        }
      }
    }
    return Demo()
  }
#endif
