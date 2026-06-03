import SharedTypes
import SwiftUI

/// A past-practice row on the Practice home. Sessions span item types, so
/// there's no type-coded left bar (unlike the single-type library rows).
struct SessionCard: View {
  let session: PracticeSessionView
  @Environment(\.locale) private var locale
  @Environment(\.calendar) private var calendar

  var body: some View {
    VStack(alignment: .leading, spacing: 3) {
      Text(dateDisplay)
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.ink)
      Text(metaLine)
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.inkFaint)
      if session.completionStatus == .endedEarly {
        Text("Ended early")
          .font(IntradaFont.micro)
          .foregroundStyle(IntradaColor.inkFaint)
          .padding(.top, 2)
      }
    }
    .padding(.vertical, IntradaSpacing.row)
    .padding(.horizontal, IntradaSpacing.card)
    .frame(maxWidth: .infinity, alignment: .leading)
    .background(IntradaColor.cardFill)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .stroke(IntradaColor.hairline, lineWidth: 1)
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel(accessibilityLabel)
  }

  private var metaLine: String {
    "\(session.totalDurationSummary) · \(itemCount)"
  }

  private var itemCount: String {
    let count = session.entries.count
    let noun: String
    if session.entries.allSatisfy({ $0.itemType == .piece }) {
      noun = count == 1 ? "piece" : "pieces"
    } else if session.entries.allSatisfy({ $0.itemType == .exercise }) {
      noun = count == 1 ? "exercise" : "exercises"
    } else {
      noun = count == 1 ? "item" : "items"
    }
    return "\(count) \(noun)"
  }

  private var dateDisplay: String {
    guard let date = session.startedDate else { return "" }
    if calendar.isDateInToday(date) { return "Today" }
    if calendar.isDateInYesterday(date) { return "Yesterday" }
    let formatter = DateFormatter()
    // Drive locale + calendar off the SwiftUI environment (not `Locale.current`/
    // `Calendar.current`) so production follows the device while snapshot hosts
    // pin both — the template reorders by region ("Sat 30 May" vs "Sat, May 30")
    // and the day bucket shifts by timezone.
    formatter.locale = locale
    formatter.calendar = calendar
    formatter.timeZone = calendar.timeZone
    formatter.setLocalizedDateFormatFromTemplate("EEEdMMM")
    return formatter.string(from: date)
  }

  private var accessibilityLabel: String {
    var parts = [dateDisplay, session.totalDurationSummary, itemCount]
    if session.completionStatus == .endedEarly { parts.append("ended early") }
    return parts.joined(separator: ", ")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.row) {
        SessionCard(session: .previewCompleted)
        SessionCard(session: .previewEndedEarly)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
