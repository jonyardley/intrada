import SharedTypes
import SwiftUI

/// The week picker — a Mon–Sun row that *selects a day*; the screen renders that
/// day's sessions below. A day is always selected (the screen auto-selects on
/// open). Today is marked with a ring, the selected day with a fill, practised
/// days carry a dot, and not-yet days dim.
struct WeekStrip: View {
  let days: [PracticeDayView]
  @Binding var selected: String

  var body: some View {
    HStack(spacing: 4) {
      ForEach(days, id: \.date) { day in
        WeekDayCell(day: day, isSelected: day.date == selected) { selected = day.date }
      }
    }
    .background(
      GeometryReader { geo in
        Color.clear.preference(key: WeekStripHeightKey.self, value: geo.size.height)
      }
    )
  }
}

/// Reports a cell's natural height so the strip can size itself (#1730).
struct WeekStripHeightKey: PreferenceKey {
  static let defaultValue: CGFloat = 0
  static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
    value = max(value, nextValue())
  }
}

private struct WeekDayCell: View {
  let day: PracticeDayView
  let isSelected: Bool
  let onTap: () -> Void
  // Scales with Dynamic Type, capped so seven days still fit the screen (#1730).
  @ScaledMetric(relativeTo: .caption) private var dayCircleDiameter: CGFloat = 32
  private var cappedDayCircleDiameter: CGFloat { min(dayCircleDiameter, 36) }

  private var hasPractice: Bool { !day.sessionIds.isEmpty }

  var body: some View {
    Button(action: onTap) {
      VStack(spacing: 5) {
        Text(day.weekdayInitial)
          .font(IntradaFont.micro)
          .fontWeight(isSelected || day.isToday ? .semibold : .regular)
          .foregroundStyle(
            isSelected || day.isToday ? IntradaColor.accent : IntradaColor.inkSecondary)
        Text(String(day.dayNumber))
          .font(IntradaFont.metaMedium)
          .foregroundStyle(dayNumberColor)
          .lineLimit(1)
          .minimumScaleFactor(0.6)
          .frame(width: cappedDayCircleDiameter, height: cappedDayCircleDiameter)
          .background(isSelected ? IntradaColor.accent : .clear, in: Circle())
          .overlay(
            Circle().strokeBorder(
              IntradaColor.accent, lineWidth: day.isToday && !isSelected ? 1.5 : 0)
          )
        Circle()
          .fill(hasPractice ? IntradaColor.accent : .clear)
          .frame(width: 5, height: 5)
      }
      .frame(maxWidth: .infinity)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel(accessibilityLabel)
    .accessibilityAddTraits(isSelected ? [.isSelected] : [])
  }

  private var dayNumberColor: Color {
    if isSelected { return IntradaColor.onAccent }
    return day.isFuture ? IntradaColor.futureDay : IntradaColor.ink
  }

  private var accessibilityLabel: String {
    let label = day.isToday ? "Today, \(day.fullDate)" : day.fullDate
    return label + (hasPractice ? ", practised" : ", no practice")
  }
}

#if DEBUG
  #Preview {
    struct Harness: View {
      @State private var selected = PracticeWeekView.previewWeek.days[5].date
      var body: some View {
        ZStack {
          PaperBackground()
          WeekStrip(days: PracticeWeekView.previewWeek.days, selected: $selected)
            .padding(IntradaSpacing.card)
        }
      }
    }
    return Harness()
  }
#endif
