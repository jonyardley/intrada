import SharedTypes
import SwiftUI

/// The profile's edit sheet: name, instrument with suggestions, the icon with
/// a way to pick another, and the eight swatches. Save sends one event; the
/// core validates, and the sheet dismisses only when it accepts (#1692).
struct ProfileEditSheet: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

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
    .onChange(of: instrument) { sendDraft() }
    .onChange(of: iconChoice) { sendDraft() }
    .onDisappear { store.send(.profile(.draftEnded)) }
  }

  private var draft: Profile {
    Profile(name: name, instrument: instrument, iconChoice: iconChoice, colour: colour)
  }

  private func sendDraft() {
    store.send(.profile(.draft(draft)))
  }

  private func load() {
    guard let profile else { return }
    name = profile.name
    instrument = profile.instrument
    iconChoice = profile.iconChosen ? profile.icon : nil
    colour = profile.colour
    sendDraft()
  }

  // Until the core has the draft, the saved profile's icon stands in (#2519).
  private var shownIcon: InstrumentIcon {
    profile?.draftIcon ?? profile?.icon ?? .other
  }

  private var iconCard: some View {
    HStack(spacing: IntradaSpacing.card) {
      ProfileBadge(icon: shownIcon)
      FieldLabel("Icon")
      Spacer(minLength: IntradaSpacing.controlGap)
      // The frame sits inside the label so the whole 44pt box takes the tap (#2318).
      Button {
        choosingIcon = true
      } label: {
        Text("Change")
          .font(IntradaFont.button)
          .foregroundStyle(IntradaColor.ink)
          .padding(.horizontal, IntradaSpacing.cardCompact)
          .frame(minHeight: 44)
          .background(IntradaColor.cardFill)
          .overlay(
            RoundedRectangle(cornerRadius: IntradaRadius.control)
              .stroke(IntradaColor.divider, lineWidth: 1)
          )
          .contentShape(RoundedRectangle(cornerRadius: IntradaRadius.control))
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Change icon")
      .accessibilityIdentifier("profileEdit.changeIcon")
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
    if let refusal = store.saveProfile(draft) {
      faultedField = refusal.field
      withAnimation { formError = refusal.message }
    } else {
      dismiss()
    }
  }
}

#if DEBUG
  #Preview {
    ProfileEditSheet()
      .environment(Store.previewProfile)
  }
#endif
