import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class FocusPlayerSnapshotTests: SnapshotTestCase {
  func testFocusPlayerWithTarget() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: .previewActive), as: config)
  }

  func testClickControlStates() {
    // Flat player paper, not the radial wash: the gradient is not what's under
    // test here and it is most of a reference's bytes (snapshot hygiene).
    let states = ZStack {
      IntradaColor.playerBgMid
      VStack(spacing: 32) {
        ClickControl(
          bpm: 66, isRunning: false, unavailable: false, atSeededTempo: true,
          targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
          onToggle: {}, onStep: { _ in }, onDragChange: { _ in })
        ClickControl(
          bpm: 72, isRunning: true, unavailable: false, atSeededTempo: false,
          targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
          onToggle: {}, onStep: { _ in }, onDragChange: { _ in })
        ClickControl(
          bpm: 96, isRunning: false, unavailable: false, atSeededTempo: true,
          targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
          onDragChange: { _ in })
        ClickControl(
          bpm: 96, isRunning: false, unavailable: true, atSeededTempo: true,
          targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
          onDragChange: { _ in })
      }
      .padding(.horizontal, IntradaSpacing.card)
    }
    assertSnapshot(of: host(states), as: config)
  }

  /// The lifted, tinted, neighbour-flanked state a finger drag on the BPM
  /// capsule produces (#1823). `initiallyDragging` stands in for the gesture,
  /// which a snapshot host can't drive.
  func testClickControlDraggingStates() {
    let states = ZStack {
      IntradaColor.playerBgMid
      VStack(spacing: 32) {
        ClickControl(
          bpm: 96, isRunning: false, unavailable: false, atSeededTempo: false,
          targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
          onDragChange: { _ in }, initiallyDragging: true)
        ClickControl(
          bpm: 168, unit: 8, isRunning: true, unavailable: false, atSeededTempo: false,
          targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
          onDragChange: { _ in }, initiallyDragging: true)
      }
      .padding(.horizontal, IntradaSpacing.card)
    }
    assertSnapshot(of: host(states), as: config)
  }

  func testClickBarLineStates() {
    let states = ZStack {
      IntradaColor.playerBgMid
      VStack(spacing: 24) {
        ClickBarLine(
          metre: Metre(beats: 4, unit: 4, groups: nil), sounding: 0b1111, currentBeat: 1,
          onTap: {})
        ClickBarLine(
          metre: Metre(beats: 4, unit: 4, groups: nil), sounding: 0b1000, currentBeat: 3,
          onTap: {})
        ClickBarLine(
          metre: Metre(beats: 7, unit: 8, groups: [3, 2, 2]), sounding: 0b0101001,
          currentBeat: 0, onTap: {})
        ClickControl(
          bpm: 168, unit: 8, isRunning: true, unavailable: false, atSeededTempo: true,
          targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
          onDragChange: { _ in })
      }
      .padding(.horizontal, IntradaSpacing.card)
    }
    assertSnapshot(of: host(states), as: config)
  }

  func testClickSheetIrregularMetre() throws {
    let click = ClickController()
    click.reseed(target: 168, metre: Metre(beats: 7, unit: 8, groups: [3, 2, 2]))
    click.apply(.groupStarts)
    let store = Store(bridge: SnapshotStubBridge())
    let limits = try XCTUnwrap(store.viewModel?.limits)
    assertSnapshot(
      of: host(ClickSheet(click: click, bpm: 168, limits: limits), store: store), as: config)
  }

  func testClickSheetAccessibilitySize() throws {
    let click = ClickController()
    click.reseed(target: 168, metre: Metre(beats: 7, unit: 8, groups: [3, 2, 2]))
    click.apply(.groupStarts)
    let store = Store(bridge: SnapshotStubBridge())
    let limits = try XCTUnwrap(store.viewModel?.limits)
    assertSnapshot(
      of: host(ClickSheet(click: click, bpm: 168, limits: limits), store: store),
      as: tallAxConfig(height: 2600))
  }

  /// The sounding row is the tight one, so it is the state that has to reflow.
  func testClickControlLargeText() {
    let sounding = ZStack {
      IntradaColor.playerBgMid
      ClickControl(
        bpm: 208, isRunning: true, unavailable: false, atSeededTempo: false,
        targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
        onToggle: {}, onStep: { _ in }, onDragChange: { _ in }
      )
      .padding(.horizontal, IntradaSpacing.card)
    }
    assertSnapshot(of: host(sounding), as: axConfig)
  }

  // A session past an hour, where the reading becomes `H:MM:SS` and outgrows the
  // orientation slot's minimum. Full-screen because the whole strip reflows.
  func testFocusPlayerLongSession() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: .previewActiveLongSession), as: config)
  }

  // Band-level rather than full-screen: what can regress is the slot clipping a
  // long clock, and a component reference costs a few KB against 300k+ for the
  // player's radial gradient (snapshot hygiene).
  private func orientationBand(elapsed: Int) -> some View {
    SessionOrientationBand(
      sessionElapsed: elapsed, positionLabel: "FOCUS · 3 OF 5",
      types: [.exercise, .exercise, .exercise, .piece, .exercise], filled: 3,
      menu: { Image(systemName: "ellipsis").frame(width: 28, height: 28) }
    )
    .frame(width: 342)
    .padding(IntradaSpacing.card)
    .background(IntradaColor.playerBgMid)
  }

  func testOrientationBandLongSession() {
    assertSnapshot(of: orientationBand(elapsed: 3732), as: .image(precision: 0.99))
  }

  func testOrientationBandLargeText() {
    assertSnapshot(
      of: orientationBand(elapsed: 3732),
      as: .image(
        precision: 0.99,
        traits: UITraitCollection { traits in
          traits.preferredContentSizeCategory = .accessibilityExtraExtraExtraLarge
        }))
  }

  func testFocusPlayerWithReps() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: .previewActiveReps), as: config)
  }

  /// The variation chip, on the one screen where space is tightest (#1739).
  func testFocusPlayerWithVariations() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: .previewActiveVariations), as: config)
  }

  func testFocusPlayerWithVariationsAccessibilitySize() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: .previewActiveVariations), as: axConfig)
  }

  /// A variation played earlier this item, not just scored in a past session,
  /// reads "Played this session" rather than falling back to its score (#1784).
  func testVariationPickerSheet() {
    let active = ActiveSessionView.previewActiveVariations
    let sheet = VariationPickerSheet(
      itemTitle: LibraryItemView.previewExerciseWithVariations.title,
      currentVariations: active.currentVariations,
      currentVariationId: active.currentVariationId,
      onPick: { _ in true })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testVariationPickerSheetAccessibilitySize() {
    let active = ActiveSessionView.previewActiveVariations
    let sheet = VariationPickerSheet(
      itemTitle: LibraryItemView.previewExerciseWithVariations.title,
      currentVariations: active.currentVariations,
      currentVariationId: active.currentVariationId,
      onPick: { _ in true })
    assertSnapshot(of: host(sheet), as: axConfig)
  }

  /// Variations named in words rather than key letters, at the largest text
  /// size: the rows wrap rather than truncating what the musician called them,
  /// including the "Played this session" caption at its longest (#1784).
  func testVariationPickerSheetLongLabels() {
    let currentVariations = [
      PickerVariationView(
        id: "rung-0", label: "Root position", caption: "Playing now", isSolid: false),
      PickerVariationView(
        id: "rung-1", label: "1st inversion", caption: "Played this session · 12m 34s",
        isSolid: true),
      PickerVariationView(
        id: "rung-2", label: "2nd inversion", caption: "Not yet played", isSolid: false),
    ]
    let sheet = VariationPickerSheet(
      itemTitle: "Triad inversions",
      currentVariations: currentVariations,
      currentVariationId: currentVariations.first?.id,
      onPick: { _ in true })
    assertSnapshot(of: host(sheet), as: axConfig)
  }

  func testRepCounter() {
    let counters = ZStack {
      PaperBackground()
      VStack(spacing: 24) {
        RepCounter(
          count: 0, slots: 10, touched: false, reached: false, onGotIt: {}, onNotQuite: {})
        RepCounter(
          count: 7, slots: 10, touched: true, reached: false, onGotIt: {}, onNotQuite: {})
        RepCounter(
          count: 10, slots: 10, touched: true, reached: true, onGotIt: {}, onNotQuite: {})
      }
      .padding(16)
    }
    assertSnapshot(of: host(counters), as: config)
  }

  func testRepCounterAccessibilitySize() {
    let counter = ZStack {
      PaperBackground()
      RepCounter(
        count: 3, slots: 10, touched: true, reached: false, onGotIt: {}, onNotQuite: {}
      )
      .padding(16)
    }
    .dynamicTypeSize(.accessibility3)
    assertSnapshot(of: host(counter), as: config)
  }
}
