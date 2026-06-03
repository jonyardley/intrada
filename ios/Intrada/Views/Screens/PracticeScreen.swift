import SharedTypes
import SwiftUI

struct PracticeScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.calendar) private var calendar
  @Environment(\.locale) private var locale

  // Injected so the week + auto-selection are deterministic in snapshots;
  // production uses "now".
  private let referenceDate: Date
  @State private var selectedDay: Date?

  init(referenceDate: Date = Date()) {
    self.referenceDate = referenceDate
  }

  #if DEBUG
    /// Snapshot seed: open on a specific day (e.g. a quiet one) without a tap.
    init(referenceDate: Date, selectedDay: Date) {
      self.referenceDate = referenceDate
      _selectedDay = State(initialValue: selectedDay)
    }
  #endif

  private var sessions: [PracticeSessionView] { store.viewModel?.sessions ?? [] }
  private var week: [Date] { PracticeWeek.days(containing: referenceDate, calendar: calendar) }
  private var practiceDays: Swift.Set<Date> {
    PracticeWeek.practiceDays(from: sessions, calendar: calendar)
  }
  private var effectiveSelection: Date {
    selectedDay
      ?? PracticeWeek.autoSelectedDay(
        in: week, today: referenceDate, practiceDays: practiceDays, calendar: calendar)
  }
  private var daySessions: [PracticeSessionView] {
    PracticeWeek.sessions(on: effectiveSelection, from: sessions, calendar: calendar)
  }

  var body: some View {
    ScreenScaffold(title: "Practice", subtitle: subtitle) {
      VStack(spacing: 0) {
        startButton
          .padding(.horizontal, 16)
          .padding(.top, 16)
        content
      }
    }
  }

  // The front door. The builder/player don't exist yet, so it's present-but-
  // disabled to establish the one-primary-action hierarchy without a dead-end.
  private var startButton: some View {
    VStack(spacing: 6) {
      Label("Start practising", systemImage: "play.fill")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 14)
        .background(LinearGradient.brandBar)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .opacity(0.5)
      Text("Coming soon")
        .font(IntradaFont.micro)
        .foregroundStyle(IntradaColor.inkFaint)
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Start practising, coming soon")
  }

  @ViewBuilder private var content: some View {
    if sessions.isEmpty {
      PlaceholderContent(
        systemImage: "metronome.fill",
        message: "Your practice sessions will appear here.")
    } else {
      WeekStrip(
        days: week, today: referenceDate, practiceDays: practiceDays,
        selected: Binding(get: { effectiveSelection }, set: { selectedDay = $0 }),
        calendar: calendar
      )
      .padding(.horizontal, 12)
      .padding(.top, 16)
      Text(dayLabel)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 16)
        .padding(.top, 16)
        .padding(.bottom, 6)
      dayContent
    }
  }

  @ViewBuilder private var dayContent: some View {
    if daySessions.isEmpty {
      PlaceholderContent(
        systemImage: "metronome.fill",
        message: "No practice on this day.")
    } else {
      ScrollView {
        LazyVStack(spacing: 14) {
          ForEach(daySessions, id: \.id) { session in
            SessionCard(session: session)
          }
        }
        .padding(.horizontal, 16)
        .padding(.bottom, 16)
      }
      .scrollEdgeShadow()
    }
  }

  private var dayLabel: String {
    if calendar.isDateInToday(effectiveSelection) { return "Today" }
    if calendar.isDateInYesterday(effectiveSelection) { return "Yesterday" }
    let formatter = DateFormatter()
    formatter.calendar = calendar
    formatter.locale = locale  // env locale, not Locale.current (see SessionCard)
    formatter.setLocalizedDateFormatFromTemplate("EEEEdMMMM")
    return formatter.string(from: effectiveSelection)
  }

  private var subtitle: String {
    let count = sessions.count
    return count == 0 ? "No sessions yet" : "\(count) session\(count == 1 ? "" : "s")"
  }
}

#if DEBUG
  #Preview("Populated") {
    PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate)
      .environment(Store.previewPractice)
  }

  #Preview("Empty") {
    PracticeScreen()
      .environment(Store.preview)
  }
#endif
