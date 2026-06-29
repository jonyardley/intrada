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
      ScrollView {
        VStack(spacing: IntradaSpacing.section) {
          hero
            .fadeUp(0)
          thisWeek
            .fadeUp(1)
          selectedDaySection
            .fadeUp(2)
          footerLink
            .fadeUp(3)
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .padding(.bottom, IntradaSpacing.section)
      }
      .scrollEdgeShadow()
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

  // MARK: - (0) One-tap hero

  private var hero: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      Eyebrow("Today", tint: IntradaColor.onAccent.opacity(0.7))

      Text("A focused session")
        .font(IntradaFont.pageTitle(25))
        .foregroundStyle(IntradaColor.paperTop)
        .multilineTextAlignment(.center)

      Button {
        store.send(.session(.startBuilding))
      } label: {
        Image(systemName: "play.fill")
          .font(.system(size: 38))
          .foregroundStyle(IntradaColor.accent)
          .frame(width: 96, height: 96)
          .background(IntradaColor.playerBgTop)
          .clipShape(Circle())
          .shadow(color: .black.opacity(0.25), radius: 16, y: 8)
      }
      .buttonStyle(PressRebound())
      .accessibilityLabel("Start practising")
      .padding(.vertical, IntradaSpacing.controlGap)

      Text("Tap to begin — one decision")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent.opacity(0.85))
        .multilineTextAlignment(.center)
    }
    .frame(maxWidth: .infinity)
    .padding(IntradaSpacing.section)
    .background(LinearGradient.practiceHero)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.hero))
    .shadow(color: .black.opacity(0.18), radius: 20, y: 10)
  }

  // MARK: - (1) This week

  private var thisWeek: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(
        title: "This week",
        trailing: "\(practiceDays.count) days practised")
      weekStrips
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
  }

  private func weekStripView(_ days: [Date]) -> some View {
    WeekStrip(
      days: days, today: referenceDate, practiceDays: practiceDays,
      selected: Binding(get: { effectiveSelection }, set: { selectedDay = $0 }),
      calendar: calendar
    )
  }

  // MARK: - (2) Selected day

  private var selectedDaySection: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      HStack(alignment: .firstTextBaseline) {
        Eyebrow(dayLabel)
        Spacer(minLength: IntradaSpacing.controlGap)
        Text(dayCountLabel)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      dayContent
    }
  }

  private var dayCountLabel: String {
    if !daySessions.isEmpty {
      let count = daySessions.count
      return "\(count) session\(count == 1 ? "" : "s")"
    }
    return isFutureSelection ? "Yet to come" : "Rest day"
  }

  private var isFutureSelection: Bool {
    calendar.startOfDay(for: effectiveSelection) > calendar.startOfDay(for: referenceDate)
  }

  @ViewBuilder private var dayContent: some View {
    if daySessions.isEmpty {
      emptyDayCard
    } else {
      VStack(spacing: IntradaSpacing.cardCompact) {
        ForEach(daySessions, id: \.id) { session in
          SessionCard(session: session)
        }
      }
    }
  }

  private var emptyDayCard: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      Image(systemName: isFutureSelection ? "sunrise" : "moon")
        .font(.system(size: 28))
        .foregroundStyle(IntradaColor.inkSecondary)
      Text(
        isFutureSelection
          ? "Nothing logged yet — the week's still young."
          : "A rest day. No pressure — your schedule has adapted."
      )
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.inkSecondary)
      .multilineTextAlignment(.center)
    }
    .frame(maxWidth: .infinity)
    .padding(IntradaSpacing.card)
    .background(IntradaColor.cardFill)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .strokeBorder(
          IntradaColor.slotOutline,
          style: StrokeStyle(lineWidth: 1, dash: [5]))
    )
  }

  // MARK: - (3) Footer link

  private var footerLink: some View {
    Button {
      store.send(.session(.startBuilding))
    } label: {
      Label("Build a custom session", systemImage: "slider.horizontal.3")
        .font(IntradaFont.bodyMedium.weight(.medium))
        .foregroundStyle(IntradaColor.inkSecondary)
    }
    .buttonStyle(.plain)
    .frame(maxWidth: .infinity)
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
