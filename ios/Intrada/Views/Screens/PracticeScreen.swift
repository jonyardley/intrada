import SharedTypes
import SwiftUI

struct PracticeScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.calendar) private var calendar
  @Environment(\.locale) private var locale

  // Injected so the weeks + auto-selection are deterministic in snapshots;
  // production uses "now".
  private let referenceDate: Date
  @State private var selectedDay: Date?
  @State private var weekIndexOverride: Int?

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
  private var weeks: [[Date]] {
    PracticeWeek.weeks(forSessions: sessions, referenceDate: referenceDate, calendar: calendar)
  }
  private var practiceDays: Swift.Set<Date> {
    PracticeWeek.practiceDays(from: sessions, calendar: calendar)
  }
  // Defaults to the last (current) week; a swipe overrides it.
  private var effectiveWeekIndex: Int {
    min(weekIndexOverride ?? (weeks.count - 1), weeks.count - 1)
  }
  private var selectedWeek: [Date] { weeks[effectiveWeekIndex] }
  private var effectiveSelection: Date {
    selectedDay
      ?? PracticeWeek.selectedDay(
        forWeek: selectedWeek, today: referenceDate, practiceDays: practiceDays, calendar: calendar)
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
    // Drop a now-out-of-range pinned week so a later data change can't jump the
    // view to a stale page; reads are already clamped, this resets the store.
    .onChange(of: weeks.count) { _, newCount in
      if let pinned = weekIndexOverride, pinned >= newCount { weekIndexOverride = nil }
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
      TabView(selection: weekBinding) {
        ForEach(Array(weeks.enumerated()), id: \.offset) { index, days in
          WeekStrip(
            days: days, today: referenceDate, practiceDays: practiceDays,
            selected: Binding(get: { effectiveSelection }, set: { selectedDay = $0 }),
            calendar: calendar
          )
          .padding(.horizontal, 12)
          .tag(index)
        }
      }
      .tabViewStyle(.page(indexDisplayMode: .never))
      .frame(height: 64)
      .padding(.top, 14)
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

  // Swiping to another week clears the day selection so that week auto-selects
  // its own day (most recent practice, or its last day).
  private var weekBinding: Binding<Int> {
    Binding(
      get: { effectiveWeekIndex },
      set: { newIndex in
        weekIndexOverride = newIndex
        selectedDay = nil
      })
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
