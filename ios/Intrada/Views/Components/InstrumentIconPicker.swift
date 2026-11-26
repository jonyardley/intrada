import SharedTypes
import SwiftUI

/// The second sheet from the profile's edit card: the instrument's match
/// first, then every icon. The pick goes to the core as tapped; the core drops
/// one that matches the instrument being saved, so the icon follows it (#2519).
struct InstrumentIconPicker: View {
  let suggested: InstrumentIcon
  @Binding var choice: InstrumentIcon?

  var body: some View {
    BottomSheet(title: "Icon", detents: [.large]) {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
          if suggested != .other {
            SectionTitle("Matches \(suggested.tileLabel.lowercased())")
            grid(of: [suggested])
              .padding(.bottom, IntradaSpacing.cardCompact)
          }
          SectionTitle("All icons")
          grid(of: InstrumentIcon.all.filter { suggested == .other || $0 != suggested })
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.controlGap)
        .padding(.bottom, IntradaSpacing.section)
      }
    }
  }

  private var shown: InstrumentIcon { choice ?? suggested }

  private func grid(of icons: [InstrumentIcon]) -> some View {
    LazyVGrid(
      columns: Array(
        repeating: GridItem(.flexible(), spacing: IntradaSpacing.controlGap),
        count: 4),
      spacing: IntradaSpacing.controlGap
    ) {
      ForEach(icons, id: \.self) { icon in
        tile(icon)
      }
    }
  }

  private func tile(_ icon: InstrumentIcon) -> some View {
    let selected = icon == shown
    return Button {
      choice = icon
      Haptic.selection.play()
    } label: {
      VStack(spacing: 6) {
        InstrumentGlyph(icon: icon, size: IntradaGlyph.bar)
          .foregroundStyle(IntradaColor.ink)
        Text(icon.tileLabel)
          .font(IntradaFont.smallMedium)
          .foregroundStyle(selected ? IntradaColor.ink : IntradaColor.inkSecondary)
          .lineLimit(1)
          .minimumScaleFactor(0.8)
      }
      .frame(maxWidth: .infinity)
      .frame(minHeight: 84)
      .padding(.horizontal, 4)
      .background(selected ? marker : IntradaColor.cardFill)
      .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
      .overlay(
        RoundedRectangle(cornerRadius: IntradaRadius.card)
          .stroke(selected ? IntradaColor.ink : IntradaColor.hairline, lineWidth: 1)
      )
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel(icon.accessibilityLabel)
    .accessibilityAddTraits(selected ? .isSelected : [])
    .accessibilityIdentifier("iconPicker.tile")
  }

  @Environment(\.marker) private var marker
}

#if DEBUG
  #Preview {
    struct Demo: View {
      @State private var choice: InstrumentIcon? = .harp
      var body: some View {
        InstrumentIconPicker(suggested: .cello, choice: $choice)
      }
    }
    return Demo()
  }
#endif
