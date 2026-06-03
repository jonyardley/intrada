import SharedTypes
import SwiftUI

/// Detail for a library item: type badge, key/tempo, notes, tags, and delete.
struct LibraryDetailScreen: View {
  let item: LibraryItemView

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var confirmingDelete = false
  @State private var editing = false

  var body: some View {
    ScreenScaffold(title: item.title, subtitle: subtitle) {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.card) {
          TypeBadge(kind: item.itemType)

          if !detailRows.isEmpty {
            VStack(spacing: 0) {
              ForEach(Array(detailRows.enumerated()), id: \.offset) { index, row in
                if index > 0 {
                  HairlineDivider()
                }
                DetailRow(label: row.label, value: row.value)
              }
            }
            .cardSurface()
          }

          if let notes = item.notes, !notes.isEmpty {
            Text(notes)
              .font(IntradaFont.body)
              .foregroundStyle(IntradaColor.inkSecondary)
              .frame(maxWidth: .infinity, alignment: .leading)
              .padding(IntradaSpacing.card)
              .cardSurface()
          }

          if !item.tags.isEmpty {
            tags
          }

          deleteButton
            .padding(.top, IntradaSpacing.controlGap)
        }
        .padding(IntradaSpacing.card)
      }
      .scrollEdgeShadow()
    }
    .navigationBarTitleDisplayMode(.inline)
    .toolbar {
      ToolbarItem(placement: .topBarTrailing) {
        Button("Edit") { editing = true }
      }
    }
    .sheet(isPresented: $editing) {
      LibraryEditScreen(item: item)
        .environment(store)
    }
    // Alert (not confirmationDialog): always renders the Cancel button, incl.
    // iPad/regular-width where a confirmationDialog popover hides it.
    .alert("Delete \(item.title)?", isPresented: $confirmingDelete) {
      Button("Delete", role: .destructive, action: delete)
      Button("Cancel", role: .cancel) {}
    } message: {
      Text("This can't be undone.")
    }
  }

  private var deleteButton: some View {
    Button(role: .destructive) {
      confirmingDelete = true
    } label: {
      Label("Delete \(item.itemType.label.lowercased())", systemImage: "trash")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.danger)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.cardCompact)
    }
    .buttonStyle(.plain)
  }

  private func delete() {
    UINotificationFeedbackGenerator().notificationOccurred(.warning)
    store.send(.item(.delete(id: item.id)))
    dismiss()
  }

  private var subtitle: String? {
    item.subtitle.isEmpty ? nil : item.subtitle
  }

  private var detailRows: [(label: String, value: String)] {
    var rows: [(String, String)] = []
    if let key = item.keyDisplay { rows.append(("Key", key)) }
    if let tempo = item.tempoDisplay { rows.append(("Tempo", tempo)) }
    return rows
  }

  private var tags: some View {
    ScrollView(.horizontal, showsIndicators: false) {
      HStack(spacing: IntradaSpacing.controlGap) {
        ForEach(item.tags, id: \.self) { tag in
          TagChip(tag, style: .outlined)
        }
      }
    }
  }
}

private struct DetailRow: View {
  let label: String
  let value: String

  var body: some View {
    HStack {
      Text(label)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkSecondary)
      Spacer(minLength: 16)
      Text(value)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.trailing)
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.card)
    .accessibilityElement(children: .combine)
    .accessibilityLabel("\(label), \(value)")
  }
}

#if DEBUG
  #Preview("Piece") {
    NavigationStack {
      LibraryDetailScreen(item: .previewDetail)
    }
    .environment(Store.preview)
  }

  #Preview("Minimal") {
    NavigationStack {
      LibraryDetailScreen(item: .previewMinimal)
    }
    .environment(Store.preview)
  }
#endif
