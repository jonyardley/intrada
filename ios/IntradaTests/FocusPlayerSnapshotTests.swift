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

  // The stop sits 42 seconds in, far from the snapshot's fixed instant, so a ring
  // that ignored it would read differently (#2297).
  func testFocusPlayerHoldsTheItemTimerWhileTheSheetIsOpen() throws {
    var active = ActiveSessionView.previewActive
    let start = try XCTUnwrap(SessionClock.parseRFC3339(active.currentItemStartedAt))
    active.reflection = ReflectionView(
      answers: ReflectionAnswers(
        marks: [], note: "", tempos: [], felt: nil, gotInTheWay: [], notePoints: [],
        intentionMet: nil, ways: []),
      reading: TempoReading(bpm: 72, clickSounding: false, click: nil),
      stoppedAt: SessionClock.nowRFC3339(start.addingTimeInterval(42)))
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: Store(bridge: PreviewBridge(activeSession: active))), as: config)
  }

  // A1's time ran out four seconds ago: On to B and Stay are offered (#2315).
  func testFocusPlayerSectionTimeUp() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: Store(bridge: PreviewBridge(activeSession: .previewActiveSectionTimeUp))),
      as: config)
  }

  func testFocusPlayerAwayOffer() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: Store(bridge: PreviewBridge(activeSession: .previewActiveAway))),
      as: config)
  }

  func testTroubleSpotSheet() {
    let sheet = TroubleSpotSheet(
      context: "Clair de Lune · in A1", barMax: 9999, refusal: nil, onAdd: { _, _ in true })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testClickControlStates() {
    // Flat player paper, not the radial wash: the gradient is not what's under
    // test here and it is most of a reference's bytes (snapshot hygiene).
    let states = ZStack {
      IntradaColor.playerBgMid
      VStack(spacing: 32) {
        ClickControl(
          bpm: 66, step: 2, band: 40...208, isRunning: false, unavailable: false,
          atSeededTempo: true,
          targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
          onToggle: {}, onStep: { _ in }, onDragChange: { _ in })
        ClickControl(
          bpm: 72, step: 2, band: 40...208, isRunning: true, unavailable: false,
          atSeededTempo: false,
          targetDisplay: "Andante · ♩ = 66", targetSpoken: "Andante, 66 beats per minute",
          onToggle: {}, onStep: { _ in }, onDragChange: { _ in })
        ClickControl(
          bpm: 96, step: 2, band: 40...208, isRunning: false, unavailable: false,
          atSeededTempo: true,
          targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
          onDragChange: { _ in })
        ClickControl(
          bpm: 96, step: 2, band: 40...208, isRunning: false, unavailable: true,
          atSeededTempo: true,
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
          bpm: 96, step: 2, band: 40...208, isRunning: false, unavailable: false,
          atSeededTempo: false,
          targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
          onDragChange: { _ in }, initiallyDragging: true)
        ClickControl(
          bpm: 168, unit: 8, step: 2, band: 80...416, isRunning: true, unavailable: false,
          atSeededTempo: false,
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
        ClickBarLine(
          metre: Metre(beats: 12, unit: 8, groups: [2, 2, 2, 2, 2, 2]),
          sounding: 0b0101_0101_0101, currentBeat: 0, onTap: {})
        ClickControl(
          bpm: 168, unit: 8, step: 2, band: 80...416, isRunning: true, unavailable: false,
          atSeededTempo: true,
          targetDisplay: nil, targetSpoken: nil, onToggle: {}, onStep: { _ in },
          onDragChange: { _ in })
      }
      .padding(.horizontal, IntradaSpacing.card)
    }
    assertSnapshot(of: host(states), as: config)
  }

  func testClickSheetIrregularMetre() throws {
    let store = Store(bridge: SnapshotStubBridge())
    let limits = try XCTUnwrap(store.viewModel?.limits)
    let sevenEight = Metre(beats: 7, unit: 8, groups: [3, 2, 2])
    var active = ActiveSessionView.previewActive
    active.clickSeedMetre = sevenEight
    active.clickSeedBpm = 168
    active.clickSeedPresets = limits.clickPresets(for: sevenEight)
    active.currentClickSounding = 0b1111111
    let click = ClickController()
    click.reseed(from: active, limits: limits)
    click.apply(.groupStarts)
    assertSnapshot(
      of: host(ClickSheet(click: click, bpm: 168, limits: limits), store: store), as: config)
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
      sessionElapsed: elapsed, positionLabel: "Focus · 3 of 5",
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

  func testFocusPlayerLastTimeOffer() {
    assertSnapshot(
      of: host(
        FocusPlayerScreen(referenceDate: ActiveSessionView.previewReferenceDate),
        store: Store(bridge: PreviewBridge(activeSession: .previewActiveLastTime))),
      as: config)
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

  /// A variation played earlier this item, not just scored in a past session,
  /// reads "Played this session" rather than falling back to its score (#1784).
  func testVariationPickerSheet() {
    let active = ActiveSessionView.previewActiveVariations
    let sheet = VariationPickerSheet(
      itemTitle: LibraryItemView.previewExerciseWithVariations.title,
      currentVariations: active.currentVariations,
      currentVariationId: active.currentVariationIds.first,
      onPick: { _ in true })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testRepCounter() {
    let counters = ZStack {
      PaperBackground()
      VStack(spacing: 24) {
        RepCounter(
          count: 0, slots: 10, touched: false, reached: false, onGotIt: {}, onNotQuite: {})
        RepCounter(
          count: 7, slots: 10, touched: true, reached: false, canUndo: true,
          onGotIt: {}, onNotQuite: {})
        RepCounter(
          count: 10, slots: 10, touched: true, reached: true, canUndo: true,
          onGotIt: {}, onNotQuite: {})
        RepCounter(
          count: 12, slots: 10, touched: true, reached: true, extra: 2, canUndo: true,
          onGotIt: {}, onNotQuite: {})
      }
      .padding(16)
    }
    assertSnapshot(of: host(counters), as: config)
  }

  func testRepCounterAccessibilitySize() {
    let counter = ZStack {
      PaperBackground()
      RepCounter(
        count: 12, slots: 10, touched: true, reached: true, extra: 2, canUndo: true,
        onGotIt: {}, onNotQuite: {}
      )
      .padding(16)
    }
    .dynamicTypeSize(.accessibility3)
    assertSnapshot(of: host(counter), as: config)
  }
}
