import SharedTypes
import SwiftUI

/// Today's keys for one plan row (#3008). Each tick goes to the core at once;
/// the sections stay fixed by what the core suggested, so a tick never moves a key.
struct TodaysKeysSheet: View {
  let itemId: String

  @Environment(Store.self) private var store
  @State private var formError: String?

  private var row: PlanRowView? {
    guard let plan = store.viewModel?.suggestedPlan else { return nil }
    if let warmUp = plan.warmUp, warmUp.itemId == itemId { return warmUp }
    return plan.rows.first { $0.itemId == itemId }
  }

  var body: some View {
    BottomSheet(title: "Today's keys", detents: [.large]) {
      ScrollView {
        if let row {
          content(row)
        } else {
          Text("This item is no longer in today's plan.")
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(IntradaSpacing.card)
        }
      }
    }
  }

  private func content(_ row: PlanRowView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      if let formError {
        FormErrorBanner(message: formError)
          .padding(.horizontal, IntradaSpacing.card)
      }
      VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
        Text("\(row.title), \(row.minutes) min")
          .font(IntradaFont.cardTitle)
          .foregroundStyle(IntradaColor.ink)
          .fixedSize(horizontal: false, vertical: true)
        Text(row.keysReason)
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
          .fixedSize(horizontal: false, vertical: true)
      }
      .accessibilityElement(children: .combine)
      .padding(.horizontal, IntradaSpacing.card)
      section("Suggested", row.keys.filter(\.suggested), in: row)
      section("Other keys", row.keys.filter { !$0.suggested }, in: row)
      Text(
        "Ticking more keys keeps \(row.minutes) minutes and gives each key less time."
      )
      .font(IntradaFont.small)
      .foregroundStyle(IntradaColor.inkSecondary)
      .fixedSize(horizontal: false, vertical: true)
      .padding(.horizontal, IntradaSpacing.card)
    }
    .padding(.vertical, IntradaSpacing.card)
  }

  @ViewBuilder
  private func section(_ title: String, _ keys: [PlanKeyView], in row: PlanRowView) -> some View {
    if !keys.isEmpty {
      SectionTitle(title).padding(.horizontal, IntradaSpacing.card)
      VStack(spacing: 0) {
        ForEach(Array(keys.enumerated()), id: \.element.label) { index, key in
          if index > 0 { HairlineDivider().padding(.leading, IntradaSpacing.card) }
          TickRow(
            label: key.label, caption: Self.caption(key), chosen: key.ticked,
            identifier: "todaysKeys.row", toggle: { toggle(key, in: row) })
        }
      }
      .cardSurface()
      .padding(.horizontal, IntradaSpacing.card)
    }
  }

  static func caption(_ key: PlanKeyView) -> String {
    guard let score = key.score else {
      return key.lastPlayed.prefix(1).uppercased() + key.lastPlayed.dropFirst()
    }
    return "Scored \(score), \(key.lastPlayed)"
  }

  private func toggle(_ tapped: PlanKeyView, in row: PlanRowView) {
    let keys = row.keys.filter { $0.key == tapped.key ? !$0.ticked : $0.ticked }.map(\.key)
    formError = nil
    let error = store.sendFromSheet(
      .session(.setTodaysKeys(itemId: row.itemId, keys: keys, now: SessionClock.nowRFC3339())))
    withAnimation { formError = error }
  }
}
