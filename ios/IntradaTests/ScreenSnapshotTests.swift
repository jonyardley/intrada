import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

private final class StubBridge: CoreBridge {
  private let core = CoreFfi()
  func update(_ event: Event) throws -> [Request] { [] }
  func resolve(_ id: UInt32, httpResult: HttpResult) throws -> [Request] { [] }
  func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request] { [] }
  func resolveEmpty(_ id: UInt32) throws -> [Request] { [] }
  func view() throws -> ViewModel {
    try ViewModel.bincodeDeserialize(input: [UInt8](core.view()))
  }
}

/// Force light mode at the controller level (SwiftUI reads colorScheme from
/// here, not the snapshot `traits:`) and pin `.iPhone13` + displayScale so the
/// host sim can't change the image; references recorded on iOS 26.5 to match CI.
@MainActor
final class ScreenSnapshotTests: XCTestCase {
  override func setUp() {
    super.setUp()
    IntradaFonts.register()
  }

  private func host(_ view: some View, store: Store = Store(bridge: StubBridge()))
    -> UIViewController
  {
    // Pin locale + calendar so date-driven UI (SessionCard's date, the week
    // strip) is deterministic regardless of host region/timezone — CI runs
    // en-US/UTC, dev sims often en-GB/local, which reorder dates and shift
    // day boundaries.
    // Suppress intro motion: the refreshed screens' entrance/one-shot animations
    // (fadeUp, count-up, ring-draw, barGrow, confetti) collapse to their final
    // state, so the captured frame is the settled layout, never a mid-reveal.
    // (`accessibilityReduceMotion` is read-only, so we use our settable flag.)
    let vc = UIHostingController(
      rootView: view.environment(store)
        .environment(\.locale, Locale(identifier: "en_US"))
        .environment(\.calendar, PreviewCalendar.utc)
        .environment(\.intradaMotionDisabled, true))
    vc.overrideUserInterfaceStyle = .light
    return vc
  }

  private var config: Snapshotting<UIViewController, UIImage> {
    .image(on: .iPhone13, perceptualPrecision: 0.98, traits: .init(displayScale: 2))
  }

  /// Largest accessibility text size — proves layouts reflow rather than clip/wrap.
  private var axConfig: Snapshotting<UIViewController, UIImage> {
    .image(
      on: .iPhone13, perceptualPrecision: 0.98,
      traits: UITraitCollection { traits in
        traits.displayScale = 2
        traits.preferredContentSizeCategory = .accessibilityExtraExtraExtraLarge
      })
  }

  func testRootShell() {
    assertSnapshot(of: host(RootView()), as: config)
  }

  func testGlobalBanner() {
    let banners = ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        GlobalBanner(message: "Couldn't delete that item.", onDismiss: {})
        GlobalBanner(message: "Storage unavailable — changes this session won't be saved.")
        Spacer()
      }
    }
    assertSnapshot(of: host(banners), as: config)
  }

  func testLibraryAddScreenWithError() {
    assertSnapshot(
      of: host(LibraryAddScreen(previewError: "A piece needs a composer.")), as: config)
  }

  func testLibraryEditScreenWithError() {
    assertSnapshot(
      of: host(LibraryEditScreen(item: .previewDetail, previewError: "A piece needs a composer.")),
      as: config)
  }

  func testLibraryScreen() {
    assertSnapshot(of: host(NavigationStack { LibraryScreen() }), as: config)
  }

  func testLibraryScreenPopulated() {
    assertSnapshot(
      of: host(NavigationStack { LibraryScreen() }, store: .previewLibrary), as: config)
  }

  func testLibraryScreenPriorities() {
    assertSnapshot(
      of: host(NavigationStack { LibraryScreen() }, store: .previewLibraryPriorities), as: config)
  }

  func testLibraryScreenFiltered() {
    assertSnapshot(
      of: host(NavigationStack { LibraryScreen() }, store: .previewLibraryFiltered), as: config)
  }

  func testLibraryScreenSearching() {
    assertSnapshot(
      of: host(
        NavigationStack { LibraryScreen(previewSearch: "clair") },
        store: .previewLibrarySearching), as: config)
  }

  func testPracticeScreen() {
    // Pin the date: the refreshed empty state shows the (live) week strip, so an
    // unfixed `Date()` would shift the week day-to-day and flake.
    assertSnapshot(
      of: host(PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate)), as: config)
  }

  func testPracticeScreenPopulated() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPractice), as: config)
  }

  func testRecoveryPromptCard() throws {
    // Component-level (not full-screen): the card is the load-bearing state and
    // the flat crop keeps the reference PNG well under the size ceiling (#840).
    let session = try XCTUnwrap(Store.previewPracticeRecovery.recoverableSession)
    let card = RecoveryPromptCard(
      session: session, referenceDate: PracticeSessionView.previewReferenceDate,
      onResume: {}, onDiscard: {}
    )
    .padding(IntradaSpacing.card)
    .background(IntradaColor.paperTop)
    .frame(width: 390)
    assertSnapshot(
      of: host(card),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 240),
        traits: .init(displayScale: 2)))
  }

  func testPracticeScreenQuietDay() {
    // Open on Monday — a day with no practice — to lock the per-day empty state.
    let monday = PracticeWeek.days(
      containing: PracticeSessionView.previewReferenceDate, calendar: PreviewCalendar.utc)[0]
    assertSnapshot(
      of: host(
        PracticeScreen(
          referenceDate: PracticeSessionView.previewReferenceDate, selectedDay: monday),
        store: .previewPractice), as: config)
  }

  func testSessionBuilderEmpty() {
    assertSnapshot(of: host(NavigationStack { SessionBuilderScreen() }), as: config)
  }

  func testSessionBuilderPopulated() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuilding), as: config)
  }

  func testSessionBuilderGrouped() {
    assertSnapshot(
      of: host(NavigationStack { SessionBuilderScreen() }, store: .previewBuildingGrouped),
      as: config)
  }

  func testFocusPlayerWithTarget() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: .previewActive), as: config)
  }

  func testFocusPlayerWithReps() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: .previewActiveReps), as: config)
  }

  func testSessionSummaryCompleted() {
    assertSnapshot(of: host(SessionSummaryScreen(), store: .previewSummary), as: config)
  }

  func testSessionSummaryWithReflection() {
    assertSnapshot(
      of: host(SessionSummaryScreen(), store: .previewSummaryWithReflection), as: config)
  }

  func testSessionSummaryEndedEarly() {
    assertSnapshot(
      of: host(SessionSummaryScreen(), store: .previewSummaryEndedEarly), as: config)
  }

  func testRoutinesScreen() {
    assertSnapshot(of: host(RoutinesScreen()), as: config)
  }

  func testAnalyticsScreen() {
    assertSnapshot(of: host(AnalyticsScreen()), as: config)
  }

  func testProgressScreenPopulated() {
    assertSnapshot(of: host(AnalyticsScreen(), store: .previewProgress), as: config)
  }

  func testLibraryScreenMastery() {
    assertSnapshot(
      of: host(NavigationStack { LibraryScreen() }, store: .previewLibraryMastery), as: config)
  }

  // ── Engaging-refresh components ──

  func testScoreRing() {
    let rings = ZStack {
      PaperBackground()
      HStack(spacing: 18) {
        ScoreRing(score: nil)
        ForEach([1, 4, 7, 10], id: \.self) { ScoreRing(score: $0) }
      }
      .padding(16)
    }
    assertSnapshot(of: host(rings), as: config)
  }

  func testScoreRingHero() {
    let hero = ZStack {
      PaperBackground()
      HStack(spacing: 24) {
        ScoreRing(score: 7, size: 132, showsScale: true)
        ScoreRing(score: nil, size: 132, showsScale: true)
      }
      .padding(16)
    }
    assertSnapshot(of: host(hero), as: config)
  }

  func testScoreSelectorPills() {
    let selectors = ZStack {
      PaperBackground()
      VStack(spacing: 20) {
        ScoreSelector(score: 0, accessibilityLabel: "Score") { _ in }
        ScoreSelector(score: 4, accessibilityLabel: "Score") { _ in }
        ScoreSelector(score: 10, accessibilityLabel: "Score") { _ in }
      }
      .padding(16)
    }
    assertSnapshot(of: host(selectors), as: config)
  }

  func testRecentSessions() {
    let block = ZStack {
      PaperBackground()
      RecentSessions(sessions: [
        RecentSession(id: "1", score: 7, dateText: "Tue · Jun 24"),
        RecentSession(id: "2", score: 6, dateText: "Sat · Jun 21"),
        RecentSession(id: "3", score: 5, dateText: "Wed · Jun 18"),
      ])
      .padding(16)
    }
    assertSnapshot(of: host(block), as: config)
  }

  func testRecentSessionsDeclining() {
    let block = ZStack {
      PaperBackground()
      RecentSessions(sessions: [
        RecentSession(id: "1", score: 5, dateText: "Tue · Jun 24"),
        RecentSession(id: "2", score: 6, dateText: "Sat · Jun 21"),
        RecentSession(id: "3", score: 8, dateText: "Wed · Jun 18"),
      ])
      .padding(16)
    }
    assertSnapshot(of: host(block), as: config)
  }

  func testAddRowButtonVariants() {
    let buttons = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        AddRowButton(title: "Add a related exercise") {}
        AddRowButton(title: "Add your first exercise", style: .outlined) {}
        AddRowButton(title: "Add a related exercise", style: .plain) {}
      }
      .padding(16)
    }
    assertSnapshot(of: host(buttons), as: config)
  }

  func testReflectionSheet() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: nil,
        onSave: { _, _, _, _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  func testReflectionSheetWithTempoTarget() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "Scales · D♭", elapsedDisplay: "7:00", tempoTarget: 96,
        onSave: { _, _, _, _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  // #1083 C2: Step picker, pre-selected to the current (not-yet-solid) step.
  func testReflectionSheetWithStepPicker() {
    let sheet = ZStack(alignment: .bottom) {
      PaperBackground()
      ReflectionSheet(
        itemTitle: "ii–V–i Enclosures", elapsedDisplay: "7:00", tempoTarget: nil,
        variants: LibraryItemView.previewExerciseWithSteps.variants,
        currentVariantId: LibraryItemView.previewExerciseWithSteps.variants.first(
          where: \.isCurrent
        )?.id,
        onSave: { _, _, _, _ in }, onSkip: {})
    }
    assertSnapshot(of: host(sheet), as: config)
  }

  func testMasteryDial() {
    let dial = ZStack {
      PaperBackground()
      MasteryDial(value: 3.4)
    }
    assertSnapshot(of: host(dial), as: config)
  }

  func testMasteryDeltaRows() {
    let rows = ZStack {
      PaperBackground()
      VStack(spacing: 12) {
        MasteryDelta(
          title: "Clair de Lune", subtitle: "D♭ major · now", was: 3, now: 4, kind: .piece)
        MasteryDelta(
          title: "Hanon No. 1", subtitle: "first time scored", was: nil, now: 3, kind: .exercise)
        MasteryDeltaToast(
          title: "Clair de Lune moved up", subtitle: "D♭ major mastery", was: 3, now: 4)
      }
      .padding(16)
    }
    assertSnapshot(of: host(rows), as: config)
  }

  func testConsistencyBars() {
    let bars = ZStack {
      PaperBackground()
      ConsistencyBars(weeks: [
        ConsistencyWeek(label: "W1", minutes: 40),
        ConsistencyWeek(label: "W2", minutes: 75),
        ConsistencyWeek(label: "W3", minutes: 55),
        ConsistencyWeek(label: "W4", minutes: 95),
        ConsistencyWeek(label: "Now", minutes: 82, isCurrent: true),
      ])
      .padding(16)
    }
    assertSnapshot(of: host(bars), as: config)
  }

  func testRepCounter() {
    let counters = ZStack {
      PaperBackground()
      VStack(spacing: 24) {
        RepCounter(count: 7, target: 12, onClean: {}, onMissed: {})
        RepCounter(count: 0, target: 8, onClean: {}, onMissed: {})  // Missed disabled
        RepCounter(count: 6, target: 6, onClean: {}, onMissed: {})  // Clean disabled
      }
      .padding(16)
    }
    assertSnapshot(of: host(counters), as: config)
  }

  func testLibraryDetailScreen() {
    // Preset path so the snapshot covers the real navigation chrome (back
    // chevron + transparent bar over the serif title), not just the body.
    let store = Store(bridge: PreviewBridge(items: [.previewDetail]))
    let pushed = NavigationStack(path: .constant([LibraryItemView.previewDetail.id])) {
      LibraryScreen()
    }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  /// The chord-chart card: parsed bar grid + "See the curriculum" (Phase A).
  func testLibraryDetailChordChartCard() {
    let store = Store(bridge: PreviewBridge(items: [.previewCharted]))
    let pushed = NavigationStack(path: .constant([LibraryItemView.previewCharted.id])) {
      LibraryScreen()
    }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  /// The selectable derived-curriculum commit sheet, with already-linked (not
  /// selectable) + fallback flags and per-row selection controls.
  func testScaffoldPreviewSheet() {
    assertSnapshot(
      of: host(ScaffoldPreviewSheet(preview: .preview, onCommit: { _ in })), as: config)
  }

  func testPieceDetailLinkedPopulated() {
    let store = Store(bridge: PreviewBridge(items: [.previewDetailWithLinkedExercises]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewDetailWithLinkedExercises.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  func testPieceDetailLinkedEmpty() {
    let store = Store(bridge: PreviewBridge(items: [.previewDetailLinkedEmpty]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewDetailLinkedEmpty.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  func testPieceDetailLinkedEditing() {
    let store = Store(bridge: PreviewBridge(items: [.previewDetailWithLinkedExercises]))
    // editingLinks is @State — seed via EditingLinkedExercisesWrapper with startEditingLinks=true.
    let editing = EditingLinkedExercisesWrapper(item: .previewDetailWithLinkedExercises)
    assertSnapshot(of: host(editing, store: store), as: config)
  }

  func testExerciseDetailLinkedFrom() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithLinkedFrom]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithLinkedFrom.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  // #1087 B2: overall-ring caption + "By piece" rows (live, removed, on-its-own).
  func testExerciseDetailByPiece() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithContexts]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithContexts.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  // #1083 C3: Steps section empty state — the two key-preset buttons.
  func testExerciseDetailStepsEmptyState() {
    let store = Store(bridge: PreviewBridge(items: [.previewExercise]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExercise.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  // #1083 C4: Steps edit mode — drag handle, inline rename field, remove button.
  func testExerciseDetailStepsEditing() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithSteps]))
    let editing = EditingStepsWrapper(item: .previewExerciseWithSteps)
    assertSnapshot(of: host(editing, store: store), as: config)
  }

  // #1083 C2: Steps section — solid / current / unrated ring states.
  func testExerciseDetailWithSteps() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithSteps]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithSteps.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  func testLibraryAddScreen() {
    assertSnapshot(of: host(LibraryAddScreen()), as: config)
  }

  func testLibraryAddScreenExercise() {
    assertSnapshot(of: host(LibraryAddScreen(defaultKind: .exercise)), as: config)
  }

  func testLibraryEditScreen() {
    assertSnapshot(of: host(LibraryEditScreen(item: .previewDetail)), as: config)
  }

  func testLibraryEditScreenExercise() {
    assertSnapshot(of: host(LibraryEditScreen(item: .previewExercise)), as: config)
  }

  func testTypeBadges() {
    let badges = ZStack {
      PaperBackground()
      HStack(spacing: 12) {
        TypeBadge(kind: .piece)
        TypeBadge(kind: .exercise)
      }
    }
    assertSnapshot(of: host(badges), as: config)
  }

  func testKeyPickerCollapsed() {
    let pickers = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        VStack(spacing: 0) {
          KeyPicker(label: "Key", key: .constant(""), modality: .constant(nil))
        }.cardSurface()
        VStack(spacing: 0) {
          KeyPicker(label: "Key", key: .constant("Gb"), modality: .constant(.major))
        }.cardSurface()
      }
      .padding(16)
    }
    assertSnapshot(of: host(pickers), as: config)
  }

  func testKeyPickerExpandedEmpty() {
    let picker = ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        KeyPicker(
          label: "Key", key: .constant(""), modality: .constant(nil), initiallyExpanded: true)
      }
      .cardSurface()
      .padding(16)
    }
    assertSnapshot(of: host(picker), as: config)
  }

  func testKeyPickerExpandedEnharmonic() {
    let picker = ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        KeyPicker(
          label: "Key", key: .constant("Gb"), modality: .constant(.major), initiallyExpanded: true)
      }
      .cardSurface()
      .padding(16)
    }
    assertSnapshot(of: host(picker), as: config)
  }

  func testAutocompleteField() {
    let pool = ["Bach", "Beethoven", "Brahms", "Chopin", "Debussy"]
    let fields = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        VStack(spacing: 0) {
          AutocompleteField(
            label: "Composer", text: .constant("B"), suggestions: pool,
            initiallyShowingSuggestions: true)
        }.cardSurface()
        VStack(spacing: 0) {
          AutocompleteField(label: "Composer", text: .constant("Ravel"), suggestions: pool)
        }.cardSurface()
      }
      .padding(16)
    }
    assertSnapshot(of: host(fields), as: config)
  }

  func testTagChipInput() {
    let pool = ["classical", "recital", "jazz", "warm-up", "technique", "etude"]
    let fields = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        VStack(spacing: 0) {
          TagChipInput(
            label: "Tags", tags: .constant(["classical", "recital"]), suggestions: pool,
            initiallyShowingSuggestions: true)
        }.cardSurface()
        VStack(spacing: 0) {
          TagChipInput(label: "Tags", tags: .constant([]), suggestions: pool)
        }.cardSurface()
      }
      .padding(16)
    }
    assertSnapshot(of: host(fields), as: config)
  }

  func testAddToSessionSheet() {
    assertSnapshot(of: host(AddToSessionSheet(), store: .previewBuilding), as: config)
  }

  func testSessionBuilderGroupedEditing() {
    // editMode is @State — seed via the startInEditMode init to capture the
    // nested-row reorder/remove/settings controls without UI interaction.
    assertSnapshot(
      of: host(
        NavigationStack { SessionBuilderScreen(startInEditMode: true) },
        store: .previewBuildingGrouped), as: config)
  }

  func testAddRelatedExerciseSheet() {
    assertSnapshot(
      of: host(
        AddRelatedExerciseSheet(groupId: "g1"), store: .previewBuildingGrouped),
      as: config)
  }

  func testEntrySettingsSheetEmpty() {
    assertSnapshot(
      of: host(EntrySettingsSheet(entry: .previewGroupedScales), store: .previewBuildingGrouped),
      as: config)
  }

  func testEntrySettingsSheetPopulated() {
    assertSnapshot(
      of: host(
        EntrySettingsSheet(entry: .previewGroupedScalesConfigured), store: .previewBuildingGrouped
      ), as: config)
  }

  func testTagFilterSheet() {
    let sheet = TagFilterSheet(
      available: ["classical", "jazz", "recital", "technique", "warm-up"],
      selected: ["jazz", "recital"],
      onChange: { _ in })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testTagFilterSheetEmpty() {
    let sheet = TagFilterSheet(available: [], selected: [], onChange: { _ in })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testLinkedExercisePicker() {
    // Three exercises; the first is already related (pre-selected → check), the
    // rest show the outlined add control.
    let sheet = LinkedExercisePickerSheet(
      available: [
        .previewExercise,
        LibraryItemView(
          id: "exercise-2", itemType: .exercise, title: "Db Major Scale", subtitle: "",
          key: "Db", modality: .major, tempo: nil, tempoMarking: nil, tempoBpm: nil,
          notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
          latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: [],
          exerciseContexts: [], scaffoldPreview: nil, chordChart: nil, variants: []),
        LibraryItemView(
          id: "exercise-3", itemType: .exercise, title: "Arpeggios in Db", subtitle: "",
          key: nil, modality: nil, tempo: nil, tempoMarking: nil, tempoBpm: nil,
          notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
          latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: [],
          exerciseContexts: [], scaffoldPreview: nil, chordChart: nil, variants: []),
      ],
      linkedIds: ["exercise-1"],
      onApply: { _ in })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testLinkedExercisePickerSelectedTray() {
    // Two of three pre-selected → the tray shows two removable chips above the
    // filter bar and list.
    let sheet = LinkedExercisePickerSheet(
      available: [
        .previewExercise,
        LibraryItemView(
          id: "exercise-2", itemType: .exercise, title: "Db Major Scale", subtitle: "",
          key: "Db", modality: .major, tempo: nil, tempoMarking: nil, tempoBpm: nil,
          notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
          latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: [],
          exerciseContexts: [], scaffoldPreview: nil, chordChart: nil, variants: []),
        LibraryItemView(
          id: "exercise-3", itemType: .exercise, title: "Arpeggios in Db", subtitle: "",
          key: nil, modality: nil, tempo: nil, tempoMarking: nil, tempoBpm: nil,
          notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
          latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: [],
          exerciseContexts: [], scaffoldPreview: nil, chordChart: nil, variants: []),
      ],
      linkedIds: ["exercise-1", "exercise-3"],
      onApply: { _ in })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testLibraryItemCards() {
    var manyTags = LibraryItemView.previewDetail
    manyTags.tags = ["jazz", "improv", "bebop", "ii-V-I", "comping"]
    // Starred: pins the accent star to the left of the tags + the trailing meter.
    var starred = LibraryItemView.previewDetail
    starred.priority = true
    let cards = ZStack {
      PaperBackground()
      VStack(spacing: 14) {
        LibraryItemCard(item: .previewPiece)
        LibraryItemCard(item: .previewDetail)
        LibraryItemCard(item: manyTags)  // 5 tags → +2 overflow pill
        LibraryItemCard(item: starred, showsMastery: true)
      }
      .padding(16)
    }
    assertSnapshot(of: host(cards), as: config)
  }
}
