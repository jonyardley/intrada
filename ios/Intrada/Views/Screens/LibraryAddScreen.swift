import SharedTypes
import SwiftUI

/// Create sheet for a new library item. Sends `Event.item(.add)` — the core
/// validates and reconciles via the temp-id mutate-response pattern; the shell
/// only collects field values. Tags are deferred to a later increment.
struct LibraryAddScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  @State private var kind: ItemKind
  @State private var title = ""
  @State private var composer = ""
  @State private var key = ""
  @State private var marking = ""
  @State private var bpm = ""
  @State private var notes = ""

  init(defaultKind: ItemKind = .piece) {
    _kind = State(initialValue: defaultKind)
  }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        ScrollView {
          VStack(spacing: 16) {
            KindSegment(selection: $kind)

            VStack(spacing: 0) {
              FormField(label: "Title", text: $title, placeholder: "Required")
              divider
              FormField(label: "Composer", text: $composer)
              divider
              FormField(label: "Key", text: $key)
            }
            .cardSurface()

            VStack(spacing: 0) {
              FormField(label: "Tempo marking", text: $marking, placeholder: "e.g. Allegro")
              divider
              FormField(label: "Beats per minute", text: $bpm, keyboard: .numberPad)
            }
            .cardSurface()

            FormField(label: "Notes", text: $notes, axis: .vertical)
              .cardSurface()
          }
          .padding(16)
        }
      }
      .navigationTitle("New \(kind.label)")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Cancel") { dismiss() }
        }
        ToolbarItem(placement: .confirmationAction) {
          Button("Add", action: add)
            .disabled(title.trimmingCharacters(in: .whitespaces).isEmpty)
        }
      }
    }
  }

  private var divider: some View {
    Rectangle().fill(IntradaColor.hairline).frame(height: 1)
  }

  private func add() {
    let input = CreateItem(
      title: title.trimmingCharacters(in: .whitespacesAndNewlines),
      kind: kind,
      composer: emptyToNil(composer),
      key: emptyToNil(key),
      tempo: buildTempo(),
      notes: emptyToNil(notes),
      tags: [])
    store.send(.item(.add(input)))
    UINotificationFeedbackGenerator().notificationOccurred(.success)
    dismiss()
  }

  private func emptyToNil(_ value: String) -> String? {
    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    return trimmed.isEmpty ? nil : trimmed
  }

  private func buildTempo() -> Tempo? {
    let mark = emptyToNil(marking)
    let beats = UInt16(bpm.trimmingCharacters(in: .whitespaces))
    if mark == nil && beats == nil { return nil }
    return Tempo(marking: mark, bpm: beats)
  }
}

/// Binary Piece/Exercise selector for the create form — the same accent-pill
/// language as `LibraryFilterTabs`, laid out full-width inside a segment track.
private struct KindSegment: View {
  @Binding var selection: ItemKind
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Namespace private var pill

  var body: some View {
    HStack(spacing: 4) {
      ForEach([ItemKind.piece, ItemKind.exercise], id: \.self) { kind in
        let isSelected = kind == selection
        Button {
          withAnimation(reduceMotion ? nil : .spring(response: 0.35, dampingFraction: 0.8)) {
            selection = kind
          }
        } label: {
          Text(kind.label)
            .font(.system(size: 14, weight: .medium))
            .foregroundStyle(isSelected ? IntradaColor.onAccent : IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 8)
            .background {
              if isSelected {
                Capsule()
                  .fill(IntradaColor.accent)
                  .matchedGeometryEffect(id: "kindPill", in: pill)
              }
            }
        }
        .buttonStyle(.plain)
        .accessibilityLabel(kind.label)
        .accessibilityAddTraits(isSelected ? [.isSelected] : [])
      }
    }
    .padding(4)
    .background(IntradaColor.cardFill, in: Capsule())
    .overlay(Capsule().stroke(IntradaColor.hairline, lineWidth: 1))
  }
}

#if DEBUG
  #Preview {
    LibraryAddScreen()
      .environment(Store.preview)
  }
#endif
