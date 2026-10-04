import SharedTypes
import SwiftUI

/// An exercise's variations on the Add and Edit forms (#1783): dragged to
/// reorder, removed and added; only a row typed here is editable (#2246).
/// Nothing here is saved until the form is.
struct VariationRowsSection: View {
  @Binding var rows: [VariationRow]
  var faulted = false

  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @FocusState private var focusedRow: UUID?
  @State private var confirmingRemoval: VariationRow?
  @State private var drag = DragReorder<UUID>()

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      FieldLabel("Variations", tint: faulted ? IntradaColor.danger : IntradaColor.inkSecondary)
        .accessibilityAddTraits(.isHeader)
        .accessibilityHint(FaultMark.spoken(faulted))
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, 10)
      if rows.isEmpty {
        presets
      } else {
        ForEach(rows) { row in
          rowView(row)
        }
        AddRowButton(title: "Add a variation", style: .plain, action: addRow)
      }
    }
    .faultWash(faulted)
    // Alert, not confirmationDialog: it always shows Cancel, iPad included.
    .alert(
      Text("Remove \(confirmingRemoval?.label ?? "")?"), isPresented: confirmsRemoval,
      presenting: confirmingRemoval
    ) { row in
      Button("Remove", role: .destructive) {
        Haptic.warning.play()
        rows.removeAll { $0.id == row.id }
      }
      Button("Cancel", role: .cancel) {}
    } message: { _ in
      Text("Its marks go with it.")
    }
  }

  private var presets: some View {
    VStack(spacing: IntradaSpacing.controlGap) {
      AddRowButton(title: "Add a variation", style: .plain, action: addRow)
    }
    .padding(IntradaSpacing.card)
  }

  // VoiceOver gets move up and down, since a drag alone is not screen-reader-operable.
  private func rowView(_ current: VariationRow) -> some View {
    let isDragged = drag.dragged == current.id
    return VStack(spacing: 0) {
      HStack(spacing: IntradaSpacing.cardCompact) {
        Image(systemName: "line.3.horizontal")
          .imageScale(.small)
          .foregroundStyle(isDragged ? IntradaColor.ink : IntradaColor.inkFaintIcon)
          .frame(height: 44)
          .contentShape(Rectangle().inset(by: -8))
          .gesture(
            drag.gesture(for: current.id, ids: { rows.map(\.id) }, reduceMotion: reduceMotion) {
              rows.move($0, by: $1)
            }
          )
          .accessibilityLabel(
            current.label.isEmpty ? "Reorder this variation" : "Reorder \(current.label)"
          )
          .accessibilityHint("Drag to change this variation's position")
          .accessibilityIdentifier("variationRow.reorder")
          .accessibilityAction(named: "Move up") { rows.move(current.id, by: -1) }
          .accessibilityAction(named: "Move down") { rows.move(current.id, by: 1) }
        if current.variantId == nil {
          TextField("e.g. C", text: label(of: current.id))
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
            .focused($focusedRow, equals: current.id)
            .accessibilityLabel("Variation")
            .accessibilityIdentifier("variationRow.label")
        } else {
          // A saved variation is renamed across the library, not on one item (#2247).
          Text(current.label)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
            .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
            .accessibilityLabel("Variation \(current.label)")
            .accessibilityIdentifier("variationRow.label")
        }
        Button {
          if current.hasMarks {
            confirmingRemoval = current
          } else {
            rows.removeAll { $0.id == current.id }
          }
        } label: {
          Image(systemName: "minus.circle")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.danger)
            .frame(width: 44, height: 44)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(
          current.label.isEmpty ? "Remove this variation" : "Remove \(current.label)")
      }
      .padding(.leading, IntradaSpacing.card)
      .padding(.trailing, IntradaSpacing.controlGap)
      HairlineDivider()
    }
    .reorderableRow(current.id, in: drag)
  }

  // By id, not index: a row removed while its field still has focus would
  // otherwise be read past the end of the list.
  private func label(of id: UUID) -> Binding<String> {
    Binding(
      get: { rows.first { $0.id == id }?.label ?? "" },
      set: { label in
        guard let index = rows.firstIndex(where: { $0.id == id }) else { return }
        rows[index].label = label
      })
  }

  private var confirmsRemoval: Binding<Bool> {
    Binding(
      get: { confirmingRemoval != nil },
      set: { if !$0 { confirmingRemoval = nil } })
  }

  private func addRow() {
    let row = VariationRow(label: "")
    rows.append(row)
    focusedRow = row.id
  }
}

#if DEBUG
  #Preview("Empty") {
    VariationRowsSection(rows: .constant([]))
      .cardSurface()
      .padding(IntradaSpacing.card)
      .background(LinearGradient.paper)
  }

  #Preview("Rows") {
    VariationRowsSection(
      rows: .constant(
        [
          VariationRow(variantId: "c", label: "C", hasMarks: true),
          VariationRow(label: "F"),
          VariationRow(label: ""),
        ])
    )
    .cardSurface()
    .padding(IntradaSpacing.card)
    .background(LinearGradient.paper)
  }
#endif
