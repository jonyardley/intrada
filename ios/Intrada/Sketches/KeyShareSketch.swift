#if DEBUG
  import SharedTypes
  import SwiftUI

  struct KeyShareSketch {
    enum Count: CaseIterable { case chip, underChip, band }

    enum Kind {
      case key, variation

      var word: String {
        switch self {
        case .key: "Key"
        case .variation: "Variation"
        }
      }
    }

    struct State {
      let title: String
      let kind: Kind
      let label: String
      let index: Int
      let total: Int
      let itemElapsed: Int
      let itemPlanned: Int
      let parts: [Double]
      let nextName: String?
      let shareUp: Bool

      var counted: Bool { total > 1 }
      var countText: String { "\(index + 1) of \(total)" }
    }

    static let filling = State(
      title: "Major scales", kind: .key, label: "C major", index: 0, total: 3,
      itemElapsed: 110, itemPlanned: 540, parts: [110.0 / 180, 0, 0],
      nextName: "G major", shareUp: false)

    static let over = State(
      title: "Major scales", kind: .key, label: "C major", index: 0, total: 3,
      itemElapsed: 205, itemPlanned: 540, parts: [1, 0, 0],
      nextName: "G major", shareUp: true)

    static let lastKey = State(
      title: "Major scales", kind: .key, label: "D major", index: 2, total: 3,
      itemElapsed: 435, itemPlanned: 540, parts: [1, 1, 75.0 / 180],
      nextName: nil, shareUp: false)

    static let oneKey = State(
      title: "Arpeggios", kind: .key, label: "E♭ major", index: 0, total: 1,
      itemElapsed: 190, itemPlanned: 300, parts: [],
      nextName: nil, shareUp: false)

    static let variations = State(
      title: "Hanon No. 1", kind: .variation, label: "Dotted rhythm", index: 1, total: 3,
      itemElapsed: 270, itemPlanned: 540, parts: [1, 90.0 / 180, 0],
      nextName: "Legato", shareUp: false)

    static let items = 3
    static let currentItem = 1
    static let nextItem = "Bach Prelude in C"
  }

  // ── The player, drawn from its public parts with fixture values ──

  struct KeySharePlayerSketch: View {
    var count: KeyShareSketch.Count = .chip
    var state: KeyShareSketch.State = KeyShareSketch.filling

    var body: some View {
      ZStack {
        IntradaColor.paperTop.ignoresSafeArea()
        VStack(spacing: 0) {
          SketchOrientationBand(count: count, state: state)
          Spacer(minLength: IntradaSpacing.card)
          centerInfo
          SketchTimerRing(elapsed: state.itemElapsed, planned: state.itemPlanned)
            .padding(.top, IntradaSpacing.section)
          ClickControl(
            bpm: 96, step: 2, band: 40...208, isRunning: false, unavailable: false,
            atSeededTempo: true, targetDisplay: nil, targetSpoken: nil,
            onToggle: {}, onStep: { _ in }, onDragChange: { _ in }
          )
          .padding(.top, IntradaSpacing.controlGap)
          Spacer(minLength: IntradaSpacing.card)
          SketchMoveOnControls(nextName: state.nextName, shareUp: state.shareUp)
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
      }
    }

    private var centerInfo: some View {
      VStack(spacing: IntradaSpacing.controlGap) {
        TypeBadge(kind: .exercise)
        Text(state.title)
          .font(IntradaFont.pageTitle)
          .foregroundStyle(IntradaColor.ink)
        if state.counted {
          SketchKeyChip(
            label: state.label, count: count == .chip ? state.countText : nil,
            spoken: "\(state.kind.word), \(state.label), \(state.countText)")
          if count == .underChip {
            Text("\(state.kind.word) \(state.countText)")
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
              .accessibilityHidden(true)
          }
        } else {
          Text(state.label)
            .font(IntradaFont.segment)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
    }
  }

  // ── Bottom: pause stays the big ink circle, moving on is a quiet line under it ──

  struct SketchMoveOnControls: View {
    let nextName: String?
    let shareUp: Bool

    private var title: String {
      if let nextName { "Move to \(nextName)" } else { "Finish exercise" }
    }

    private var spoken: String {
      if let nextName {
        "Move to \(nextName)"
      } else {
        "Finish exercise, then \(KeyShareSketch.nextItem)"
      }
    }

    var body: some View {
      VStack(spacing: IntradaSpacing.cardCompact) {
        TransportButton(systemImage: "pause.fill", prominence: .primary, label: "Pause") {}
        VStack(spacing: 0) {
          Button {
          } label: {
            HStack(spacing: IntradaSpacing.small) {
              if shareUp {
                Circle()
                  .fill(IntradaColor.ink)
                  .frame(width: IntradaSpacing.small, height: IntradaSpacing.small)
                  .accessibilityHidden(true)
              }
              Text(title)
                .font(shareUp ? IntradaFont.label.weight(.semibold) : IntradaFont.secondary)
            }
            .foregroundStyle(shareUp ? IntradaColor.ink : IntradaColor.inkSecondary)
            .frame(minHeight: 44)
            .contentShape(Rectangle())
          }
          .buttonStyle(PressRebound())
          .accessibilityLabel(spoken)
          if nextName == nil {
            Text("Then \(KeyShareSketch.nextItem)")
              .font(IntradaFont.small)
              .foregroundStyle(IntradaColor.inkSecondary)
              .accessibilityHidden(true)
          }
        }
      }
      .padding(.bottom, IntradaSpacing.card)
    }
  }

  /// A copy of `SessionOrientationBand` with the split strip in place of `SegmentedProgress`.
  struct SketchOrientationBand: View {
    let count: KeyShareSketch.Count
    let state: KeyShareSketch.State

    private var showsCount: Bool { count == .band && state.counted }

    var body: some View {
      VStack(spacing: IntradaSpacing.cardCompact) {
        HStack(spacing: IntradaSpacing.controlGap) {
          Text("17:12")
            .font(IntradaFont.figure)
            .monospacedDigit()
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(minWidth: 56, alignment: .leading)
          Text("Focus · 2 of 3")
            .font(IntradaFont.badge)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity)
          Image(systemName: "ellipsis")
            .iconSize(.control)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(width: 28, height: 28)
            .frame(minWidth: 56, alignment: .trailing)
        }
        VStack(spacing: IntradaSpacing.xSmall) {
          if showsCount { countRow }
          SketchSplitProgress(
            items: KeyShareSketch.items, current: KeyShareSketch.currentItem, parts: state.parts)
        }
      }
    }

    private var countRow: some View {
      HStack(spacing: IntradaSpacing.controlGap) {
        ForEach(0..<KeyShareSketch.items, id: \.self) { index in
          if index == KeyShareSketch.currentItem {
            Text("\(state.kind.word) \(state.countText)")
              .font(IntradaFont.small)
              .foregroundStyle(IntradaColor.inkSecondary)
              .lineLimit(1)
              .fixedSize()
              .frame(maxWidth: .infinity, alignment: .trailing)
          } else {
            Color.clear.frame(maxWidth: .infinity, maxHeight: 0)
          }
        }
      }
      .accessibilityHidden(true)
    }
  }

  /// The session strip, with the current item's segment split into `parts`, each
  /// a fraction from 0 to 1, 2 points apart against 8 between items.
  struct SketchSplitProgress: View {
    let items: Int
    let current: Int
    let parts: [Double]
    var height: CGFloat = 4

    var body: some View {
      HStack(spacing: IntradaSpacing.controlGap) {
        ForEach(0..<items, id: \.self) { index in
          if index == current && parts.count > 1 {
            HStack(spacing: IntradaSpacing.xxSmall) {
              ForEach(Array(parts.enumerated()), id: \.offset) { _, fraction in
                part(fraction).clipShape(Capsule())
              }
            }
          } else {
            Capsule().fill(index <= current ? IntradaColor.ink : IntradaColor.divider)
          }
        }
      }
      .frame(height: height)
      .accessibilityHidden(true)
    }

    private func part(_ fraction: Double) -> some View {
      GeometryReader { geo in
        ZStack(alignment: .leading) {
          Rectangle().fill(IntradaColor.divider)
          Rectangle()
            .fill(IntradaColor.ink)
            .frame(width: geo.size.width * min(max(fraction, 0), 1))
        }
      }
    }
  }

  /// A copy of the player's private key chip, so the sketch sits in the real layout.
  struct SketchKeyChip: View {
    let label: String
    var count: String?
    let spoken: String

    var body: some View {
      HStack(spacing: IntradaSpacing.controlGap) {
        HStack(spacing: 0) {
          Text(label)
            .foregroundStyle(IntradaColor.ink)
          if let count {
            Text(" · \(count)")
              .foregroundStyle(IntradaColor.inkSecondary)
          }
        }
        .font(IntradaFont.segment)
        Image(systemName: "chevron.down")
          .font(IntradaFont.small.weight(.semibold))
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .frame(minHeight: 44)
      .background(IntradaColor.cardFill, in: Capsule())
      .overlay(Capsule().stroke(IntradaColor.hairline, lineWidth: 1))
      .cardShadow()
      .accessibilityElement(children: .ignore)
      .accessibilityLabel(spoken)
      .accessibilityAddTraits(.isButton)
    }
  }

  /// A copy of the player's private `TimerRing` at its usual 200 points.
  struct SketchTimerRing: View {
    let elapsed: Int
    let planned: Int

    private var fraction: Double {
      guard planned > 0 else { return 0 }
      return min(Double(elapsed) / Double(planned), 1)
    }

    var body: some View {
      ZStack {
        ZStack {
          Circle().stroke(IntradaColor.track, lineWidth: 10)
          Circle()
            .trim(from: 0, to: fraction)
            .stroke(IntradaColor.ink, style: StrokeStyle(lineWidth: 10, lineCap: .round))
            .rotationEffect(.degrees(-90))
        }
        .padding(IntradaSpacing.card + IntradaSpacing.xxSmall)
        VStack(spacing: IntradaSpacing.xSmall) {
          Text(SessionClock.clockDisplay(elapsed))
            .font(IntradaFont.timer(48))
            .monospacedDigit()
            .foregroundStyle(IntradaColor.ink)
          Text("of \(SessionClock.clockDisplay(planned))")
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(width: 200, height: 200)
    }
  }

  #Preview("Count in the chip, share up") {
    KeySharePlayerSketch(count: .chip, state: KeyShareSketch.over)
  }
#endif
