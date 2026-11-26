import SharedTypes
import SwiftUI

struct PracticeScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.marker) private var marker
  @Environment(\.heroGradient) private var heroGradient
  @Environment(\.openLibrary) private var openLibrary

  // Injected so the recovery card's date is deterministic in snapshots;
  // production uses "now".
  private let referenceDate: Date
  /// The tapped day's `date` key; nil lets the week open on its own day.
  @State private var selectedDay: String?
  @State private var weekIndexOverride: Int?
  @State private var openSessionId: String?
  // iPad split: a tapped card selects into the detail column instead of pushing.
  private var selection: Binding<String?>?
  @State private var showingProfile = false
  @State private var addingItem = false
  @State private var keysItemId: String?
  // Measured from a cell, not hard-coded (#1730).
  @State private var weekStripHeight: CGFloat = 64

  init(referenceDate: Date = Date(), selection: Binding<String?>? = nil) {
    self.referenceDate = referenceDate
    self.selection = selection
  }

  #if DEBUG
    /// Snapshot seed: open on a specific day (e.g. a quiet one) without a tap.
    init(referenceDate: Date, selectedDay: String) {
      self.referenceDate = referenceDate
      _selectedDay = State(initialValue: selectedDay)
    }
  #endif

  private var sessions: [PracticeSessionView] { store.sessionHistory }
  private var weeks: [PracticeWeekView] { store.practiceWeeks }
  // Defaults to the last (current) week; a swipe overrides it.
  private var effectiveWeekIndex: Int {
    max(0, min(weekIndexOverride ?? (weeks.count - 1), weeks.count - 1))
  }
  private var selectedWeek: PracticeWeekView? {
    weeks.indices.contains(effectiveWeekIndex) ? weeks[effectiveWeekIndex] : nil
  }
  private var effectiveSelection: PracticeDayView? {
    guard let week = selectedWeek else { return nil }
    if let tapped = week.days.first(where: { $0.date == selectedDay }) { return tapped }
    let opening = Int(week.openingDay)
    return week.days.indices.contains(opening) ? week.days[opening] : week.days.last
  }
  private var daySessions: [PracticeSessionView] {
    let ids = effectiveSelection?.sessionIds ?? []
    return ids.compactMap { id in sessions.first { $0.id == id } }
  }

  var body: some View {
    ScreenScaffold(title: "Practice", subtitle: subtitle, trailingPlacement: .header) {
      Button {
        showingProfile = true
      } label: {
        ProfileBadge(icon: store.viewModel?.profile.icon ?? .other, size: IntradaGlyph.bar)
          .frame(width: 44, height: 44)
          .contentShape(Circle())
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Profile")
      .accessibilityIdentifier("practice.profile")
    } content: {
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
          heroSection
            .fadeUp(0)
          if let firstRun = store.viewModel?.firstRun, firstRun.showsStartHere {
            StartHereCard(
              progress: firstRun,
              onAdd: { addingItem = true },
              onStartSession: { plan == nil ? store.send(.session(.startBuilding)) : editPlan() },
              onPlay: { plan == nil ? store.send(.session(.startBuilding)) : startPlan() }
            )
            .fadeUp(0)
          }
          thisWeek
            .fadeUp(1)
          selectedDaySection
            .fadeUp(2)
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .padding(.bottom, IntradaSpacing.section)
      }
      .scrollEdgeShadow()
      .safeAreaInset(edge: .bottom, spacing: 0) { startBar }
    }
    .reservingSubtitleLine(selection != nil)
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
    // Item-driven, not a NavigationLink: a link renders its label as a button,
    // and even with .buttonStyle(.plain) SwiftUI tints the whole screen a shade
    // lighter, which four Practice snapshots caught (#1371). The card must look
    // the same whether or not it happens to be tappable.
    .navigationDestination(item: $openSessionId) { id in
      if let found = sessions.first(where: { $0.id == id }) {
        PracticeSessionDetailScreen(session: found)
      }
    }
    .navigationDestination(isPresented: $showingProfile) {
      ProfileScreen()
    }
    .sheet(isPresented: $addingItem) {
      LibraryAddScreen().environment(store)
    }
    .sheet(isPresented: keysSheetShown) {
      if let keysItemId { TodaysKeysSheet(itemId: keysItemId).environment(store) }
    }
  }

  private var keysSheetShown: Binding<Bool> {
    Binding(get: { keysItemId != nil }, set: { if !$0 { keysItemId = nil } })
  }

  private var buildingBinding: Binding<Bool> {
    Binding(
      get: { store.viewModel?.buildingSetlist != nil },
      set: { presented in
        if !presented { store.send(.session(.cancelBuilding)) }
      })
  }

  // MARK: - (0) The plan from the time the musician has

  private var lastPractised: LastPractisedView? { store.viewModel?.lastPractised }
  private var plan: PlanView? { store.viewModel?.suggestedPlan }

  // An empty library has no plan; the plain hero still opens the builder.
  @ViewBuilder private var heroSection: some View {
    if let plan {
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        lengthSection(plan)
        warmUpSection(plan)
        sessionSection(plan)
      }
      .frame(maxWidth: .infinity, alignment: .leading)
    } else {
      hero
    }
  }

  private func lengthSection(_ plan: PlanView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionTitle("How long have you got?")
      lengthStepper(plan)
        .frame(maxWidth: .infinity)
        .padding(IntradaSpacing.card)
        .cardSurface()
    }
  }

  @ViewBuilder private func lengthStepper(_ plan: PlanView) -> some View {
    if let limits = store.viewModel?.limits {
      let minutes = Int(plan.lengthMins)
      ScoreStepper(
        score: minutes, caption: .unit("minutes"),
        range: Int(limits.sessionLengthMinMins)...Int(limits.sessionLengthMaxMins),
        step: Int(limits.sessionLengthStepMins), figure: IntradaFont.timer(32), typing: true,
        accessibilityLabel: "Session length", accessibilityValue: minutesPhrase(minutes)
      ) { next in
        store.send(.session(.setPlanLength(lengthMins: UInt16(clamping: next))))
      }
      .accessibilityIdentifier("practice.length.value")
    }
  }

  @ViewBuilder private func warmUpSection(_ plan: PlanView) -> some View {
    if plan.warmUpEnabled {
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        SectionTitle("Warm-up")
        if let row = plan.warmUp {
          planRow(row).cardSurface()
        } else {
          emptyWarmUpCard
        }
      }
    }
  }

  private var emptyWarmUpCard: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.xSmall) {
      Text("No warm-up items yet")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.ink)
      Text("Turn on \u{201C}Include in warm-up?\u{201D} for a piece or exercise in the Library.")
        .font(IntradaFont.secondary)
        .foregroundStyle(IntradaColor.inkSecondary)
        .fixedSize(horizontal: false, vertical: true)
      Button("Choose warm-up items") { openLibrary.callAsFunction() }
        .font(IntradaFont.button)
        .foregroundStyle(IntradaColor.accent)
        .padding(.top, IntradaSpacing.controlGap)
        .accessibilityIdentifier("practice.plan.chooseWarmUp")
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(IntradaSpacing.card)
    .cardSurface()
  }

  private func sessionSection(_ plan: PlanView) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(
        title: "Session",
        action: .init(title: "Change", identifier: "practice.plan.change", perform: editPlan))
      if !plan.rows.isEmpty {
        VStack(spacing: 0) {
          ForEach(Array(plan.rows.enumerated()), id: \.element.itemId) { index, row in
            if index > 0 { HairlineDivider().padding(.leading, IntradaSpacing.card) }
            planRow(row)
          }
        }
        .cardSurface()
      }
      AddRowButton(title: "Add piece or exercise", style: .plain, action: editPlan)
    }
  }

  private func planRow(_ row: PlanRowView) -> some View {
    let ticked = row.keys.filter(\.ticked).map(\.label)
    return VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
      HStack(spacing: IntradaSpacing.cardCompact) {
        row.kind.bar.frame(width: Self.barWidth, height: 34).clipShape(Capsule())
        VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
          Text(row.title)
            .font(IntradaFont.cardTitle)
            .foregroundStyle(IntradaColor.ink)
            .fixedSize(horizontal: false, vertical: true)
          Text(row.reason)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        Text("\(row.minutes) min")
          .font(IntradaFont.secondary)
          .monospacedDigit()
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .accessibilityElement(children: .ignore)
      .accessibilityLabel(
        [
          row.title, row.reason.replacingOccurrences(of: " · ", with: ", "),
          minutesPhrase(row.minutes),
        ]
        .joined(separator: ", "))
      if !ticked.isEmpty {
        keysButton(row, ticked: ticked)
          .padding(.leading, Self.barWidth + IntradaSpacing.cardCompact)
      }
    }
    .padding(IntradaSpacing.cardCompact)
  }

  private static let barWidth: CGFloat = 4

  private func keysButton(_ row: PlanRowView, ticked: [String]) -> some View {
    Button {
      keysItemId = row.itemId
    } label: {
      FlowLayout(spacing: IntradaSpacing.xSmall) {
        ForEach(ticked, id: \.self) { TagChip($0) }
      }
      .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Today's keys for \(row.title): \(ticked.joined(separator: ", "))")
    .accessibilityHint("Changes today's keys for \(row.title)")
    .accessibilityIdentifier("practice.plan.keys")
  }

  private func minutesPhrase<N: BinaryInteger>(_ minutes: N) -> String {
    "\(minutes) \(minutes == 1 ? "minute" : "minutes")"
  }

  private func editPlan() {
    store.send(
      .session(.editPlannedSession(now: SessionClock.nowRFC3339())),
      onSuccess: .impact, when: { $0.buildingSetlist != nil })
  }

  private func startPlan() {
    store.send(
      .session(.startPlannedSession(now: SessionClock.nowRFC3339())),
      onSuccess: .impact, when: { $0.activeSession != nil })
  }

  @ViewBuilder private var startBar: some View {
    if let plan {
      BrandBarButton(action: startPlan) {
        Image(systemName: "play.fill")
        Text(plan.startTitle)
      }
      .accessibilityLabel("Start session")
      .accessibilityValue(minutesPhrase(plan.totalMins))
      .accessibilityIdentifier("practice.plan.start")
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.cardCompact)
      .padding(.bottom, IntradaSpacing.section)
      .background(IntradaColor.paperTop)
      .overlay(alignment: .top) { Rectangle().fill(IntradaColor.hairline).frame(height: 1) }
    }
  }

  private var hero: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      // One element carrying all three strings: the day sits below the button
      // visually, so read separately it would reach VoiceOver detached from
      // the piece it describes.
      VStack(spacing: IntradaSpacing.cardCompact) {
        SectionTitle(heroHeading, tint: IntradaColor.onAccent.opacity(IntradaOpacity.secondary))

        if let lastPractised {
          Text(lastPractised.itemTitle)
            .font(IntradaFont.title)
            .foregroundStyle(IntradaColor.paperTop)
            .multilineTextAlignment(.center)
            .lineLimit(2)
            .minimumScaleFactor(0.75)
        }
      }
      .accessibilityElement(children: .ignore)
      .accessibilityLabel(heroLabel)

      Button {
        store.send(.session(.startBuilding))
      } label: {
        Image(systemName: "play.fill")
          .iconSize(.hero)
          .foregroundStyle(IntradaColor.onMarker)
          .frame(width: 96, height: 96)
          .background(marker)
          .clipShape(Circle())
          .dropShadow(.heroButton)
      }
      .buttonStyle(PressRebound())
      .accessibilityLabel("Start practising")
      .accessibilityIdentifier("practice.start")
      .padding(.vertical, IntradaSpacing.controlGap)
    }
    .frame(maxWidth: .infinity)
    .padding(IntradaSpacing.section)
    .background(heroGradient)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .dropShadow(.hero)
  }

  // The day lives here now, not under the play button (#1725).
  private var heroHeading: String {
    guard let lastPractised else { return "First session" }
    return "Last practised · \(lastPractised.relativeDay)"
  }

  private var heroLabel: String {
    guard let lastPractised else { return heroHeading }
    return "\(lastPractised.label), \(lastPractised.itemTitle)"
  }

  // MARK: - (1) This week

  private var thisWeek: some View {
    let count = Int(selectedWeek?.practisedDays ?? 0)
    return VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(
        title: selectedWeek?.title ?? "This week",
        trailing: "\(count) day\(count == 1 ? "" : "s") practised")
      weekStrips
    }
  }

  // Under UI tests the paging TabView's animation never lets the app idle, so
  // XCUITest stalls (#941) — show the current week statically instead.
  @ViewBuilder private var weekStrips: some View {
    Group {
      if UITestFlags.animationsDisabled {
        weekStripView(selectedWeek?.days ?? [])
      } else {
        TabView(selection: weekBinding) {
          ForEach(Array(weeks.enumerated()), id: \.offset) { index, week in
            weekStripView(week.days).tag(index)
          }
        }
        .tabViewStyle(.page(indexDisplayMode: .never))
      }
    }
    .frame(height: weekStripHeight)
    .onPreferenceChange(WeekStripHeightKey.self) { height in
      if height > 0 { weekStripHeight = height }
    }
  }

  private func weekStripView(_ days: [PracticeDayView]) -> some View {
    WeekStrip(
      days: days,
      selected: Binding(get: { effectiveSelection?.date ?? "" }, set: { selectedDay = $0 }))
  }

  // MARK: - (2) Selected day

  private var selectedDaySection: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: dayLabel, trailing: dayCountLabel)
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

  private var isFutureSelection: Bool { effectiveSelection?.isFuture ?? false }

  @ViewBuilder private var dayContent: some View {
    if daySessions.isEmpty {
      emptyDayCard
    } else {
      VStack(spacing: IntradaSpacing.cardCompact) {
        ForEach(daySessions, id: \.id) { session in
          SessionCard(session: session)
            .splitSelected(selection?.wrappedValue == session.id)
            .contentShape(Rectangle())
            .onTapGesture { open(session) }
            // The trait alone only relabels it; the action is what VoiceOver and
            // Switch Control actually invoke, since this is a gesture not a Button.
            .accessibilityAddTraits(.isButton)
            .accessibilityAction { open(session) }
            .accessibilityHint("Opens the session")
            .accessibilityIdentifier("practice.sessionCard")
            .swipeToDelete(label: "Delete session") { delete(session) }
        }
      }
    }
  }

  private func open(_ session: PracticeSessionView) {
    if let selection {
      selection.wrappedValue = session.id
    } else {
      openSessionId = session.id
    }
  }

  private func delete(_ session: PracticeSessionView) {
    store.send(.session(.deleteSaved(sessionId: session.id, now: SessionClock.nowRFC3339())))
  }

  private var emptyDayCard: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      Image(systemName: isFutureSelection ? "sunrise" : "moon")
        .iconSize(.large)
        .foregroundStyle(IntradaColor.inkSecondary)
      Text(isFutureSelection ? "Nothing logged yet" : "No practice logged")
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
          IntradaColor.divider,
          style: StrokeStyle(lineWidth: 1, dash: [5]))
    )
  }

  private var dayLabel: String { effectiveSelection?.heading ?? "" }

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

  // nil once there's a last-practised fact: the hero heading says it instead (#1725).
  private var subtitle: String? {
    let rawGreeting: String? = store.viewModel?.profile.greeting
    let greeting = rawGreeting.flatMap { $0.isEmpty ? nil : $0 }
    guard lastPractised != nil else {
      guard let greeting else { return "No sessions yet" }
      return "\(greeting) · No sessions yet"
    }
    return greeting
  }
}

#if DEBUG
  #Preview("Populated") {
    PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate)
      .environment(Store.previewPractice)
  }

  #Preview("Empty warm-up") {
    PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate)
      .environment(Store.previewPractice(plan: .previewEmptyWarmUp))
  }

  #Preview("Empty") {
    PracticeScreen()
      .environment(Store.preview)
  }
#endif
