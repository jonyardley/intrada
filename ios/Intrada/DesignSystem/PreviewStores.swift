#if DEBUG
  import Foundation
  import IntradaCoreFFI
  import SharedTypes

  /// A fixed Gregorian/UTC calendar so date-derived UI (the week strip) renders
  /// identically on any host; pair it with `previewReferenceDate` in previews
  /// and pin it via `.environment(\.calendar, PreviewCalendar.utc)` in snapshot hosts.
  enum PreviewCalendar {
    static var utc: Calendar {
      var calendar = Calendar(identifier: .gregorian)
      calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
      return calendar
    }
  }

  /// Offline bridge for Xcode previews: serves the core's initial (empty)
  /// ViewModel, optionally seeded with library items, and emits no effects,
  /// so store-backed screens render in the canvas without FFI networking.
  final class PreviewBridge: CoreBridge {
    private let core = CoreFfi()
    private let items: [LibraryItemView]
    private let activeQuery: ListQuery?
    private let sessions: [PracticeSessionView]
    private let practiceWeeks: [PracticeWeekView]?
    private let buildingSetlist: BuildingSetlistView?
    private let activeSession: ActiveSessionView?
    private let summary: SummaryView?

    private let analytics: AnalyticsView?
    private let lastPractised: LastPractisedView?
    private let upNext: SuggestedSession?
    private let recentlyPractisedIds: [String]
    private let visibleIds: [String]?
    private let profile: ProfileView?

    init(
      items: [LibraryItemView] = [], activeQuery: ListQuery? = nil,
      visibleIds: [String]? = nil,
      sessions: [PracticeSessionView] = [], practiceWeeks: [PracticeWeekView]? = nil,
      buildingSetlist: BuildingSetlistView? = nil,
      activeSession: ActiveSessionView? = nil, summary: SummaryView? = nil,
      analytics: AnalyticsView? = nil, lastPractised: LastPractisedView? = nil,
      upNext: SuggestedSession? = nil, recentlyPractisedIds: [String] = [],
      profile: ProfileView? = nil
    ) {
      self.items = items
      self.activeQuery = activeQuery
      self.sessions = sessions
      self.practiceWeeks = practiceWeeks
      self.buildingSetlist = buildingSetlist
      self.activeSession = activeSession
      self.summary = summary
      self.analytics = analytics
      self.lastPractised = lastPractised
      self.upNext = upNext
      self.recentlyPractisedIds = recentlyPractisedIds
      self.visibleIds = visibleIds
      self.profile = profile
    }

    func update(_ event: Event) throws -> [Request] { [] }
    func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request] { [] }
    func resolve(_ id: UInt32, recognitionOutput: RecognitionOutput) throws -> [Request] { [] }
    func resolveEmpty(_ id: UInt32) throws -> [Request] { [] }
    func view() throws -> ViewModel {
      var viewModel = try ViewModel.bincodeDeserialize(input: [UInt8](core.view()))
      viewModel.activeQuery = activeQuery
      let visible: [LibraryItemView]
      if let visibleIds {
        visible = items.filter { visibleIds.contains($0.id) }
      } else {
        let kind = activeQuery?.itemType
        let priorityOnly = activeQuery?.priorityOnly ?? false
        visible = items.filter { item in
          (kind.map { item.itemType == $0 } ?? true) && (!priorityOnly || item.priority)
        }
      }
      viewModel.visibleIds = visible.map(\.id)
      viewModel.visiblePieces = UInt64(visible.filter { $0.itemType == .piece }.count)
      viewModel.visibleExercises = UInt64(visible.filter { $0.itemType == .exercise }.count)
      // Derived from the whole library, never `visible`, so a fixture with a
      // filter on still reports what the core would report (#981).
      viewModel.showsPriorities =
        items.contains { $0.priority } && buildingSetlist == nil && activeSession == nil
        && summary == nil
      viewModel.buildingSetlist = buildingSetlist
      viewModel.activeSession = activeSession
      viewModel.summary = summary
      if let analytics { viewModel.analytics = analytics }
      viewModel.lastPractised = lastPractised
      viewModel.upNext = upNext
      viewModel.recentlyPractisedIds = recentlyPractisedIds
      if let profile { viewModel.profile = profile }
      return viewModel
    }

    var sections: [AppEffect] {
      [.libraryChanged(items), .historyChanged(sessions), .weeksChanged(practiceWeeks ?? [])]
    }
  }

  extension Store {
    /// A store over a preview bridge, holding the rows the bridge was given.
    convenience init(bridge: PreviewBridge) {
      self.init(bridge: bridge as CoreBridge)
      receive(bridge.sections)
    }

    /// A deterministic, offline store for `#Preview` blocks.
    static var preview: Store { Store(bridge: PreviewBridge()) }

    static var previewPracticeEmpty: Store {
      Store(bridge: PreviewBridge(practiceWeeks: [.previewEmptyWeek]))
    }

    /// A cellist called Jon on coral, greeted in the morning (#1692).
    static var previewProfile: Store {
      Store(bridge: PreviewBridge(profile: .previewCellist))
    }

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
          activeQuery: ListQuery(
            text: nil, itemType: .piece, key: nil, tags: [], priorityOnly: false)))
    }

    /// Text-searched library for the revealed-search-bar snapshot: "clair"
    /// matches Clair de Lune.
    static var previewLibrarySearching: Store {
      Store(
        bridge: PreviewBridge(
          items: [.previewPiece, .previewExercise, .previewMinimal],
          activeQuery: ListQuery(
            text: "clair", itemType: nil, key: nil, tags: [], priorityOnly: false),
          visibleIds: [LibraryItemView.previewPiece.id]))
    }

    /// A store driven by the *real* core seeded with the canonical demo dataset
    /// (`Event.loadSampleData` → `sample_items()`). Render-only, so it completes
    /// synchronously and offline. Use in screen previews: same data as the CI
    /// screenshot, and the filter pills actually work in the canvas.
    /// Not for snapshot tests: `sample_items()` stamps wall-clock timestamps.
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
    /// populated-state snapshot, covers both completed + ended-early cards.
    static var previewPractice: Store {
      Store(
        bridge: PreviewBridge(
          sessions: [.previewCompleted, .previewEndedEarly], practiceWeeks: [.previewWeek],
          lastPractised: .previewYesterday))
    }

    /// The Practice home as a named cellist sees it: greeted, with the badge (#1694).
    static var previewPracticeProfile: Store {
      Store(
        bridge: PreviewBridge(
          sessions: [.previewCompleted, .previewEndedEarly], practiceWeeks: [.previewWeek],
          lastPractised: .previewYesterday, profile: .previewCellist))
    }

    /// Practice home with something to suggest (#1082).
    static var previewPracticeSuggestion: Store {
      Store(
        bridge: PreviewBridge(
          sessions: [.previewCompleted, .previewEndedEarly], practiceWeeks: [.previewWeek],
          lastPractised: .previewYesterday, upNext: .previewStarred))
    }

    /// Practice home with something starred, so the priorities route shows
    /// under the ordinary hero (#981).
    static var previewPracticePriorities: Store {
      Store(
        bridge: PreviewBridge(
          items: [starred(.previewPiece), .previewMinimal],
          sessions: [.previewCompleted, .previewEndedEarly], practiceWeeks: [.previewWeek],
          lastPractised: .previewYesterday))
    }

    /// A suggestion and a starred library together: the one layout where two
    /// secondary actions stack under a single primary (#981).
    static var previewPracticeSuggestionPriorities: Store {
      Store(
        bridge: PreviewBridge(
          items: [starred(.previewPiece), .previewMinimal],
          sessions: [.previewCompleted, .previewEndedEarly], practiceWeeks: [.previewWeek],
          lastPractised: .previewYesterday, upNext: .previewStarred))
    }

    /// Practice home with a crash-recovery blob pending (#962), drives the
    /// Resume / Discard prompt above the hero.
    static var previewPracticeRecovery: Store {
      let store = Store(
        bridge: PreviewBridge(
          sessions: [.previewCompleted, .previewEndedEarly], practiceWeeks: [.previewWeek],
          lastPractised: .previewYesterday))
      store.recoverableSession = ActiveSession(
        id: "recover-1",
        entries: [
          SetlistEntry(
            id: "re1", itemId: "i1", itemTitle: "Scales · D♭ major", itemType: .exercise,
            position: 0, durationSecs: 180, status: .completed,
            notes: nil, intention: nil, plannedDurationSecs: nil,
            groupId: nil, plannedVariationId: nil, plannedRepTarget: nil,
            plays: [
              VariationPlay(
                id: "re1-p1", variationId: nil, startedAt: "2026-06-16T08:59:00Z",
                seconds: 180, repTarget: nil, repCount: nil, repTargetReached: nil,
                repHistory: nil, achievedTempo: nil, clickPattern: nil,
                score: nil)
            ]),
          SetlistEntry(
            id: "re2", itemId: "i2", itemTitle: "Clair de Lune", itemType: .piece,
            position: 1, durationSecs: 0, status: .notAttempted,
            notes: nil, intention: nil, plannedDurationSecs: nil,
            groupId: nil, plannedVariationId: nil, plannedRepTarget: nil, plays: []),
        ],
        currentIndex: 1,
        currentItemStartedAt: "2026-06-16T09:02:00Z", sessionStartedAt: "2026-06-16T09:02:00Z")
      return store
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
                entries: [.previewPiece], takenElsewhere: []),
              SetlistBlockView(
                groupId: nil, pieceTitle: nil, relatedCount: 0, durationDisplay: "—",
                entries: [.previewExercise], takenElsewhere: []),
            ],
            totalDurationDisplay: nil, totalDurationSummary: nil, entryVariations: [])))
    }

    /// Session builder's add-items sheet with a "Recently practised" quick-add
    /// section (#1362), independent of `previewBuilding`'s own items/setlist.
    static var previewBuildingRecentlyPractised: Store {
      Store(
        bridge: PreviewBridge(
          items: [.previewPiece, .previewExercise, .previewMinimal],
          buildingSetlist: BuildingSetlistView(
            entries: [], itemCount: 0, blocks: [],
            totalDurationDisplay: nil, totalDurationSummary: nil, entryVariations: []),
          recentlyPractisedIds: [
            LibraryItemView.previewPiece.id, LibraryItemView.previewExercise.id,
          ]))
    }

    /// Same as `previewBuildingRecentlyPractised`, but with the type filter set
    /// to exercises, the section must not survive an active filter (#1362).
    static var previewBuildingRecentlyPractisedFiltered: Store {
      Store(
        bridge: PreviewBridge(
          items: [.previewPiece, .previewExercise],
          activeQuery: ListQuery(
            text: nil, itemType: .exercise, key: nil, tags: [], priorityOnly: false),
          buildingSetlist: BuildingSetlistView(
            entries: [], itemCount: 0, blocks: [],
            totalDurationDisplay: nil, totalDurationSummary: nil, entryVariations: []),
          recentlyPractisedIds: [
            LibraryItemView.previewPiece.id, LibraryItemView.previewExercise.id,
          ]))
    }

    /// Session builder with a block (a piece + 2 related) above a standalone
    /// item, the grouped-state preview + snapshot.
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
                durationDisplay: "12 min", entries: block, takenElsewhere: ["ex-c"]),
              SetlistBlockView(
                groupId: nil, pieceTitle: nil, relatedCount: 0, durationDisplay: "—",
                entries: [.previewStandaloneExercise], takenElsewhere: []),
            ],
            totalDurationDisplay: "12m 0s", totalDurationSummary: "12 min", entryVariations: [])))
    }

    /// `previewBuildingGrouped` as the related-exercise sheet sees it: the
    /// sheet's own scope has narrowed the query to exercises (#1999).
    static var previewBuildingGroupedRelatedSheet: Store {
      let block: [SetlistEntryView] = [
        .previewGroupedScales, .previewGroupedArpeggios, .previewGroupedPiece,
      ]
      return Store(
        bridge: PreviewBridge(
          items: [.previewPiece, .previewExercise, .previewMinimal],
          activeQuery: ListQuery(
            text: nil, itemType: .exercise, key: nil, tags: [], priorityOnly: false),
          buildingSetlist: BuildingSetlistView(
            entries: block + [.previewStandaloneExercise],
            itemCount: 4,
            blocks: [
              SetlistBlockView(
                groupId: "g1", pieceTitle: "Clair de Lune", relatedCount: 2,
                durationDisplay: "12 min", entries: block, takenElsewhere: ["ex-c"]),
              SetlistBlockView(
                groupId: nil, pieceTitle: nil, relatedCount: 0, durationDisplay: "—",
                entries: [.previewStandaloneExercise], takenElsewhere: []),
            ],
            totalDurationDisplay: "12m 0s", totalDurationSummary: "12 min", entryVariations: [])))
    }

    /// Session builder where one of the block's related exercises is also in
    /// the library, the added state of the add-related sheet (#1103).
    static var previewBuildingGroupedAdded: Store {
      let block: [SetlistEntryView] = [.previewGroupedScales, .previewGroupedPiece]
      return Store(
        bridge: PreviewBridge(
          items: [.previewScales, .previewExercise],
          buildingSetlist: BuildingSetlistView(
            entries: block,
            itemCount: 2,
            blocks: [
              SetlistBlockView(
                groupId: "g1", pieceTitle: "Clair de Lune", relatedCount: 1,
                durationDisplay: "12 min", entries: block, takenElsewhere: [])
            ],
            totalDurationDisplay: "12m 0s", totalDurationSummary: "12 min", entryVariations: [])))
    }

    /// Player Focus: a piece mid-session, no reps.
    static var previewActive: Store {
      Store(bridge: PreviewBridge(activeSession: .previewActive))
    }

    /// Player Focus: an exercise with an active rep counter.
    static var previewActiveLongSession: Store {
      Store(bridge: PreviewBridge(activeSession: .previewActiveLongSession))
    }

    static var previewActiveReps: Store {
      Store(bridge: PreviewBridge(activeSession: .previewActiveReps))
    }

    /// Player Focus: an exercise with variations, so the picker chip shows
    /// what is being practised right now (#1739).
    static var previewActiveVariations: Store {
      Store(
        bridge: PreviewBridge(
          items: [.previewExerciseWithVariations], activeSession: .previewActiveVariations))
    }

    static var previewSummary: Store {
      Store(bridge: PreviewBridge(summary: .previewSummary, analytics: .previewAnalytics))
    }

    /// Player Summary: one exercise practised across three keys, so each gets
    /// its own mark (#1739 decision 10).
    static var previewSummaryVariations: Store {
      Store(bridge: PreviewBridge(summary: .previewSummaryVariations))
    }

    /// Player Summary: ended early, so the unreached item shows not-attempted.
    static var previewSummaryEndedEarly: Store {
      Store(bridge: PreviewBridge(summary: .previewSummaryEndedEarly))
    }

    /// Progress: a populated analytics view (dial, consistency, recent
    /// mastery) plus two practised exercises with variations, so the coverage
    /// section renders (#1739).
    static var previewProgress: Store {
      Store(
        bridge: PreviewBridge(
          items: [
            scored(.previewExerciseWithVariations, 7),
            scored(.previewExerciseWithTwelveVariations, 6),
          ],
          analytics: .previewAnalytics))
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

    /// Detail view: piece with no linked exercises, shows the empty state.
    static var previewDetailLinkedEmpty: Store {
      Store(bridge: PreviewBridge(items: [.previewDetailLinkedEmpty]))
    }

    /// Detail view: exercise linked to 2 pieces it has never been practised with.
    static var previewExerciseLinkedOnlyStore: Store {
      Store(bridge: PreviewBridge(items: [.previewExerciseLinkedOnly]))
    }

    private static func scored(_ item: LibraryItemView, _ score: UInt8) -> LibraryItemView {
      var copy = item
      copy.practice = ItemPracticeSummary.fixture(
        sessionCount: 8, totalMinutes: 120, latestScore: score, scoreHistory: [],
        lastPracticedAt: "2026-05-30T09:00:00Z")
      return copy
    }
  }

  extension ProfileView {
    static var previewCellist: ProfileView {
      ProfileView(
        name: "Jon", instrument: "Cello", suggestedIcon: .cello, icon: .cello, colour: .coral,
        greeting: "Morning, Jon")
    }
  }
#endif
