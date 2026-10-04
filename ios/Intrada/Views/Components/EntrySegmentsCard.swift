import SharedTypes
import SwiftUI

/// Splits an entry into its sections, each with its own minutes (#2315). The
/// core keeps the minutes summing to the item's planned time: a segment sent
/// at zero shares what the others leave.
struct EntrySegmentsCard: View {
  let segments: [SegmentView]
  let sections: [SectionView]
  let hasPlannedTime: Bool
  let minimumSecs: UInt32
  let send: ([Segment]) -> Void

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
            .frame(minHeight: 32)
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
        HairlineDivider()
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
          onIncrement: { step(index, by: 60) },
          onDecrement: { step(index, by: -60) }
        )
        .labelsHidden()
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

  private var planned: [Segment] {
    segments.map { Segment(sectionId: $0.sectionId, plannedSecs: $0.plannedSecs) }
  }

  private func add(_ sectionId: String) {
    send(Self.evenly(segments.map(\.sectionId) + [sectionId]))
  }

  private func remove(_ index: Int) {
    var ids = segments.map(\.sectionId)
    ids.remove(at: index)
    send(Self.evenly(ids))
  }

  private func step(_ index: Int, by delta: Int) {
    if let next = Self.stepped(planned, at: index, by: delta, minimum: minimumSecs) { send(next) }
  }

  static func evenly(_ sectionIds: [String]) -> [Segment] {
    sectionIds.map { Segment(sectionId: $0, plannedSecs: 0) }
  }

  /// Moves `delta` seconds onto one segment from its neighbour (the next, or
  /// the one before for the last), which is sent at zero so the core gives it
  /// what is left. Nil when the step would take the segment under `minimum`.
  static func stepped(_ segments: [Segment], at index: Int, by delta: Int, minimum: UInt32)
    -> [Segment]?
  {
    guard segments.count > 1, segments.indices.contains(index) else { return nil }
    let next = Int(segments[index].plannedSecs) + delta
    guard next >= Int(minimum) else { return nil }
    var result = segments
    result[index].plannedSecs = UInt32(next)
    result[index == segments.count - 1 ? index - 1 : index + 1].plannedSecs = 0
    return result
  }
}
