import SharedTypes
import SwiftUI

enum SectionSheetTarget: Identifiable {
  case new
  case existing(SectionView)

  var id: String {
    switch self {
    case .new: ""
    case .existing(let section): section.id
    }
  }
}

/// Adds or edits one section (#2247). The core reads the bars as typed and
/// refuses the save whole, so the sheet stays open with its reason (#1595).
struct SectionSheet: View {
  let item: LibraryItemView
  let target: SectionSheetTarget

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var name: String
  @State private var bars: String
  @State private var kind: SectionKind
  @State private var bpm: String
  @State private var formError: String?
  @State private var confirmingRemoval = false

  init(item: LibraryItemView, target: SectionSheetTarget, previewError: String? = nil) {
    self.item = item
    self.target = target
    let section: SectionView? =
      if case .existing(let section) = target { section } else { nil }
    _name = State(initialValue: section?.name ?? "")
    _bars = State(initialValue: section.map(SectionEdits.barsText(of:)) ?? "")
    _kind = State(initialValue: section?.kind ?? .form)
    _bpm = State(initialValue: section?.targetBpm.map(String.init) ?? "")
    _formError = State(initialValue: previewError)
  }

  private var existing: SectionView? {
    if case .existing(let section) = target { return section }
    return nil
  }

  var body: some View {
    BottomSheet(
      title: existing?.label ?? "New section", detents: [.large], confirmationLabel: "Save",
      dismissesOnDone: false, onDone: save,
      leadingAction: {
        Button("Cancel") { dismiss() }
          .accessibilityIdentifier("sectionSheet.cancel")
      },
      content: { form }
    )
    // Alert, not confirmationDialog: it always shows Cancel, iPad included.
    .alert("Remove \(existing?.label ?? "this section")?", isPresented: $confirmingRemoval) {
      Button("Remove", role: .destructive) {
        if let existing { remove(existing) }
      }
      Button("Cancel", role: .cancel) {}
    }
  }

  private var form: some View {
    VStack(spacing: 0) {
      if let formError {
        FormErrorBanner(message: formError)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.top, IntradaSpacing.cardCompact)
          .transition(.move(edge: .top).combined(with: .opacity))
      }
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.card) {
          VStack(spacing: 0) {
            FormField(
              label: "Name", text: $name, placeholder: "A1, Coda, Exposition",
              autocapitalization: .words, identifier: "sectionSheet.name")
            HairlineDivider()
            FormField(
              label: "Bars", text: $bars, placeholder: "1 to 16",
              keyboard: .numbersAndPunctuation,
              identifier: "sectionSheet.bars", note: "Optional. Bar 12, or 12 to 14")
          }
          .cardSurface()
          FieldCard("Kind") {
            SegmentedPills(
              options: [SectionKind.form, .troubleSpot], selection: $kind,
              label: {
                $0 == .form
                  ? "Part of the \(item.itemType.label.lowercased())" : SectionText.trickySpot
              },
              identifier: { $0 == .form ? "sectionSheet.kind.form" : "sectionSheet.kind.spot" },
              layout: .fullWidthTrack)
          }
          FormField(
            label: "Target tempo", text: $bpm, placeholder: tempoPlaceholder,
            keyboard: .numberPad, identifier: "sectionSheet.bpm",
            note:
              "Beats per minute. Leave empty to use the \(item.itemType.label.lowercased())'s tempo"
          )
          .cardSurface()
          if let existing {
            DeleteButton(title: "Remove section") { confirmingRemoval = true }
              .accessibilityIdentifier("sectionSheet.remove")
          }
        }
        .padding(IntradaSpacing.card)
      }
    }
  }

  private var tempoPlaceholder: String {
    item.tempoBpm.map { "\($0)" } ?? ""
  }

  private func save() {
    let edit = SectionEdit(
      id: existing?.id, name: name, bars: .typed(bars), kind: kind, targetBpm: bpm)
    send(SectionEdits.saving(edit, into: item.sections))
  }

  private func remove(_ section: SectionView) {
    Haptic.warning.play()
    send(SectionEdits.removing(section.id, from: item.sections))
  }

  private func send(_ sections: [SectionEdit]) {
    formError = nil
    let error = store.sendFromSheet(.item(.updateSections(id: item.id, sections: sections)))
    withAnimation { formError = error }
    if error == nil { dismiss() }
  }
}
