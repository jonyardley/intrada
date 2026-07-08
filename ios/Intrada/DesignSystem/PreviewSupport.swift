#if DEBUG
  import Foundation
  import IntradaCoreFFI
  import SharedTypes

  /// A fixed Gregorian/UTC calendar so date-derived UI (the week strip) renders
  /// identically on any host — pair it with `previewReferenceDate` in previews
  /// and pin it via `.environment(\.calendar, PreviewCalendar.utc)` in snapshot hosts.
  enum PreviewCalendar {
    static var utc: Calendar {
      var calendar = Calendar(identifier: .gregorian)
      calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
      return calendar
    }
  }

  /// Offline bridge for Xcode previews: serves the core's initial (empty)
  /// ViewModel — optionally seeded with library items — and emits no effects,
  /// so store-backed screens render in the canvas without FFI networking.
  final class PreviewBridge: CoreBridge {
    private let core = CoreFfi()
    private let items: [LibraryItemView]
    private let activeQuery: ListQuery?
    private let sessions: [PracticeSessionView]
    private let buildingSetlist: BuildingSetlistView?
    private let activeSession: ActiveSessionView?
    private let summary: SummaryView?

    private let analytics: AnalyticsView?

    init(
      items: [LibraryItemView] = [], activeQuery: ListQuery? = nil,
      sessions: [PracticeSessionView] = [], buildingSetlist: BuildingSetlistView? = nil,
      activeSession: ActiveSessionView? = nil, summary: SummaryView? = nil,
      analytics: AnalyticsView? = nil
    ) {
      self.items = items
      self.activeQuery = activeQuery
      self.sessions = sessions
      self.buildingSetlist = buildingSetlist
      self.activeSession = activeSession
      self.summary = summary
      self.analytics = analytics
    }

    func update(_ event: Event) throws -> [Request] { [] }
    func resolve(_ id: UInt32, httpResult: HttpResult) throws -> [Request] { [] }
    func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request] { [] }
    func resolveEmpty(_ id: UInt32) throws -> [Request] { [] }
    func view() throws -> ViewModel {
      var viewModel = try ViewModel.bincodeDeserialize(input: [UInt8](core.view()))
      viewModel.activeQuery = activeQuery
      let visible: [LibraryItemView]
      if let kind = activeQuery?.itemType {
        visible = items.filter { $0.itemType == kind }
      } else {
        visible = items
      }
      viewModel.items = visible
      // Type-filters items; callers pre-filter the list for text/tag queries.
      viewModel.visiblePieces = UInt64(visible.filter { $0.itemType == .piece }.count)
      viewModel.visibleExercises = UInt64(visible.filter { $0.itemType == .exercise }.count)
      viewModel.sessions = sessions
      viewModel.buildingSetlist = buildingSetlist
      viewModel.activeSession = activeSession
      viewModel.summary = summary
      if let analytics { viewModel.analytics = analytics }
      return viewModel
    }
  }

  extension Store {
    /// A deterministic, offline store for `#Preview` blocks.
    static var preview: Store { Store(bridge: PreviewBridge()) }

    /// An offline store with curated sample items (specific edge cases).
    /// Used by snapshot tests where the exact data must be deterministic.
    static var previewLibrary: Store {
      Store(bridge: PreviewBridge(items: [.previewPiece, .previewExercise, .previewMinimal]))
    }

    /// Pieces-filtered library for the filtered-state snapshot (#792).
    static var previewLibraryFiltered: Store {
      Store(
        bridge: PreviewBridge(
          items: [.previewPiece, .previewExercise, .previewMinimal],
          activeQuery: ListQuery(text: nil, itemType: .piece, key: nil, tags: [])))
    }

    /// Text-searched library for the revealed-search-bar snapshot: "clair"
    /// matches Clair de Lune. The bridge serves the already-matched subset.
    static var previewLibrarySearching: Store {
      Store(
        bridge: PreviewBridge(
          items: [.previewPiece],
          activeQuery: ListQuery(text: "clair", itemType: nil, key: nil, tags: [])))
    }

    /// A store driven by the *real* core seeded with the canonical demo dataset
    /// (`Event.loadSampleData` → `sample_items()`). Render-only, so it completes
    /// synchronously and offline. Use in screen previews: same data as the CI
    /// screenshot, and the filter pills actually work in the canvas.
    /// Not for snapshot tests — `sample_items()` stamps wall-clock timestamps.
    static var previewSeeded: Store {
      let store = Store()
      store.send(.loadSampleData)
      return store
    }

    /// Library with a populated Priorities section (2 starred, 1 not) for the
    /// pinned-section snapshot. Injected directly so priority + ids are stable.
    static var previewLibraryPriorities: Store {
      Store(
        bridge: PreviewBridge(items: [
          starred(.previewPiece), starred(.previewExercise), .previewMinimal,
        ]))
    }

    private static func starred(_ item: LibraryItemView) -> LibraryItemView {
      var copy = item
      copy.priority = true
      return copy
    }

    /// Practice home with deterministic sessions (fixed past dates) for the
    /// populated-state snapshot — covers both completed + ended-early cards.
    static var previewPractice: Store {
      Store(
        bridge: PreviewBridge(sessions: [
          .previewCompleted, .previewEndedEarly,
        ]))
    }

    /// Session builder mid-assembly: a non-empty setlist for the populated-state
    /// preview + snapshot. Injected directly (deterministic, offline) rather than
    /// driven through the core, whose ulids/timestamps aren't snapshot-stable.
    static var previewBuilding: Store {
      Store(
        bridge: PreviewBridge(
          items: [.previewPiece, .previewExercise, .previewMinimal],
          buildingSetlist: BuildingSetlistView(
            entries: [.previewPiece, .previewExercise],
            itemCount: 2,
            blocks: [
              SetlistBlockView(
                groupId: nil, pieceTitle: nil, relatedCount: 0, durationDisplay: "—",
                entries: [.previewPiece]),
              SetlistBlockView(
                groupId: nil, pieceTitle: nil, relatedCount: 0, durationDisplay: "—",
                entries: [.previewExercise]),
            ],
            blockCount: 2,
            totalDurationDisplay: nil, totalDurationSummary: nil,
            sessionIntention: nil, targetDurationMins: nil,
            sourceStatus: .noSource)))
    }

    /// Session builder with a block (a piece + 2 related) above a standalone
    /// item — the grouped-state preview + snapshot.
    static var previewBuildingGrouped: Store {
      let block: [SetlistEntryView] = [
        .previewGroupedScales, .previewGroupedArpeggios, .previewGroupedPiece,
      ]
      return Store(
        bridge: PreviewBridge(
          items: [.previewPiece, .previewExercise, .previewMinimal],
          buildingSetlist: BuildingSetlistView(
            entries: block + [.previewStandaloneExercise],
            itemCount: 4,
            blocks: [
              SetlistBlockView(
                groupId: "g1", pieceTitle: "Clair de Lune", relatedCount: 2,
                durationDisplay: "12 min", entries: block),
              SetlistBlockView(
                groupId: nil, pieceTitle: nil, relatedCount: 0, durationDisplay: "—",
                entries: [.previewStandaloneExercise]),
            ],
            blockCount: 2,
            totalDurationDisplay: "12m 0s", totalDurationSummary: "12 min",
            sessionIntention: nil, targetDurationMins: nil, sourceStatus: .noSource)))
    }

    /// Player Focus — a piece mid-session with a session intention and a time
    /// target (the target bar), no reps.
    static var previewActive: Store {
      Store(bridge: PreviewBridge(activeSession: .previewActive))
    }

    /// Player Focus — an exercise with an active rep counter.
    static var previewActiveReps: Store {
      Store(bridge: PreviewBridge(activeSession: .previewActiveReps))
    }

    /// Player Summary — a completed session with scored entries. Analytics are
    /// injected so the gold mastery toast (Clair de Lune 3 → 4) has data.
    static var previewSummary: Store {
      Store(bridge: PreviewBridge(summary: .previewSummary, analytics: .previewAnalytics))
    }

    /// Player Summary — ended early, so the unreached item shows not-attempted.
    static var previewSummaryEndedEarly: Store {
      Store(bridge: PreviewBridge(summary: .previewSummaryEndedEarly))
    }

    /// Progress — a populated analytics view (dial, consistency, recent mastery).
    static var previewProgress: Store {
      Store(bridge: PreviewBridge(analytics: .previewAnalytics))
    }

    /// Library where rows carry a mastery score, so the trailing meters fill.
    static var previewLibraryMastery: Store {
      Store(
        bridge: PreviewBridge(items: [
          scored(.previewPiece, 4), scored(.previewExercise, 3), .previewMinimal,
        ]))
    }

    /// Detail view: piece with 3 linked exercises (varied scores, one unrated).
    static var previewDetailLinkedPopulated: Store {
      Store(bridge: PreviewBridge(items: [.previewDetailWithLinkedExercises]))
    }

    /// Detail view: piece with no linked exercises — shows the empty state.
    static var previewDetailLinkedEmpty: Store {
      Store(bridge: PreviewBridge(items: [.previewDetailLinkedEmpty]))
    }

    /// Detail view: exercise related to 2 pieces — shows the "Related pieces" card.
    static var previewExerciseLinkedFrom: Store {
      Store(bridge: PreviewBridge(items: [.previewExerciseWithLinkedFrom]))
    }

    private static func scored(_ item: LibraryItemView, _ score: UInt8) -> LibraryItemView {
      var copy = item
      copy.practice = ItemPracticeSummary(
        sessionCount: 8, totalMinutes: 120, latestScore: score, scoreHistory: [],
        latestTempo: nil, tempoHistory: [], lastPracticedAt: "2026-05-30T09:00:00Z")
      return copy
    }
  }

  extension AnalyticsView {
    /// A deterministic analytics fixture for the Progress screen + snapshots.
    /// `scoreTrends` (all items' latest score) drives the dial mean (≈3.4);
    /// `scoreChanges` (this week's movers) drive the Recent-mastery rows; weekly
    /// `dailyTotals` roll up to the consistency bars (40/75/55/95/82).
    static var previewAnalytics: AnalyticsView {
      AnalyticsView(
        weeklySummary: WeeklySummary(
          totalMinutes: 380, sessionCount: 14, itemsCovered: 11,
          prevTotalMinutes: 300, prevSessionCount: 11, prevItemsCovered: 9,
          timeDirection: .up, sessionsDirection: .up, itemsDirection: .up,
          hasPrevWeekData: true),
        streak: PracticeStreak(currentDays: 4),
        dailyTotals: [
          DailyPracticeTotal(date: "2026-04-29", minutes: 40),
          DailyPracticeTotal(date: "2026-05-06", minutes: 75),
          DailyPracticeTotal(date: "2026-05-13", minutes: 55),
          DailyPracticeTotal(date: "2026-05-20", minutes: 95),
          DailyPracticeTotal(date: "2026-05-28", minutes: 82),
        ],
        topItems: [
          ItemRanking(
            itemId: "piece-1", itemTitle: "Clair de Lune", itemType: .piece,
            totalMinutes: 180, sessionCount: 9)
        ],
        scoreTrends: [
          scoreTrend("piece-1", "Clair de Lune", 4),
          scoreTrend("exercise-1", "Hanon No. 1", 4),
          scoreTrend("piece-2", "Gymnopédie No. 1", 3),
          scoreTrend("piece-3", "Nocturne Op. 9", 3),
          scoreTrend("exercise-2", "Major Scales", 3),
        ],
        neglectedItems: [],
        scoreChanges: [
          ScoreChange(
            itemId: "piece-1", itemTitle: "Clair de Lune", previousScore: 3,
            currentScore: 4, delta: 1, isNew: false),
          ScoreChange(
            itemId: "exercise-1", itemTitle: "Hanon No. 1", previousScore: 2,
            currentScore: 3, delta: 1, isNew: false),
          ScoreChange(
            itemId: "piece-2", itemTitle: "Gymnopédie No. 1", previousScore: nil,
            currentScore: 3, delta: 0, isNew: true),
        ])
    }

    private static func scoreTrend(_ id: String, _ title: String, _ latest: UInt8)
      -> ItemScoreTrend
    {
      ItemScoreTrend(
        itemId: id, itemTitle: title,
        scores: [ScorePoint(date: "2026-05-30", score: latest)], latestScore: latest)
    }
  }

  extension LibraryItemView {
    static var previewPiece: LibraryItemView {
      LibraryItemView(
        id: "piece-1", itemType: .piece, title: "Clair de Lune", subtitle: "Claude Debussy",
        key: "Db", modality: .major, tempo: "Andante (72 BPM)", tempoMarking: "Andante",
        tempoBpm: 72,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        latestAchievedTempo: nil, priority: false,
        linkedExercises: [
          LinkedExerciseView(
            id: "exercise-1", title: "Hanon No. 1", key: "C major", tempo: "♩ = 108",
            practice: nil),
          LinkedExerciseView(
            id: "exercise-2", title: "Db Major Scale", key: "Db major", tempo: nil, practice: nil),
        ],
        linkedFromPieces: [])
    }

    static var previewExercise: LibraryItemView {
      LibraryItemView(
        id: "exercise-1", itemType: .exercise, title: "Hanon No. 1",
        subtitle: "Charles-Louis Hanon",
        key: "C", modality: .major, tempo: "108 BPM", tempoMarking: nil, tempoBpm: 108,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: [])
    }

    static var previewDetail: LibraryItemView {
      LibraryItemView(
        id: "piece-3", itemType: .piece, title: "Clair de Lune", subtitle: "Claude Debussy",
        key: "Db", modality: .major, tempo: "Andante (72 BPM)", tempoMarking: "Andante",
        tempoBpm: 72,
        notes: "Focus on the rubato in the opening phrase; keep the left hand soft.",
        tags: ["recital", "impressionist", "memorised"], createdAt: "", updatedAt: "",
        practice: nil, latestAchievedTempo: nil, priority: false, linkedExercises: [],
        linkedFromPieces: [])
    }

    static var previewMinimal: LibraryItemView {
      LibraryItemView(
        id: "piece-2", itemType: .piece, title: "Prelude in C", subtitle: "",
        key: nil, modality: nil, tempo: nil, tempoMarking: nil, tempoBpm: nil,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: [])
    }

    /// A piece with a populated linked-exercises list (3 items, varied scores including
    /// one unrated) — for the linked-exercises section snapshots.
    static var previewDetailWithLinkedExercises: LibraryItemView {
      LibraryItemView(
        id: "piece-3", itemType: .piece, title: "Clair de Lune", subtitle: "Claude Debussy",
        key: "Db", modality: .major, tempo: "Andante (72 BPM)", tempoMarking: "Andante",
        tempoBpm: 72,
        notes: "Focus on the rubato in the opening phrase; keep the left hand soft.",
        tags: ["recital", "impressionist"], createdAt: "", updatedAt: "",
        practice: ItemPracticeSummary(
          sessionCount: 12, totalMinutes: 240, latestScore: 6,
          scoreHistory: [
            ScoreHistoryEntry(sessionDate: "2026-06-24T09:00:00Z", score: 6, sessionId: "s1"),
            ScoreHistoryEntry(sessionDate: "2026-06-21T09:00:00Z", score: 5, sessionId: "s2"),
            ScoreHistoryEntry(sessionDate: "2026-06-18T09:00:00Z", score: 4, sessionId: "s3"),
          ],
          latestTempo: 72, tempoHistory: [], lastPracticedAt: "2026-06-24T09:00:00Z"),
        latestAchievedTempo: nil, priority: false,
        linkedExercises: [
          LinkedExerciseView(
            id: "exercise-1", title: "Hanon No. 1", key: "C major", tempo: "♩ = 108",
            practice: ItemPracticeSummary(
              sessionCount: 8, totalMinutes: 60, latestScore: 7, scoreHistory: [],
              latestTempo: 108, tempoHistory: [], lastPracticedAt: "2026-06-28T09:00:00Z")),
          LinkedExerciseView(
            id: "exercise-2", title: "Db Major Scale", key: "Db major", tempo: nil,
            practice: ItemPracticeSummary(
              sessionCount: 3, totalMinutes: 20, latestScore: 4, scoreHistory: [],
              latestTempo: nil, tempoHistory: [], lastPracticedAt: "2026-06-25T09:00:00Z")),
          LinkedExerciseView(
            id: "exercise-3", title: "Arpeggios in Db", key: nil, tempo: nil,
            practice: nil),
        ],
        linkedFromPieces: [])
    }

    /// An exercise related to 2 pieces — for the "Related pieces" card snapshot.
    static var previewExerciseWithLinkedFrom: LibraryItemView {
      LibraryItemView(
        id: "exercise-1", itemType: .exercise, title: "Hanon No. 1",
        subtitle: "Charles-Louis Hanon",
        key: "C", modality: .major, tempo: "108 BPM", tempoMarking: nil, tempoBpm: 108,
        notes: nil, tags: [], createdAt: "", updatedAt: "",
        practice: ItemPracticeSummary(
          sessionCount: 9, totalMinutes: 90, latestScore: 7,
          scoreHistory: [
            ScoreHistoryEntry(sessionDate: "2026-06-24T09:00:00Z", score: 7, sessionId: "e1"),
            ScoreHistoryEntry(sessionDate: "2026-06-21T09:00:00Z", score: 6, sessionId: "e2"),
            ScoreHistoryEntry(sessionDate: "2026-06-18T09:00:00Z", score: 5, sessionId: "e3"),
          ],
          latestTempo: 108, tempoHistory: [], lastPracticedAt: "2026-06-24T09:00:00Z"),
        latestAchievedTempo: nil, priority: false, linkedExercises: [],
        linkedFromPieces: [
          PieceRefView(id: "piece-1", title: "Clair de Lune"),
          PieceRefView(id: "piece-2", title: "Gymnopédie No. 1"),
        ])
    }

    /// A piece with no linked exercises — for the empty-state snapshot.
    static var previewDetailLinkedEmpty: LibraryItemView {
      LibraryItemView(
        id: "piece-4", itemType: .piece, title: "Gymnopédie No. 1", subtitle: "Erik Satie",
        key: "D", modality: .major, tempo: "Lent et douloureux (60 BPM)",
        tempoMarking: "Lent et douloureux",
        tempoBpm: 60, notes: nil, tags: [], createdAt: "", updatedAt: "",
        practice: nil, latestAchievedTempo: nil, priority: false,
        linkedExercises: [], linkedFromPieces: [])
    }
  }

  extension SetlistEntryView {
    static var previewPiece: SetlistEntryView {
      building(id: "setlist-1", item: "piece-1", title: "Clair de Lune", type: .piece, position: 0)
    }

    static var previewExercise: SetlistEntryView {
      building(
        id: "setlist-2", item: "exercise-1", title: "Hanon No. 1", type: .exercise, position: 1)
    }

    static var previewGroupedScales: SetlistEntryView {
      building(id: "g-a", item: "ex-a", title: "Scales", type: .exercise, position: 0, group: "g1")
    }
    static var previewGroupedArpeggios: SetlistEntryView {
      building(
        id: "g-b", item: "ex-b", title: "Broken arpeggios", type: .exercise, position: 1,
        group: "g1")
    }
    static var previewGroupedPiece: SetlistEntryView {
      building(
        id: "g-p", item: "piece-1", title: "Clair de Lune", type: .piece, position: 2, group: "g1")
    }
    static var previewStandaloneExercise: SetlistEntryView {
      building(id: "g-s", item: "ex-c", title: "Sight-reading", type: .exercise, position: 3)
    }

    private static func building(
      id: String, item: String, title: String, type: ItemKind, position: UInt64,
      group: String? = nil
    ) -> SetlistEntryView {
      SetlistEntryView(
        id: id, itemId: item, itemTitle: title, itemType: type, position: position,
        durationDisplay: "—", status: .notAttempted, notes: nil, score: nil, intention: nil,
        repTarget: nil, repCount: nil, repTargetReached: nil, repHistory: nil,
        plannedDurationSecs: nil, plannedDurationDisplay: nil, achievedTempo: nil, groupId: group)
    }
  }

  extension PracticeSessionView {
    /// Sunday 31 May 2026 (noon UTC) — same Mon–Sun week as the fixtures below
    /// (Thu 28th, Sat 30th). "Today" has no practice, so the screen auto-selects
    /// the most recent earlier practice day (Sat 30th), exercising both the
    /// today-ring and selected-fill states deterministically.
    static var previewReferenceDate: Date {
      var components = DateComponents()
      components.year = 2026
      components.month = 5
      components.day = 31
      components.hour = 12
      return PreviewCalendar.utc.date(from: components) ?? .distantPast
    }

    /// Fixed past dates (not "now") so the card renders a deterministic absolute
    /// date — reusable from snapshot tests as well as the canvas.
    static var previewCompleted: PracticeSessionView {
      PracticeSessionView(
        id: "session-1", startedAt: "2026-05-30T09:00:00Z", finishedAt: "2026-05-30T09:32:00Z",
        totalDurationDisplay: "32m 0s", totalDurationSummary: "32m",
        completionStatus: .completed, notes: nil,
        entries: [
          previewEntry(0, "Clair de Lune", .piece),
          previewEntry(1, "Gymnopédie No. 1", .piece),
          previewEntry(2, "Nocturne Op. 9 No. 2", .piece),
        ],
        sessionIntention: nil, sessionScore: nil)
    }

    static var previewEndedEarly: PracticeSessionView {
      PracticeSessionView(
        id: "session-2", startedAt: "2026-05-28T18:00:00Z", finishedAt: "2026-05-28T18:14:00Z",
        totalDurationDisplay: "14m 0s", totalDurationSummary: "14m",
        completionStatus: .endedEarly, notes: nil,
        entries: [
          previewEntry(0, "Hanon No. 1", .exercise),
          previewEntry(1, "Major Scales", .exercise),
        ],
        sessionIntention: nil, sessionScore: nil)
    }

    private static func previewEntry(_ position: UInt64, _ title: String, _ type: ItemKind)
      -> SetlistEntryView
    {
      SetlistEntryView(
        id: "entry-\(position)", itemId: "item-\(position)", itemTitle: title, itemType: type,
        position: position, durationDisplay: "10 min", status: .completed, notes: nil,
        score: nil, intention: nil, repTarget: nil, repCount: nil, repTargetReached: nil,
        repHistory: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil, achievedTempo: nil,
        groupId: nil)
    }
  }

  extension ActiveSessionView {
    /// Item start instant + the snapshot reference (start + 4:12) so the timer
    /// renders a fixed `04:12` deterministically.
    static let previewStartedAt = "2026-05-30T09:00:00Z"
    static var previewReferenceDate: Date {
      (SessionClock.parseRFC3339(previewStartedAt) ?? .distantPast).addingTimeInterval(252)
    }

    static var previewActive: ActiveSessionView {
      ActiveSessionView(
        currentItemTitle: "Clair de Lune", currentItemType: .piece,
        currentPosition: 1, totalItems: 5,
        startedAt: previewStartedAt, currentItemStartedAt: previewStartedAt,
        entries: [], sessionIntention: "Even tempo — don't rush the runs",
        currentRepTarget: nil, currentRepCount: nil, currentRepTargetReached: nil,
        currentRepHistory: nil, currentPlannedDurationSecs: 480, nextItemTitle: "Hanon No. 1")
    }

    static var previewActiveReps: ActiveSessionView {
      ActiveSessionView(
        currentItemTitle: "Hanon No. 1", currentItemType: .exercise,
        currentPosition: 2, totalItems: 5,
        startedAt: previewStartedAt, currentItemStartedAt: previewStartedAt,
        entries: [], sessionIntention: "Keep the wrist relaxed",
        currentRepTarget: 8, currentRepCount: 3, currentRepTargetReached: false,
        currentRepHistory: nil, currentPlannedDurationSecs: nil, nextItemTitle: "Czerny Op. 299")
    }
  }

  extension SummaryView {
    static var previewSummary: SummaryView {
      SummaryView(
        totalDurationDisplay: "37m 50s", completionStatus: .completed, notes: nil,
        entries: [
          summaryEntry("e1", "Clair de Lune", .piece, "12m 40s", .completed, score: 3),
          summaryEntry("e2", "Hanon No. 1", .exercise, "8m 10s", .completed, score: 4, tempo: 96),
          summaryEntry("e3", "Gymnopédie No. 1", .piece, "11m 30s", .completed, score: 5),
          summaryEntry("e4", "Czerny Op. 299", .exercise, "5m 30s", .completed, score: 3),
        ], sessionIntention: nil, sessionScore: 8)
    }

    static var previewSummaryEndedEarly: SummaryView {
      SummaryView(
        totalDurationDisplay: "20m 50s", completionStatus: .endedEarly, notes: nil,
        entries: [
          summaryEntry("e1", "Clair de Lune", .piece, "12m 40s", .completed, score: 3),
          summaryEntry("e2", "Hanon No. 1", .exercise, "8m 10s", .completed, score: 4),
          summaryEntry("e3", "Étude Op. 10", .piece, "0s", .notAttempted, score: nil),
        ], sessionIntention: nil, sessionScore: 8)
    }

    private static func summaryEntry(
      _ id: String, _ title: String, _ type: ItemKind, _ duration: String,
      _ status: EntryStatus, score: UInt8?, tempo: UInt16? = nil
    ) -> SetlistEntryView {
      SetlistEntryView(
        id: id, itemId: id, itemTitle: title, itemType: type, position: 0,
        durationDisplay: duration, status: status, notes: nil, score: score, intention: nil,
        repTarget: nil, repCount: nil, repTargetReached: nil, repHistory: nil,
        plannedDurationSecs: nil, plannedDurationDisplay: nil, achievedTempo: tempo, groupId: nil)
    }
  }
#endif
