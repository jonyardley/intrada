import SharedTypes
import SwiftUI

/// Splits an entry into its sections, each with its own minutes (#2315). The
/// core keeps the minutes summing to the item's planned time, and adding or
/// removing a section keeps the others' tuning (#2398). A planned section
/// offers the drills linked to it (#2249).
struct EntrySegmentsCard: View {
  let segments: [SegmentView]
  let sections: [SectionView]
  let drills: [DrillOfferView]
  let hasPlannedTime: Bool
  let canAddSection: Bool
  let wholePiece: () -> Void
  let add: (_ sectionId: String) -> Void
  let remove: (_ sectionId: String) -> Void
  let step: (_ sectionId: String, _ minutes: Int8) -> Void
  let tickDrill: (DrillOfferView) -> Void

  private var unplanned: [SectionView] {
    sections.filter { section in !segments.contains { $0.sectionId == section.id } }
  }

  private var offersAdd: Bool { canAddSection && !unplanned.isEmpty }

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      HStack {
        FieldLabel("Sections")
        Spacer()
        if !segments.isEmpty {
          Button("Whole piece") { wholePiece() }
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
        segmentRow(segment)
        let offers = drills.filter { $0.sectionId == segment.sectionId }
        if !offers.isEmpty {
          drillGroup(segment.label, offers)
        }
        if index < segments.count - 1 || offersAdd {
          HairlineDivider()
        }
      }
      if offersAdd {
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

  private func segmentRow(_ segment: SegmentView) -> some View {
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
        remove(segment.sectionId)
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

  private func drillGroup(_ sectionLabel: String, _ offers: [DrillOfferView]) -> some View {
    VStack(alignment: .leading, spacing: 0) {
      FieldLabel("Drills for \(sectionLabel)")
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.cardCompact)
        .accessibilityAddTraits(.isHeader)
      ForEach(offers, id: \.exerciseId) { offer in
        let added = offer.addedEntryId != nil
        TickRow(label: offer.title, chosen: added, identifier: "entrySettings.drill") {
          tickDrill(offer)
        }
        .accessibilityValue(
          added ? "drill for \(sectionLabel), added" : "drill for \(sectionLabel)")
      }
    }
    .background(IntradaColor.paperTop)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.control))
    .padding(.bottom, IntradaSpacing.controlGap)
  }
}
