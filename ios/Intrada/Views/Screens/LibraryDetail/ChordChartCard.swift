import SharedTypes
import SwiftUI

struct ChordChartCard: View {
  let item: LibraryItemView
  let onEdit: () -> Void

  var body: some View {
    VStack(spacing: 0) {
      SectionHeader(
        title: "Chord chart",
        action: .init(
          title: item.chordChart == nil ? "Add" : "Edit",
          accessibilityLabel: item.chordChart == nil ? "Add a chord chart" : "Edit chord chart",
          perform: onEdit)
      )
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.card)
      .padding(.bottom, item.chordChart == nil ? IntradaSpacing.card : IntradaSpacing.cardCompact)

      if let chart = item.chordChart {
        chartSubtitle(chart)
        chartBarGrid(chart).padding(.bottom, IntradaSpacing.controlGap)
      } else {
        SectionEmptyState("Paste the changes to keep them with the piece.")
      }
    }
    .cardSurface()
  }

  private func chartSubtitle(_ chart: ChordChart) -> some View {
    let bars = chart.sections.reduce(0) { $0 + $1.bars.count }
    let changes = chart.sections.reduce(0) { $0 + $1.bars.reduce(0) { $0 + $1.chords.count } }
    let key = item.keyDisplay ?? chart.key
    return Text("\(key) · \(bars) \(bars == 1 ? "bar" : "bars") · \(changes) changes")
      .font(IntradaFont.meta)
      .foregroundStyle(IntradaColor.inkSecondary)
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.bottom, IntradaSpacing.cardCompact)
  }

  private func chartBarGrid(_ chart: ChordChart) -> some View {
    let columns = Array(repeating: GridItem(.flexible(), spacing: 6), count: 4)
    return VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      ForEach(Array(chart.sections.enumerated()), id: \.offset) { _, section in
        if let label = section.label, !label.isEmpty {
          Eyebrow(label)
        }
        LazyVGrid(columns: columns, spacing: 6) {
          ForEach(Array(sectionChords(section).enumerated()), id: \.offset) { _, raw in
            Text(raw)
              .font(IntradaFont.cardTitle())
              .foregroundStyle(IntradaColor.ink)
              .lineLimit(1)
              .minimumScaleFactor(0.7)
              .frame(maxWidth: .infinity)
              .padding(.vertical, IntradaSpacing.controlGap)
              .padding(.horizontal, 4)
              .background(
                RoundedRectangle(cornerRadius: IntradaRadius.badge)
                  .fill(IntradaColor.paperTop)
                  .stroke(IntradaColor.divider, lineWidth: 1)
              )
          }
        }
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.bottom, IntradaSpacing.cardCompact)
    .accessibilityElement(children: .combine)
    .accessibilityLabel(
      "Chord chart: " + chart.sections.flatMap { sectionChords($0) }.joined(separator: ", "))
  }

  private func sectionChords(_ section: ChartSection) -> [String] {
    section.bars.flatMap { $0.chords.map { $0.symbol.raw } }
  }
}
