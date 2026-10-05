import SharedTypes
import SwiftUI

/// Splits an entry into its sections, each with its own minutes (#2315). The
/// core keeps the minutes summing to the item's planned time: a segment sent
/// at zero shares what the others leave.
struct EntrySegmentsCard: View {
  let segments: [SegmentView]
  let sections: [SectionView]
  let hasPlannedTime: Bool
  let send: ([Segment]) -> Void
  let step: (_ sectionId: String, _ minutes: Int8) -> Void

  private var unplanned: [SectionView] {
    sections.filter { section in !segments.contains { $0.sectionId == section.id } }
  }

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      HStack {
        FieldLabel("Sections")
        Spacer()
        if !segments.isEmpty {
          Button("Whole piece") { send([]) }
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(minHeight: 44)
            .accessibilityIdentifier("entrySettings.wholePiece")
        }
      }
      if segments.isEmpty {
        Text("Whole piece")
          .font(IntradaFont.body).foregroundStyle(IntradaColor.ink)
          .padding(.vertical, IntradaSpacing.controlGap)
      }
      ForEach(Array(segments.enumerated()), id: \.element.sectionId) { index, segment in
        segmentRow(segment, at: index)
        if index < segments.count - 1 || !unplanned.isEmpty {
          HairlineDivider()
        }
      }
      if !unplanned.isEmpty {
        Menu {
          ForEach(unplanned, id: \.id) { section in
            Button(section.label) { add(section.id) }
          }
        } label: {
          Label("Add a section", systemImage: "plus")
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.accent)
            .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
            .contentShape(Rectangle())
        }
        .accessibilityIdentifier("entrySettings.addSection")
      }
    }
    .fieldCardSurface()
  }

  private func segmentRow(_ segment: SegmentView, at index: Int) -> some View {
    let caption = sections.first { $0.id == segment.sectionId }?.barsCaption
    return HStack(spacing: IntradaSpacing.cardCompact) {
      VStack(alignment: .leading, spacing: 2) {
        Text(segment.label).font(IntradaFont.bodyMedium).foregroundStyle(IntradaColor.ink)
        if let caption {
          Text(caption).font(IntradaFont.small).foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      Spacer(minLength: IntradaSpacing.controlGap)
      if hasPlannedTime, segments.count > 1 {
        Text(segment.plannedDisplay)
          .font(IntradaFont.figure).foregroundStyle(IntradaColor.ink)
        Stepper(
          "Minutes on \(segment.label)",
          onIncrement: segment.canAddMinute ? { step(segment.sectionId, 1) } : nil,
          onDecrement: segment.canTakeMinute ? { step(segment.sectionId, -1) } : nil
        )
        .labelsHidden()
        .accessibilityValue(segment.plannedDisplay)
        .accessibilityIdentifier("entrySettings.segmentMinutes")
      }
      Button {
        remove(index)
      } label: {
        Image(systemName: "xmark")
          .iconSize(.badge, weight: .semibold)
          .foregroundStyle(IntradaColor.inkFaintIcon)
          .frame(width: 44, height: 44)
          .contentShape(Rectangle())
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Remove \(segment.label)")
    }
    .padding(.vertical, IntradaSpacing.controlGap)
  }

  private func add(_ sectionId: String) {
    send(Self.evenly(segments.map(\.sectionId) + [sectionId]))
  }

  private func remove(_ index: Int) {
    var ids = segments.map(\.sectionId)
    ids.remove(at: index)
    send(Self.evenly(ids))
  }

  static func evenly(_ sectionIds: [String]) -> [Segment] {
    sectionIds.map { Segment(sectionId: $0, plannedSecs: 0) }
  }
}
