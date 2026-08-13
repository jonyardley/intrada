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

  // ── The built session (#1256, Journey A) ────────────────────────────

  /// A2 — first use: what the sheet costs is on the primary action, and the
  /// unresolved row wears no false kind while it still owes a question.
  func testComposeSheetFirstUse() {
    assertSnapshot(of: host(ComposeSheet(), store: .previewComposing), as: config)
  }

  /// A2r — the repeat visit, where every row is already known and the price is
  /// zero. The whole point of resolution being paid once per item, ever.
  func testComposeSheetAllKnown() {
    assertSnapshot(of: host(ComposeSheet(), store: .previewComposingAllKnown), as: config)
  }

  /// A3 — the proposed match carries its own evidence, so the confirmation is
  /// informed rather than blind.
  func testResolutionNodeMatch() {
    assertSnapshot(
      of: host(ResolutionFlow(onFinished: { _ in }), store: .previewResolvingNodeMatch),
      as: config)
  }

  /// A4 — the criterion sentence with its three read-back chips. The form the
  /// whole of decision 19b hangs on.
  func testResolutionUserDrill() {
    assertSnapshot(
      of: host(ResolutionFlow(onFinished: { _ in }), store: .previewComposing),
      as: config)
  }

  /// The read-back chips and the serves tags are the reflow risk on A4 — three
  /// chips across at accessibility sizes is where a form breaks. Cropped above
  /// the fold: the reflow *is* above the fold, and a full-height radial wash at
  /// this text size does not fit the hygiene ceiling.
  func testResolutionUserDrillLargeText() {
    assertSnapshot(
      of: host(ResolutionFlow(onFinished: { _ in }), store: .previewComposing),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 600),
        traits: UITraitCollection { traits in
          traits.displayScale = 2
          traits.preferredContentSizeCategory = .accessibilityExtraExtraExtraLarge
        }))
  }

  /// A6 — the composed session, shape offered and declinable. Component-level:
  /// the hero gradient is already covered by `testPressStartHeroPlanned`.
  func testComposedSession() {
    let composed = ComposedSessionScreen(session: .preview, onStart: {})
      .padding(IntradaSpacing.card)
      .background(IntradaColor.paperTop)
      .frame(width: 390)
    assertSnapshot(
      of: host(composed, store: .previewComposedSession),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 680),
        traits: .init(displayScale: 2)))
  }

  /// Component-level: the hero's gradient is already covered by
  /// `testPressStartHeroPlanned` and would triple this PNG.
  func testSessionOverview() {
    let overview = SessionOverview(
      blocks: PlanView.preview.blocks, deferred: PlanView.preview.deferred
    )
    .padding(IntradaSpacing.card)
    .background(IntradaColor.paperTop)
    .frame(width: 390)
    assertSnapshot(
      of: host(overview),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 560),
        traits: .init(displayScale: 2)))
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

  /// The coach blob's prompt (#1193, #1305). Two references, not one: the
  /// altitude variant swaps the eyebrow for the chip, which is the consent
  /// signal, so it can regress on its own.
  func testCoachRecoveryPromptCard() throws {
    let recovery = try XCTUnwrap(Store.previewCoachRecovery.viewModel?.coach.recovery)
    let card = RecoveryPromptCard(
      recovery: recovery, referenceDate: PracticeSessionView.previewReferenceDate,
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

  func testCoachRecoveryPromptCardAltitude() throws {
    let recovery = try XCTUnwrap(Store.previewCoachRecoveryAltitude.viewModel?.coach.recovery)
    let card = RecoveryPromptCard(
      recovery: recovery, referenceDate: PracticeSessionView.previewReferenceDate,
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

  // #1083 C2/C3: Steps section empty state — key-preset buttons + custom-steps link.
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

  // #1083 C2: Steps section — solid / current / unrated ring states, horizontal
  // scroller, "N of M solid" header; Key/Tempo rows hidden for laddered exercises.
  func testExerciseDetailWithSteps() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithSteps]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithSteps.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  /// Largest accessibility text size — proves the Steps scroller reflows
  /// rather than clipping or wrapping (#1083 C2).
  func testExerciseDetailWithStepsAccessibilitySize() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithSteps]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithSteps.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: axConfig)
  }

  // #1083 C2: 12-step ladder — survives max realistic length without wrapping.
  func testExerciseDetailWith12Steps() {
    let store = Store(bridge: PreviewBridge(items: [.previewExerciseWithFullLadder]))
    let pushed = NavigationStack(
      path: .constant([LibraryItemView.previewExerciseWithFullLadder.id])
    ) { LibraryScreen() }
    assertSnapshot(of: host(pushed, store: store), as: config)
  }

  // #1083 C2: minimal step-list creation sheet, opened from the "+ Add steps" link.
  func testAddStepsSheet() {
    assertSnapshot(of: host(AddStepsSheet(itemId: "exercise-1")), as: config)
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
        TypeBadge(kind: ItemKind.piece)
        TypeBadge(kind: ItemKind.exercise)
        TypeBadge(kind: ComposeKind.journal)
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

  // ── Press start: the way in to the drill loop ──

  /// Cropped to the hero rather than framed on the device: the paper below it
  /// adds nothing to the assertion and the gradient makes a full-screen
  /// reference too large for the hygiene ceiling.
  func testPressStartHeroPlanned() {
    let hero = ZStack {
      PaperBackground()
      PressStartHero(
        headline: "Rootless voicings",
        section: "A section",
        why: "Shells and rootless are what sit between you and improvising over Strasbourg.",
        footnote: "5 blocks · about 30 minutes",
        onStart: {}
      )
      .padding(IntradaSpacing.card)
    }
    assertSnapshot(
      of: host(hero),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 440),
        traits: .init(displayScale: 2)))
  }

  // ── The coach drill loop (A2 / A3) ──

  private func drill(_ state: DrillView) -> some View {
    DrillScreen(
      state: state, onVerdict: { _ in }, onStuck: {}, onDiscard: {}, onStart: {}, onSkip: {},
      onDismiss: {})
  }

  func testDrillScreenDuringPlay() {
    assertSnapshot(of: host(drill(.preview())), as: config)
  }

  /// Roughly what the screen looks like from two metres: the chip and tune line
  /// give way, the drill, tempo, position and target never do.
  func testDrillScreenDuringPlayAccessibilitySize() {
    assertSnapshot(of: host(drill(.preview())), as: axConfig)
  }

  /// l0: no tempo, no click, no beat position, tap-verdict reachable in place.
  func testDrillScreenDuringPlayUntimed() {
    let state = DrillView.preview(tempoBpm: nil, clickLevel: "no click")
    assertSnapshot(of: host(drill(state)), as: config)
  }

  func testDrillScreenTapVerdict() {
    assertSnapshot(of: host(drill(.preview(phase: .awaitingVerdict))), as: config)
  }

  /// The one branch the compact A3 snapshot can't reach: the escapes stack.
  func testDrillScreenTapVerdictAccessibilitySize() {
    assertSnapshot(of: host(drill(.preview(phase: .awaitingVerdict))), as: axConfig)
  }

  /// Identical composition to a pass — a miss is the user's own report.
  func testDrillScreenMissAcknowledged() {
    let state = DrillView.preview(phase: .acknowledged(clean: false), elapsedSeconds: 798)
    assertSnapshot(of: host(drill(state)), as: config)
  }

  /// The during-play page during the count-in: dots in the stuck target's
  /// place (#1184).
  func testDrillScreenCountIn() {
    let state = DrillView.preview(phase: .countIn(remaining: 2), elapsedSeconds: 803)
    assertSnapshot(of: host(drill(state)), as: config)
  }

  func testDrillScreenGateOpen() {
    let state = DrillView.preview(phase: .gateOpen, gateFilled: 3, elapsedSeconds: 842)
    assertSnapshot(of: host(drill(state)), as: config)
  }

  /// A block that has not run, so no clock — how every block opens.
  func testDrillScreenBlockEntry() {
    assertSnapshot(of: host(drill(.preview(phase: .blockEntry, elapsedSeconds: 0))), as: config)
  }

  /// The entry card names the click even where there isn't one.
  func testDrillScreenBlockEntryUntimed() {
    let state = DrillView.preview(
      phase: .blockEntry, elapsedSeconds: 0, tempoBpm: nil, clickLevel: "no click")
    assertSnapshot(of: host(drill(state)), as: config)
  }

  /// The parked-card clock, which a full-screen reference would cost 250KB
  /// to show.
  func testOrientationStripClockStates() {
    let strips = ZStack {
      RadialGradient.playerPaper
      VStack(spacing: IntradaSpacing.section) {
        OrientationStrip(
          elapsedSeconds: 724, ceilingSeconds: 360,
          blockKinds: [.piece, .exercise, .exercise], blockIndex: 1, onDismiss: {})
        OrientationStrip(
          elapsedSeconds: 184, blockKinds: [.piece, .exercise, .exercise], blockIndex: 1,
          onDismiss: {})
        OrientationStrip(
          elapsedSeconds: nil, blockKinds: [.piece, .exercise, .exercise], blockIndex: 1,
          onDismiss: {})
        Spacer()
      }
      .padding(IntradaSpacing.card)
    }
    assertSnapshot(
      of: host(strips),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 300),
        traits: .init(displayScale: 2)))
  }

  /// The sizes no screen snapshot reaches: iPad regular width, and an
  /// accessibility size where the labels grow but the keys must not.
  func testCoachActionWeights() {
    let keys = ZStack {
      RadialGradient.playerPaper
      VStack(spacing: IntradaSpacing.row) {
        CoachAction(title: "Start", emphasis: .primary, action: {})
        CoachAction(title: "I'm stuck", action: {})
        CoachAction(title: "Don't count that", emphasis: .quiet, action: {})
        CoachAction(title: "Start", emphasis: .primary, action: {})
          .environment(\.coachScale, .regular)
        CoachAction(title: "Start", emphasis: .primary, action: {})
          .dynamicTypeSize(.accessibility3)
      }
      .padding(IntradaSpacing.card)
    }
    assertSnapshot(
      of: host(keys),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 500),
        traits: .init(displayScale: 2)))
  }

  /// The two verdict-pair layouts no screen snapshot reaches: iPad regular
  /// width, and the accessibility-size stack.
  func testTapVerdictLayouts() {
    let pairs = ZStack {
      RadialGradient.playerPaper
      VStack(spacing: 20) {
        TapVerdict(onClean: {}, onMissed: {})
        TapVerdict(onClean: {}, onMissed: {})
          .environment(\.coachScale, .regular)
        TapVerdict(onClean: {}, onMissed: {})
          .dynamicTypeSize(.accessibility3)
      }
      .padding(16)
    }
    assertSnapshot(
      of: host(pairs),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 760, height: 520),
        traits: .init(displayScale: 2)))
  }

  /// Sized to the component, so three gate states cost one small reference.
  func testGateDotsStates() {
    let dots = ZStack {
      RadialGradient.playerPaper
      VStack(alignment: .leading, spacing: 14) {
        GateDots(filled: 0, target: 3)
        GateDots(filled: 2, target: 3)
        GateDots(filled: 3, target: 3, caption: "gate open")
        GateDots(filled: 3, target: 3, caption: "3 clean at 120")
      }
      .padding(16)
    }
    assertSnapshot(
      of: host(dots),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 260, height: 140),
        traits: .init(displayScale: 2)))
  }

  // ── Journey B's three altitudes (#1256 Phase C) ──

  private func runThrough(_ state: RunThroughView) -> some View {
    RunThroughScreen(
      state: state, onVerdict: { _ in }, onDiscard: {}, onFinish: {}, onDismiss: {})
  }

  /// Mid-run: the chip, the section the next tap judges, and the dots behind it
  /// carrying one held and one broken down.
  func testRunThroughScreenMidRun() {
    assertSnapshot(of: host(runThrough(.preview())), as: config)
  }

  /// Every section judged, and still not written: "Don't count this run" has to
  /// survive the last verdict (B1).
  func testRunThroughScreenComplete() {
    let state = RunThroughView.preview(held: [true, false, true, true], elapsedSeconds: 448)
    assertSnapshot(of: host(runThrough(state)), as: config)
  }

  /// The chip and the tune line give way; the section, the question and the
  /// pair never do.
  func testRunThroughScreenAccessibilitySize() {
    assertSnapshot(of: host(runThrough(.preview())), as: axConfig)
  }

  /// Fixed instants, so the clock the screen draws from `startedAt` is the same
  /// image every run.
  private static let openPlayNow = Date(timeIntervalSince1970: 1_786_226_298)

  func testOffPisteScreen() {
    let screen = OpenPlayScreen(state: .preview(), frozenNow: Self.openPlayNow, onDone: {})
    assertSnapshot(of: host(screen), as: config)
  }

  /// The emptiest screen in the app, on purpose: minutes in `inkSecondary`, no
  /// piece named, no primary action.
  func testUnmonitoredScreen() {
    let screen = OpenPlayScreen(
      state: .preview(altitude: .unmonitored), frozenNow: Self.openPlayNow, onDone: {})
    assertSnapshot(of: host(screen), as: config)
  }

  func testOffPisteScreenAccessibilitySize() {
    let screen = OpenPlayScreen(state: .preview(), frozenNow: Self.openPlayNow, onDone: {})
    assertSnapshot(of: host(screen), as: axConfig)
  }

  private func playThroughSheet(_ offer: AltitudeOffer) -> some View {
    PlayThroughSheet(offer: offer)
  }

  func testPlayThroughSheet() {
    let offer = AltitudeOffer(
      itemId: "p1", title: "Alice in Wonderland", runThroughAvailable: true,
      sections: ["A", "B", "Bridge", "A'"])
    assertSnapshot(of: host(playThroughSheet(offer)), as: config)
  }

  /// A piece with no labelled sections: the run-through stays on screen and
  /// says why, because a missing option with no explanation reads as a bug.
  func testPlayThroughSheetWithoutSections() {
    let offer = AltitudeOffer(
      itemId: "p1", title: "Alice in Wonderland", runThroughAvailable: false, sections: [])
    assertSnapshot(of: host(playThroughSheet(offer)), as: config)
  }

  /// The three chips together — the only place the consent gradient reads as a
  /// gradient.
  func testAltitudeChips() {
    let chips = ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: 14) {
        AltitudeChip(altitude: .runThrough)
        AltitudeChip(altitude: .offPiste)
        AltitudeChip(altitude: .unmonitored)
      }
      .padding(16)
    }
    assertSnapshot(
      of: host(chips),
      as: .image(
        on: .iPhone13, perceptualPrecision: 0.98, size: CGSize(width: 390, height: 160),
        traits: .init(displayScale: 2)))
  }

  // ── Journey C's qualitative capture (#1256 Phase D) ──

  private func feelScreen() -> some View {
    FeelScreen(
      prompt: FeelPrompt(blockId: "01BLOCK000000000000000003", title: "Freer rubato in the intro"),
      onFeel: { _ in }, onSkip: {})
  }

  /// The target in the user's own words, the question, and three chips of which
  /// only "It sang" is tinted — feel is not a verdict.
  func testFeelScreen() {
    assertSnapshot(of: host(feelScreen()), as: config)
  }

  /// The chips stack rather than shrink to three unreadable columns.
  func testFeelScreenAccessibilitySize() {
    assertSnapshot(of: host(feelScreen()), as: axConfig)
  }

  /// Nothing said yet: "Keep it" is unavailable, and "Not tonight" is a
  /// first-class exit rather than the harder path.
  func testSessionReflectionScreenEmpty() {
    assertSnapshot(of: host(SessionReflectionScreen(onKeep: { _ in }, onDismiss: {})), as: config)
  }

  /// The user's own sentence, in the serif reserved for their words.
  func testSessionReflectionScreenDictated() {
    let screen = SessionReflectionScreen(
      draft: "Stride's nearly there at 72. The bridge still rushes when I go from memory. "
        + "That's the thing to hit next.",
      onKeep: { _ in }, onDismiss: {})
    assertSnapshot(of: host(screen), as: config)
  }

  func testSessionReflectionScreenAccessibilitySize() {
    assertSnapshot(of: host(SessionReflectionScreen(onKeep: { _ in }, onDismiss: {})), as: axConfig)
  }

  /// C3 where it lands: above the untouched hero, on the real screen. Cropped
  /// to the top of the scroll — the rest of Practice is snapshotted elsewhere,
  /// and the hero's gradient is most of a full-screen reference's weight.
  func testPracticeScreenProposedSteer() {
    let screen = PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate)
    assertSnapshot(
      of: host(screen, store: .previewProposedSteer),
      as: .image(
        on: .iPhone13, perceptualPrecision: 0.98, size: CGSize(width: 390, height: 420),
        traits: .init(displayScale: 2)))
  }

  /// The card alone, at its own size — the quote glyph, the serif quote and the
  /// two text actions.
  func testProposedSteerCard() {
    let card = ZStack {
      PaperBackground()
      ProposedSteerCard(steer: .preview, onAccept: {}, onDecline: {})
        .padding(16)
    }
    assertSnapshot(
      of: host(card),
      as: .image(
        on: .iPhone13, perceptualPrecision: 0.98, size: CGSize(width: 390, height: 190),
        traits: .init(displayScale: 2)))
  }

  /// Accepted: the block sits second in the shape, wearing its provenance where
  /// the minutes go.
  func testSessionOverviewAddedByYou() {
    let plan = PlanView.previewWithSteer
    let overview = ZStack {
      PaperBackground()
      SessionOverview(blocks: plan.blocks, deferred: [])
        .padding(16)
    }
    assertSnapshot(
      of: host(overview),
      as: .image(
        on: .iPhone13, perceptualPrecision: 0.98, size: CGSize(width: 390, height: 330),
        traits: .init(displayScale: 2)))
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
