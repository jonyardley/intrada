import SharedTypes
import SwiftUI

/// Name and instrument in one card, shared by the profile's edit sheet and the
/// welcome's profile step.
struct ProfileNameFields: View {
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
        suggestions: InstrumentNames.suggestions, faulted: faultedField == .instrument,
        identifier: "profileEdit.instrument")
    }
    .cardSurface()
  }
}

/// The eight highlighter swatches under their eyebrow.
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

/// What a refused profile save leaves on screen.
struct ProfileSaveRefusal {
  let message: String
  let field: ProfileField?
}

extension Store {
  /// Sends the save and reads back the core's verdict: `nil` when it was
  /// accepted, so the caller may dismiss (#1595).
  func saveProfile(_ profile: Profile) -> ProfileSaveRefusal? {
    let accepted = sendAccepted(.profile(.save(profile)))
    guard
      let message = viewModel?.error ?? (accepted ? nil : "Couldn't save your profile. Try again.")
    else {
      Haptic.success.play()
      return nil
    }
    var field: ProfileField?
    if case .profile(let faulted) = viewModel?.errorTarget { field = faulted }
    send(.clearError)
    Haptic.error.play()
    UIAccessibility.post(notification: .announcement, argument: "Error: \(message)")
    return ProfileSaveRefusal(message: message, field: field)
  }
}
