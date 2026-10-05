import SwiftUI

/// Boxed answers three to a row; tapping one that is on turns it off.
struct ChoiceGrid<Option: Hashable>: View {
  let options: [Option]
  let isOn: (Option) -> Bool
  let label: (Option) -> String
  let identifier: String
  let onTap: (Option) -> Void

  @Environment(\.marker) private var marker
  @Environment(\.dynamicTypeSize) private var typeSize

  private var columns: [GridItem] {
    Array(
      repeating: GridItem(.flexible(), spacing: IntradaSpacing.controlGap),
      count: typeSize.isAccessibilitySize ? 1 : 3)
  }

  var body: some View {
    LazyVGrid(columns: columns, spacing: IntradaSpacing.controlGap) {
      ForEach(options, id: \.self) { option in
        box(option)
      }
    }
  }

  private func box(_ option: Option) -> some View {
    let on = isOn(option)
    let shape = RoundedRectangle(cornerRadius: IntradaRadius.control)
    return Button {
      Haptic.selection.play()
      onTap(option)
    } label: {
      Text(label(option))
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.ink)
        .lineLimit(2)
        .minimumScaleFactor(0.8)
        .multilineTextAlignment(.center)
        .frame(maxWidth: .infinity, minHeight: 44)
        .padding(.horizontal, IntradaSpacing.controlGap)
        .background(on ? marker : IntradaColor.cardFill, in: shape)
        .overlay(shape.strokeBorder(on ? IntradaColor.ink : IntradaColor.divider, lineWidth: 1))
    }
    .buttonStyle(.plain)
    .accessibilityLabel(label(option))
    .accessibilityAddTraits(on ? [.isSelected] : [])
    .accessibilityIdentifier(identifier)
  }
}

#if DEBUG
  #Preview("Choice grid") {
    ChoiceGrid(
      options: ["Comfortable", "Hard work", "Strained"], isOn: { $0 == "Strained" },
      label: { $0 }, identifier: "preview.choice", onTap: { _ in }
    )
    .padding()
    .background(IntradaColor.paperTop)
  }
#endif
