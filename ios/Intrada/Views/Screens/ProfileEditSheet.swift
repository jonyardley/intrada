import SharedTypes
import SwiftUI

/// The profile's edit sheet: name, instrument with suggestions, the icon with
/// a way to pick another, and the eight swatches. Save sends one event; the
/// core validates, and the sheet dismisses only when it accepts (#1692).
struct ProfileEditSheet: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @Environment(\.dynamicTypeSize) private var dynamicTypeSize

  @State private var name = ""
  @State private var instrument = ""
  @State private var iconChoice: InstrumentIcon?
  @State private var colour: HighlighterColour = .butter
  @State private var formError: String?
  @State private var faultedField: ProfileField?
  @State private var choosingIcon = false

  init() {}

  #if DEBUG
    init(previewError: String, field: ProfileField) {
      _formError = State(initialValue: previewError)
      _faultedField = State(initialValue: field)
    }
  #endif

  private var profile: ProfileView? { store.viewModel?.profile }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        VStack(spacing: 0) {
          if let formError {
            FormErrorBanner(message: formError)
              .padding(.horizontal, IntradaSpacing.card)
              .padding(.top, IntradaSpacing.cardCompact)
              .transition(.move(edge: .top).combined(with: .opacity))
          }
          ScrollView {
            VStack(alignment: .leading, spacing: IntradaSpacing.card) {
              iconCard
              ProfileNameFields(name: $name, instrument: $instrument, faultedField: faultedField)
              HighlighterSwatches(colour: $colour)
            }
            .padding(IntradaSpacing.card)
          }
        }
      }
      .navigationTitle("Profile")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Cancel") { dismiss() }
            .accessibilityIdentifier("profileEdit.cancel")
        }
        ToolbarItem(placement: .confirmationAction) {
          Button("Save", action: save)
            .accessibilityIdentifier("profileEdit.save")
        }
      }
      .sheet(isPresented: $choosingIcon) {
        InstrumentIconPicker(
          suggested: profile?.suggestedIcon ?? .other, choice: $iconChoice)
      }
    }
    // The sheet wears the swatch under the finger; the rest of the app
    // follows once the core has accepted the save.
    .marker(colour)
    .onAppear(perform: load)
  }

  private func load() {
    guard let profile else { return }
    name = profile.name
    instrument = profile.instrument
    iconChoice = profile.iconChosen ? profile.icon : nil
    colour = profile.colour
  }

  private var shownIcon: InstrumentIcon {
    iconChoice ?? profile?.suggestedIcon ?? .other
  }

  private var iconCaption: String {
    if iconChoice != nil { return "Chosen by you" }
    if let suggested = profile?.suggestedIcon, suggested != .other {
      return "Matches \(suggested.tileLabel.lowercased())"
    }
    return "No match"
  }

  // Stacked at accessibility sizes so the caption keeps whole words and the button its label (#2126).
  private var iconCard: some View {
    let stacked = dynamicTypeSize.isAccessibilitySize
    let layout: AnyLayout =
      stacked
      ? AnyLayout(VStackLayout(alignment: .leading, spacing: IntradaSpacing.cardCompact))
      : AnyLayout(HStackLayout(spacing: IntradaSpacing.card))
    return layout {
      ProfileBadge(icon: shownIcon)
      VStack(alignment: .leading, spacing: 3) {
        Text("Icon")
          .font(IntradaFont.metaMedium)
          .foregroundStyle(IntradaColor.inkSecondary)
        Text(iconCaption)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      if !stacked {
        Spacer(minLength: IntradaSpacing.controlGap)
      }
      Button("Change") { choosingIcon = true }
        .font(IntradaFont.button)
        .foregroundStyle(IntradaColor.ink)
        .padding(.horizontal, IntradaSpacing.cardCompact)
        .frame(minHeight: 36)
        .background(IntradaColor.cardFill)
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.control)
            .stroke(IntradaColor.divider, lineWidth: 1)
        )
        .accessibilityLabel("Change icon")
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.card)
    .cardSurface()
  }

  // Never dismiss or celebrate until the core has accepted (#1595).
  private func save() {
    formError = nil
    faultedField = nil
    let profile = Profile(
      name: name, instrument: instrument, iconChoice: iconChoice, colour: colour)
    if let refusal = store.saveProfile(profile) {
      faultedField = refusal.field
      withAnimation { formError = refusal.message }
    } else {
      dismiss()
    }
  }
}

/// The instrument field's suggestion pool: vocabulary for the autocomplete,
/// not a domain decision; the core decides what each one matches.
enum InstrumentNames {
  static let suggestions: [String] = [
    "Accordion", "Alto saxophone", "Bagpipes", "Banjo", "Baritone", "Bass guitar", "Bassoon",
    "Cello", "Clarinet", "Cor anglais", "Cornet", "Double bass", "Drums", "Electric guitar",
    "Euphonium", "Flute", "French horn", "Guitar", "Harp", "Harpsichord", "Mandolin", "Marimba",
    "Oboe", "Organ", "Percussion", "Piano", "Piccolo", "Recorder", "Saxophone", "Soprano",
    "Tenor saxophone", "Timpani", "Trombone", "Trumpet", "Tuba", "Ukulele", "Viola", "Violin",
    "Voice",
  ]
}

#if DEBUG
  #Preview {
    ProfileEditSheet()
      .environment(Store.previewProfile)
  }
#endif
