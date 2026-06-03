import SharedTypes
import SwiftUI

/// A past-practice row on the Practice home. Sessions span item types, so
/// there's no type-coded left bar (unlike the single-type library rows).
struct SessionCard: View {
  let session: PracticeSessionView
  @Environment(\.locale) private var locale

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
    .padding(.vertical, 14)
    .padding(.horizontal, 16)
    .frame(maxWidth: .infinity, alignment: .leading)
    .background(IntradaColor.cardFill)
    .clipShape(RoundedRectangle(cornerRadius: 12))
    .overlay(
      RoundedRectangle(cornerRadius: 12)
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
    guard let date = Self.parse(session.startedAt) else { return "" }
    let calendar = Calendar.current
    if calendar.isDateInToday(date) { return "Today" }
    if calendar.isDateInYesterday(date) { return "Yesterday" }
    let formatter = DateFormatter()
    // Drive the format off the SwiftUI environment locale (not `Locale.current`)
    // so production localizes per device while snapshot hosts can pin it — the
    // raw template reorders by region ("Sat 30 May" en-GB vs "Sat, May 30" en-US).
    formatter.locale = locale
    formatter.setLocalizedDateFormatFromTemplate("EEEdMMM")
    return formatter.string(from: date)
  }

  private var accessibilityLabel: String {
    var parts = [dateDisplay, session.totalDurationSummary, itemCount]
    if session.completionStatus == .endedEarly { parts.append("ended early") }
    return parts.joined(separator: ", ")
  }

  /// chrono's `to_rfc3339` emits fractional seconds; fall back to the plain
  /// form so either shape parses.
  private static func parse(_ value: String) -> Date? {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    if let date = formatter.date(from: value) { return date }
    formatter.formatOptions = [.withInternetDateTime]
    return formatter.date(from: value)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(spacing: 14) {
        SessionCard(session: .previewCompleted)
        SessionCard(session: .previewEndedEarly)
      }
      .padding(16)
    }
  }
#endif
