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
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.top, IntradaSpacing.card)
        content
      }
    }
    // Drop a now-out-of-range pinned week so a later data change can't jump the
    // view to a stale page; reads are already clamped, this resets the store.
    .onChange(of: weeks.count) { _, newCount in
      if let pinned = weekIndexOverride, pinned >= newCount { weekIndexOverride = nil }
    }
    // State-driven: `startBuilding` makes `buildingSetlist` non-nil → push; a
    // pop sends `cancelBuilding` → core returns to Idle. No local nav flag.
    .navigationDestination(isPresented: buildingBinding) {
      SessionBuilderScreen()
    }
  }

  private var buildingBinding: Binding<Bool> {
    Binding(
      get: { store.viewModel?.buildingSetlist != nil },
      set: { presented in
        if !presented { store.send(.session(.cancelBuilding)) }
      })
  }

  private var startButton: some View {
    Button {
      store.send(.session(.startBuilding))
    } label: {
      Label("Start practising", systemImage: "play.fill")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.row)
        .background(LinearGradient.brandBar)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Start practising")
  }

  @ViewBuilder private var content: some View {
    if sessions.isEmpty {
      PlaceholderContent(
        systemImage: "metronome.fill",
        message: "Your practice sessions will appear here.")
    } else {
      weekStrips
      Text(dayLabel)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .padding(.bottom, 6)
      dayContent
    }
  }

  // Under UI tests the paging TabView's animation never lets the app idle, so
  // XCUITest stalls (#941) — show the current week statically instead.
  @ViewBuilder private var weekStrips: some View {
    Group {
      if UITestFlags.animationsDisabled {
        weekStripView(selectedWeek)
      } else {
        TabView(selection: weekBinding) {
          ForEach(Array(weeks.enumerated()), id: \.offset) { index, days in
            weekStripView(days).tag(index)
          }
        }
        .tabViewStyle(.page(indexDisplayMode: .never))
      }
    }
    .frame(height: 64)
    .padding(.top, IntradaSpacing.row)
  }

  private func weekStripView(_ days: [Date]) -> some View {
    WeekStrip(
      days: days, today: referenceDate, practiceDays: practiceDays,
      selected: Binding(get: { effectiveSelection }, set: { selectedDay = $0 }),
      calendar: calendar
    )
    .padding(.horizontal, IntradaSpacing.cardCompact)
  }

  @ViewBuilder private var dayContent: some View {
    if daySessions.isEmpty {
      PlaceholderContent(
        systemImage: "metronome.fill",
        message: "No practice on this day.")
    } else {
      ScrollView {
        LazyVStack(spacing: IntradaSpacing.row) {
          ForEach(daySessions, id: \.id) { session in
            SessionCard(session: session)
          }
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .padding(.bottom, IntradaSpacing.card)
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
