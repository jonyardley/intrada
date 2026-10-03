import SharedTypes
import SwiftUI

struct ProfileNameFields: View {
  @Environment(Store.self) private var store
  @Binding var name: String
  @Binding var instrument: String
  var faultedField: ProfileField?

  var body: some View {
    VStack(spacing: 0) {
      FormField(
        label: "Name", text: $name, placeholder: "Your name",
        autocapitalization: .words, faulted: faultedField == .name,
        identifier: "profileEdit.name")
      HairlineDivider()
      AutocompleteField(
        label: "Instrument", text: $instrument, placeholder: "e.g. Cello",
        suggestions: store.viewModel?.profile.instrumentNames ?? [],
        faulted: faultedField == .instrument,
        identifier: "profileEdit.instrument")
    }
    .cardSurface()
  }
}

struct HighlighterSwatches: View {
  @Binding var colour: HighlighterColour

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Eyebrow("Highlighter")
      LazyVGrid(
        columns: Array(repeating: GridItem(.flexible(), spacing: 0), count: 4),
        spacing: IntradaSpacing.cardCompact
      ) {
        ForEach(HighlighterColour.all, id: \.self) { swatch in
          Button {
            colour = swatch
            Haptic.selection.play()
          } label: {
            VStack(spacing: 6) {
              Circle()
                .fill(IntradaColor.marker(swatch))
                .frame(width: 40, height: 40)
                .overlay {
                  if swatch == colour {
                    Image(systemName: "checkmark")
                      .iconSize(.inline, weight: .bold)
                      .foregroundStyle(IntradaColor.onMarker)
                  }
                }
              Text(swatch.label)
                .font(IntradaFont.metaMedium)
                .foregroundStyle(swatch == colour ? IntradaColor.ink : IntradaColor.inkSecondary)
            }
            .frame(maxWidth: .infinity)
            .contentShape(Rectangle())
          }
          .buttonStyle(.plain)
          .accessibilityLabel(swatch.label)
          .accessibilityIdentifier("profileEdit.highlighter.\(swatch)")
          .accessibilityAddTraits(swatch == colour ? .isSelected : [])
        }
      }
      .padding(.vertical, IntradaSpacing.cardCompact)
      .padding(.horizontal, IntradaSpacing.controlGap)
      .cardSurface()
    }
  }
}
