import SharedTypes
import SwiftUI

/// A piece or exercise's sections in score order (#2247): tap one to edit it,
/// or Reorder to drag and remove them, saved together on Done.
struct SectionsSection: View {
  let item: LibraryItemView
  let onAdd: () -> Void
  let onEdit: (SectionView) -> Void

  @Environment(Store.self) private var store
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @State private var reordering: [SectionView]?
  @State private var drag = DragReorder<String>()

  init(
    item: LibraryItemView, startReordering: Bool = false, onAdd: @escaping () -> Void,
    onEdit: @escaping (SectionView) -> Void
  ) {
    self.item = item
    self.onAdd = onAdd
    self.onEdit = onEdit
    _reordering = State(initialValue: startReordering ? item.sections : nil)
  }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: "Sections", action: headerAction)
      if let reordering {
        VStack(spacing: 0) {
          ForEach(Array(reordering.enumerated()), id: \.element.id) { index, section in
            if index > 0 {
              HairlineDivider()
            }
            reorderRow(section)
          }
        }
        .cardSurface()
      } else if item.sections.isEmpty {
        AddRowButton(title: "Add sections", hint: "A1, B, Coda, or bars 12 to 14", action: onAdd)
          .accessibilityLabel("Add sections to this \(item.itemType.label.lowercased())")
          .accessibilityIdentifier("sections.add")
      } else {
        VStack(spacing: 0) {
          ForEach(item.sections, id: \.id) { section in
            Button {
              onEdit(section)
            } label: {
              SectionRow(section: section)
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("sections.row")
            HairlineDivider()
          }
          AddRowButton(title: "Add section", style: .plain, action: onAdd)
            .accessibilityLabel("Add a section")
            .accessibilityIdentifier("sections.add")
        }
        .cardSurface()
      }
    }
    .onChange(of: item.sections.isEmpty) { _, isEmpty in
      if isEmpty { reordering = nil }
    }
  }

  private var headerAction: SectionHeader.Action? {
    if reordering != nil {
      return .init(
        title: "Done", accessibilityLabel: "Done reordering sections",
        identifier: "sections.header",
        perform: finishReordering)
    }
    guard !item.sections.isEmpty else { return nil }
    return .init(
      title: "Reorder", accessibilityLabel: "Reorder or remove sections",
      identifier: "sections.header", perform: { reordering = item.sections })
  }

  private func reorderRow(_ section: SectionView) -> some View {
    let isDragged = drag.dragged == section.id
    return HStack(spacing: IntradaSpacing.cardCompact) {
      Button {
        reordering?.removeAll { $0.id == section.id }
      } label: {
        Image(systemName: "minus.circle")
          .iconSize(.control)
          .foregroundStyle(IntradaColor.danger)
          .frame(width: 44, height: 44)
          .contentShape(Rectangle())
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Remove \(section.label)")
      SectionTitleStack(section: section)
        .padding(.vertical, IntradaSpacing.cardCompact)
      Image(systemName: "line.3.horizontal")
        .imageScale(.small)
        .foregroundStyle(isDragged ? IntradaColor.ink : IntradaColor.inkFaintIcon)
        .frame(width: 44, height: 44)
        .contentShape(Rectangle())
        .gesture(
          drag.gesture(
            for: section.id, ids: { reordering?.map(\.id) ?? [] }, reduceMotion: reduceMotion,
            move: move)
        )
        .accessibilityLabel("Reorder \(section.label)")
        .accessibilityHint("Drag to change this section's position")
        .accessibilityIdentifier("sections.reorder")
        .accessibilityAction(named: "Move up") { move(section.id, by: -1) }
        .accessibilityAction(named: "Move down") { move(section.id, by: 1) }
    }
    .padding(.leading, IntradaSpacing.controlGap)
    .padding(.trailing, IntradaSpacing.controlGap)
    .reorderableRow(section.id, in: drag)
  }

  private func move(_ id: String, by step: Int) {
    guard var rows = reordering, let from = rows.firstIndex(where: { $0.id == id }),
      rows.indices.contains(from + step)
    else { return }
    rows.swapAt(from, from + step)
    reordering = rows
  }

  // Removals and the new order land as one write, and the list stays in
  // reorder until the core accepts it, so a refusal loses nothing. Only rows
  // this item still holds go out: a list carried over from another item would
  // replace that item's sections (iPad keeps the pane's identity).
  private func finishReordering() {
    guard let rows = reordering else { return }
    let current = Swift.Set(item.sections.map(\.id))
    guard rows.allSatisfy({ current.contains($0.id) }) else {
      reordering = nil
      return
    }
    let unchanged = rows.map(\.id) == item.sections.map(\.id)
    if unchanged
      || store.sendAccepted(
        .item(.updateSections(id: item.id, sections: rows.map(SectionEdits.edit(from:)))))
    {
      if !unchanged { Haptic.impact.play() }
      reordering = nil
    }
  }
}

private struct SectionTitleStack: View {
  let section: SectionView

  var body: some View {
    VStack(alignment: .leading, spacing: 3) {
      Text(section.label)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.leading)
        .fixedSize(horizontal: false, vertical: true)
      if section.barsCaption != nil || section.kind == .troubleSpot || section.isWeakest {
        HStack(spacing: IntradaSpacing.controlGap) {
          if let bars = section.barsCaption {
            Text(bars)
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
          if section.kind == .troubleSpot {
            TagChip(SectionText.trickySpot)
          }
          if section.isWeakest {
            TagChip(SectionText.weakest)
          }
        }
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
  }
}

private struct SectionRow: View {
  let section: SectionView

  var body: some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      ScoreRing(score: section.latestScore.map(Int.init), size: 32)
      SectionTitleStack(section: section)
      if let bpm = section.targetBpm {
        Text("♩ \(bpm)")
          .font(IntradaFont.figure)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      Image(systemName: "chevron.right")
        .font(IntradaFont.secondary)
        .foregroundStyle(IntradaColor.inkFaintIcon)
        .accessibilityHidden(true)
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.card)
    .frame(minHeight: 44)
    .contentShape(Rectangle())
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(SectionText.spoken(section))
    .accessibilityHint("Opens this section to edit it")
  }
}

enum SectionText {
  static let trickySpot = "Tricky spot"
  static let weakest = "Weakest"

  /// "A1, bars 1 to 14, 8 of 10" or "Bars 19 to 20, tricky spot, 54 beats per
  /// minute, 4 of 10, weakest".
  static func spoken(_ section: SectionView) -> String {
    var parts = [section.label]
    if let bars = section.barsCaption { parts.append(bars.lowercased()) }
    if section.kind == .troubleSpot { parts.append(trickySpot.lowercased()) }
    if let bpm = section.targetBpm { parts.append("\(bpm) beats per minute") }
    parts.append(section.caption)
    if section.isWeakest { parts.append(weakest.lowercased()) }
    return parts.joined(separator: ", ")
  }
}
