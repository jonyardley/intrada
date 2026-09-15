import SharedTypes
import SwiftUI

/// An exercise's variations on the Add and Edit forms (#1783): renamed in place,
/// dragged to reorder, removed and added. Nothing here is saved until the form is.
struct VariationRowsSection: View {
  @Binding var rows: [VariationRow]
  var faulted = false

  @FocusState private var focusedRow: UUID?
  @State private var confirmingRemoval: VariationRow?

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      Text("Variations")
        .font(IntradaFont.metaMedium)
        .foregroundStyle(faulted ? IntradaColor.danger : IntradaColor.inkSecondary)
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
          HairlineDivider()
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
        UINotificationFeedbackGenerator().notificationOccurred(.warning)
        rows.removeAll { $0.id == row.id }
      }
      Button("Cancel", role: .cancel) {}
    } message: { _ in
      Text("Its marks go with it.")
    }
  }

  private var presets: some View {
    VStack(spacing: IntradaSpacing.controlGap) {
      AddRowButton(title: "Add 12 major keys") {
        rows = KeyHelper.circleMajor.map { VariationRow(label: $0) }
      }
      .accessibilityLabel("Add 12 major keys as this exercise's variations")
      AddRowButton(title: "Add 12 minor keys") {
        rows = KeyHelper.circleMinor.map { VariationRow(label: $0) }
      }
      .accessibilityLabel("Add 12 minor keys as this exercise's variations")
      AddRowButton(title: "Add a variation", style: .plain, action: addRow)
    }
    .padding(IntradaSpacing.card)
  }

  // VoiceOver gets move up and down, since a drag alone is not screen-reader-operable.
  private func rowView(_ current: VariationRow) -> some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      Image(systemName: "line.3.horizontal")
        .imageScale(.small)
        .foregroundStyle(IntradaColor.inkFaint)
        .frame(height: 44)
        .contentShape(Rectangle())
        .draggable(current.id.uuidString)
        .accessibilityLabel(
          current.label.isEmpty ? "Reorder this variation" : "Reorder \(current.label)"
        )
        .accessibilityHint("Drag to change this variation's position")
        .accessibilityAction(named: "Move up") { rows.move(current.id, by: -1) }
        .accessibilityAction(named: "Move down") { rows.move(current.id, by: 1) }
      TextField("e.g. C", text: label(of: current.id))
        .font(IntradaFont.field)
        .foregroundStyle(IntradaColor.ink)
        .focused($focusedRow, equals: current.id)
        .accessibilityLabel("Variation")
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
    .dropDestination(for: String.self) { items, _ in
      guard let dropped = items.first.flatMap(UUID.init(uuidString:)) else { return false }
      rows.move(dropped, before: current.id)
      return true
    }
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
