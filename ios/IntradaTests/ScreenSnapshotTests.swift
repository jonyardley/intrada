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
    let vc = UIHostingController(
      rootView: view.environment(store)
        .environment(\.locale, Locale(identifier: "en_US"))
        .environment(\.calendar, PreviewCalendar.utc))
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
    assertSnapshot(of: host(PracticeScreen()), as: config)
  }

  func testPracticeScreenPopulated() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPractice), as: config)
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

  func testLibraryDetailScreen() {
    // Preset path so the snapshot covers the real navigation chrome (back
    // chevron + transparent bar over the serif title), not just the body.
    let store = Store(bridge: PreviewBridge(items: [.previewDetail]))
    let pushed = NavigationStack(path: .constant([LibraryItemView.previewDetail.id])) {
      LibraryScreen()
    }
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

  func testLibraryFilterTabs() {
    let tabs = ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: 16) {
        LibraryFilterTabs(selection: .constant(.all))
        LibraryFilterTabs(selection: .constant(.pieces))
        LibraryFilterTabs(selection: .constant(.exercises))
      }
      .padding(16)
    }
    assertSnapshot(of: host(tabs), as: config)
  }

  // #810: at the largest a11y size the pills stay one line + scroll, not wrap.
  func testLibraryFilterTabsAccessibility() {
    let tabs = ZStack {
      PaperBackground()
      LibraryFilterTabs(selection: .constant(.exercises))
        .padding(16)
    }
    assertSnapshot(of: host(tabs), as: axConfig)
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

  func testLibraryItemCards() {
    var manyTags = LibraryItemView.previewDetail
    manyTags.tags = ["jazz", "improv", "bebop", "ii-V-I", "comping"]
    let cards = ZStack {
      PaperBackground()
      VStack(spacing: 14) {
        LibraryItemCard(item: .previewPiece)
        LibraryItemCard(item: .previewDetail)
        LibraryItemCard(item: manyTags)  // 5 tags → +2 overflow pill
      }
      .padding(16)
    }
    assertSnapshot(of: host(cards), as: config)
  }
}
