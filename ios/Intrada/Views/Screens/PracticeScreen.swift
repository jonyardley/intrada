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
  // Presentation only: the core owns the session itself, and closes the loop by
  // clearing `coach.drill`, which `DrillLoopHost` reports back through onClose.
  @State private var drillLoopRunning = false
  @State private var composing = false

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
          if let recoverable = store.recoverableSession {
            RecoveryPromptCard(
              session: recoverable,
              referenceDate: referenceDate,
              onResume: { store.resumeRecoverableSession() },
              onDiscard: { store.discardSessionInProgress() }
            )
            .fadeUp(0)
          }
          if let built = store.viewModel?.built.session {
            // The composed session replaces the prescribed hero for as long as
            // it is today's steer — same scaffold, its own source line (A6).
            ComposedSessionScreen(session: built, onStart: { start(built: built.id) })
              .fadeUp(0)
          } else {
            if let steer = store.viewModel?.built.steer { steerCard(steer).fadeUp(0) }
            hero
              .fadeUp(0)
            steerLine
              .fadeUp(1)
            sessionOverview
              .fadeUp(2)
          }
          thisWeek
            .fadeUp(3)
          selectedDaySection
            .fadeUp(4)
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
    .fullScreenCover(isPresented: $drillLoopRunning) {
      DrillLoopHost(onClose: {
        drillLoopRunning = false
        planToday()
      })
    }
    .sheet(isPresented: $composing) {
      ComposeSheet()
    }
    .task { planToday() }
  }

  /// The core clears `plan` when the first block opens, so this asks for one
  /// only when there isn't one — on arrival, and again after a session ends.
  /// `availableMinutes: nil` takes the authored `[defaults]` length; the
  /// declaration surfaces that would ask are Phase 2b.
  private func planToday() {
    guard store.viewModel?.coach.plan == nil else { return }
    store.send(
      .coach(
        .planSession(
          now: SessionClock.nowRFC3339(), availableMinutes: nil,
          utcOffsetMinutes: SessionClock.utcOffsetMinutes())))
  }

  // MARK: - (0) One-tap hero

  private var plan: PlanView? { store.viewModel?.coach.plan }
  private var firstBlock: PlannedBlockView? { plan?.blocks.first }

  // No haptic on the tap: the loop is only really started once the core hands
  // back a block, and DrillLoopHost closes straight away if it can't.
  private var hero: some View {
    PressStartHero(
      headline: firstBlock?.drillTitle ?? "A focused session",
      section: firstBlock?.section,
      why: firstBlock?.why,
      footnote: planShape ?? "Tap to begin — one decision",
      onStart: { drillLoopRunning = true })
  }

  /// A1 — where composition lives (#1256). Directly under the hero, quiet
  /// weight, accent-coloured: reachable in one tap, never a footer secret and
  /// never a mode chooser (decision 11). Phrased as the user's own thought,
  /// because the built session is a steer on today, not an alternative app.
  private var steerLine: some View {
    Button {
      store.send(.builtSession(.openCompose), onSuccess: .selection)
      composing = true
    } label: {
      Label("I know what I want to practise today", systemImage: "text.badge.plus")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.accent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.cardCompact)
        .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
  }

  /// C3 — last night's words above the untouched hero (#1256). The shell does
  /// not offer it over a composed session (#1316): that list is already the
  /// user's own, and an accept the user could not see land is the #846 class.
  private func steerCard(_ steer: ProposedSteer) -> some View {
    ProposedSteerCard(
      steer: steer,
      onAccept: {
        store.send(
          .builtSession(.acceptProposedSteer(reflectionId: steer.reflectionId)),
          onSuccess: .impact)
      },
      onDecline: {
        store.send(
          .builtSession(.declineProposedSteer(reflectionId: steer.reflectionId)),
          onSuccess: .selection)
      })
  }

  /// Entering the drill loop is the core's call: it hands back a block, or it
  /// surfaces why it could not, and only then does the loop open.
  private func start(built id: String) {
    store.send(
      .builtSession(.startBuiltSession(sessionId: id, now: SessionClock.nowRFC3339())))
    if store.viewModel?.coach.drill != nil { drillLoopRunning = true }
  }

  private var planShape: String? {
    guard let plan, !plan.blocks.isEmpty else { return nil }
    let count = plan.blocks.count
    return "\(count) block\(count == 1 ? "" : "s") · about \(plan.totalMinutes) minutes"
  }

  // The whole session, not just the block the hero headlines, plus what the plan
  // could not take. The prose — deferred, titles, why — is the core's wording
  // verbatim; the shell only formats minutes and the spoken position.
  @ViewBuilder private var sessionOverview: some View {
    if let plan, SessionOverview.hasContent(blocks: plan.blocks, deferred: plan.deferred) {
      SessionOverview(blocks: plan.blocks, deferred: plan.deferred)
    }
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

  #Preview("Planned") {
    PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate)
      .environment(Store.previewPracticePlanned)
  }

  #Preview("Proposed steer") {
    PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate)
      .environment(Store.previewProposedSteer)
  }

  #Preview("Accepted steer") {
    PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate)
      .environment(Store.previewAcceptedSteer)
  }

  #Preview("Empty") {
    PracticeScreen()
      .environment(Store.preview)
  }
#endif
