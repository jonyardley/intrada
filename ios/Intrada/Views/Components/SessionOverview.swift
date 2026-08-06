import SharedTypes
import SwiftUI

/// Today's whole session at a glance, under the press-start hero: the blocks in
/// order, then what the plan could not take. Orientation only — the rows are not
/// tappable and offer no per-block choice, because a decision point before play
/// is a stall (#1225).
struct SessionOverview: View {
  let blocks: [PlannedBlockView]
  /// The plan's own words for what today could not take. Rendered as it stands;
  /// dropping it silently is a defect (spec §5 stage 5).
  let deferred: [String]

  /// One block needs no list — the press-start hero already is it.
  static func hasContent(blocks: [PlannedBlockView], deferred: [String]) -> Bool {
    blocks.count > 1 || !deferred.isEmpty
  }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.section) {
      if blocks.count > 1 { blockList }
      if !deferred.isEmpty { deferredList }
    }
  }

  private var blockList: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Eyebrow("Today's blocks")
      VStack(spacing: 0) {
        ForEach(Array(blocks.enumerated()), id: \.offset) { index, block in
          row(block, position: index)
          if index < blocks.count - 1 { HairlineDivider() }
        }
      }
      .padding(.horizontal, IntradaSpacing.card)
      .cardSurface()
    }
  }

  private var deferredList: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Eyebrow("Queued for another day")
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        ForEach(deferred, id: \.self) { line in
          Text(line)
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.inkSecondary)
            .fixedSize(horizontal: false, vertical: true)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(IntradaSpacing.card)
      .cardSurface()
    }
  }

  private func row(_ block: PlannedBlockView, position: Int) -> some View {
    HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.cardCompact) {
      // Glyph as well as tint: colour is an accelerator, never the only signal (T3).
      Image(systemName: block.kind.iconName)
        .font(.system(size: 12))
        .foregroundStyle(block.kind.accent)
        .frame(width: 16, alignment: .leading)
      VStack(alignment: .leading, spacing: 2) {
        Text(block.drillTitle)
          .font(IntradaFont.cardTitle(15))
          .foregroundStyle(IntradaColor.ink)
          .fixedSize(horizontal: false, vertical: true)
        if let section = block.section {
          Text(section)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
            .fixedSize(horizontal: false, vertical: true)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      Text("\(block.minutes) min")
        .font(IntradaFont.metaMedium)
        .monospacedDigit()
        .foregroundStyle(IntradaColor.inkSecondary)
        .layoutPriority(1)
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(spoken(block, position: position))
  }

  private func spoken(_ block: PlannedBlockView, position: Int) -> String {
    var parts = [
      "Block \(position + 1) of \(blocks.count)", block.kind.label, block.drillTitle,
    ]
    if let section = block.section { parts.append(section) }
    parts.append("\(block.minutes) minutes")
    return parts.joined(separator: ", ")
  }
}

#if DEBUG
  extension PlanView {
    static var preview: PlanView {
      PlanView(
        blocks: [
          PlannedBlockView(
            drillTitle: "Rootless voicings", section: "A section", kind: .exercise, minutes: 8,
            why: "Shells and rootless are what sit between you and improvising over Strasbourg."),
          PlannedBlockView(
            drillTitle: "Strasbourg / St. Denis", section: "A section", kind: .piece, minutes: 10,
            why: "The tune you are building towards."),
          PlannedBlockView(
            drillTitle: "Scales · D flat major", section: nil, kind: .exercise, minutes: 6,
            why: "D flat is the key the head sits in."),
          PlannedBlockView(
            drillTitle: "Clair de Lune", section: "Opening", kind: .piece, minutes: 6,
            why: "Kept warm while the jazz work leads."),
        ],
        totalMinutes: 30,
        deferred: [
          "Chopin Op. 10 No. 4 needs a longer run than today has.",
          "Left-hand octaves are queued behind the voicings work.",
        ])
    }
  }

  #Preview {
    ZStack {
      PaperBackground()
      SessionOverview(blocks: PlanView.preview.blocks, deferred: PlanView.preview.deferred)
        .padding(IntradaSpacing.card)
    }
  }
#endif
