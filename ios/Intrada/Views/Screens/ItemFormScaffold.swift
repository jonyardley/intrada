import SharedTypes
import SwiftUI

/// Shared body for the add/edit item sheets: the field cards plus the
/// confirm/cancel toolbar and the error-reconcile flow. `send` dispatches the
/// add/update event; the scaffold owns the "don't celebrate until the core
/// confirms" handling so both screens behave identically.
struct ItemFormScaffold: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  @Bindable var form: ItemFormModel
  let title: String
  let confirmLabel: String
  let composerSuggestions: [String]
  let tagSuggestions: [String]
  let send: () -> Void

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        VStack(spacing: 0) {
          if let formError = form.formError {
            FormErrorBanner(message: formError)
              .padding(.horizontal, 16)
              .padding(.top, 12)
              .transition(.move(edge: .top).combined(with: .opacity))
          }
          ScrollView {
            VStack(spacing: 16) {
              KindSegment(selection: $form.kind)

              VStack(spacing: 0) {
                FormField(label: "Title", text: $form.title, placeholder: "Required")
                divider
                AutocompleteField(
                  label: "Composer", text: $form.composer, suggestions: composerSuggestions)
                divider
                KeyPicker(label: "Key", key: $form.key, modality: $form.modality)
              }
              .cardSurface()

              VStack(spacing: 0) {
                FormField(label: "Tempo marking", text: $form.marking, placeholder: "e.g. Allegro")
                divider
                FormField(label: "Beats per minute", text: $form.bpm, keyboard: .numberPad)
              }
              .cardSurface()

              FormField(label: "Notes", text: $form.notes, axis: .vertical)
                .cardSurface()

              VStack(spacing: 0) {
                TagChipInput(label: "Tags", tags: $form.tags, suggestions: tagSuggestions)
              }
              .cardSurface()
            }
            .padding(16)
          }
        }
      }
      .navigationTitle(title)
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Cancel") { dismiss() }
        }
        ToolbarItem(placement: .confirmationAction) {
          Button(confirmLabel, action: confirm)
            .disabled(!form.canSubmit)
        }
      }
    }
  }

  private var divider: some View {
    Rectangle().fill(IntradaColor.hairline).frame(height: 1)
  }

  // Don't celebrate or dismiss until the core confirms: a validation reject or
  // failed local write surfaces in viewModel.error, which we keep on screen.
  private func confirm() {
    form.formError = nil
    send()
    if let error = store.viewModel?.error {
      withAnimation { form.formError = error }
      UINotificationFeedbackGenerator().notificationOccurred(.error)
      UIAccessibility.post(notification: .announcement, argument: "Error: \(error)")
    } else {
      UINotificationFeedbackGenerator().notificationOccurred(.success)
      dismiss()
    }
  }
}
