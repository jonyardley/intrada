#if DEBUG
  import SharedTypes
  import SwiftUI

  struct KeyShareSketch {
    enum Look: CaseIterable { case hint, beside, ring }

    enum Move {
      case nextKey(name: String, short: String)
      case finish
    }

    struct State {
      let keyLabel: String
      let keyChoices: Int
      let itemElapsed: Int
      let itemPlanned: Int?
      let parts: [Double]
      let partLine: String?
      let partSide: String?
      let move: Move
      let shareUp: Bool
    }

    static let filling = State(
      keyLabel: "C major", keyChoices: 3, itemElapsed: 110, itemPlanned: 540,
      parts: [110.0 / 180, 0, 0], partLine: "C major, 1:50 of 3:00",
      partSide: "3 minutes each", move: .nextKey(name: "G major", short: "G"), shareUp: false)

    static let over = State(
      keyLabel: "C major", keyChoices: 3, itemElapsed: 185, itemPlanned: 540,
      parts: [1, 0, 0], partLine: "C major, 3:05 of 3:00",
      partSide: nil, move: .nextKey(name: "G major", short: "G"), shareUp: true)

    static let lastKey = State(
      keyLabel: "D major", keyChoices: 3, itemElapsed: 432, itemPlanned: 540,
      parts: [1, 1, 72.0 / 180], partLine: "D major, 1:12 of 3:00",
      partSide: "Last key", move: .finish, shareUp: false)

    static let oneKey = State(
      keyLabel: "C major", keyChoices: 1, itemElapsed: 110, itemPlanned: 540,
      parts: [], partLine: nil, partSide: nil, move: .finish, shareUp: false)

    static let untimed = State(
      keyLabel: "C major", keyChoices: 3, itemElapsed: 110, itemPlanned: nil,
      parts: [], partLine: nil, partSide: nil,
      move: .nextKey(name: "G major", short: "G"), shareUp: false)

    static let items = 3
    static let currentItem = 1
    static let nextItem = "Bach Prelude in C"
  }

  // ── The player, drawn from its public parts with fixture values ──

  struct KeySharePlayerSketch: View {
    var look: KeyShareSketch.Look = .hint
    var state: KeyShareSketch.State = KeyShareSketch.filling

    var body: some View {
      ZStack {
        IntradaColor.paperTop.ignoresSafeArea()
        VStack(spacing: 0) {
          SketchOrientationBand(parts: state.parts)
          Spacer(minLength: IntradaSpacing.card)
          centerInfo
          SketchTimerRing(elapsed: state.itemElapsed, planned: state.itemPlanned)
            .padding(.top, IntradaSpacing.section)
          if let line = state.partLine {
            partRow(line).padding(.top, IntradaSpacing.cardCompact)
          }
          ClickControl(
            bpm: 96, step: 2, band: 40...208, isRunning: false, unavailable: false,
            atSeededTempo: true, targetDisplay: nil, targetSpoken: nil,
            onToggle: {}, onStep: { _ in }, onDragChange: { _ in }
          )
          .padding(.top, IntradaSpacing.controlGap)
          Spacer(minLength: IntradaSpacing.card)
          SketchMoveOnControls(look: look, move: state.move, shareUp: state.shareUp)
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
      }
    }

    private var centerInfo: some View {
      VStack(spacing: IntradaSpacing.controlGap) {
        TypeBadge(kind: .exercise)
        Text("Major scales")
          .font(IntradaFont.pageTitle)
          .foregroundStyle(IntradaColor.ink)
        if state.keyChoices > 1 {
          SketchKeyChip(label: state.keyLabel)
        } else {
          Text(state.keyLabel)
            .font(IntradaFont.segment)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
    }

    private func partRow(_ line: String) -> some View {
      HStack {
        Text(line)
          .font(IntradaFont.secondary)
          .monospacedDigit()
          .foregroundStyle(IntradaColor.ink)
        Spacer(minLength: IntradaSpacing.controlGap)
        if let side = state.partSide {
          Text(side)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(minHeight: 44)
    }
  }

  // ── Bottom: pause stays the big ink circle, moving on is quieter ──

  struct SketchMoveOnControls: View {
    let look: KeyShareSketch.Look
    let move: KeyShareSketch.Move
    let shareUp: Bool

    private var tint: Color { shareUp ? IntradaColor.ink : IntradaColor.inkSecondary }

    private var title: String {
      switch move {
      case .nextKey(let name, _): "Move to \(name)"
      case .finish: "Finish exercise"
      }
    }

    private var spoken: String {
      switch move {
      case .nextKey(let name, _): "Move to \(name)"
      case .finish: "Finish exercise, then \(KeyShareSketch.nextItem)"
      }
    }

    var body: some View {
      VStack(spacing: IntradaSpacing.cardCompact) {
        switch look {
        case .hint: hintLook
        case .beside: besideLook
        case .ring: ringLook
        }
      }
      .padding(.bottom, IntradaSpacing.card)
    }

    private var pause: some View {
      TransportButton(systemImage: "pause.fill", prominence: .primary, label: "Pause") {}
    }

    private var afterLine: some View {
      Text("Then \(KeyShareSketch.nextItem)")
        .font(IntradaFont.small)
        .foregroundStyle(IntradaColor.inkSecondary)
    }

    // Look A: the hint line under pause becomes the control.
    @ViewBuilder private var hintLook: some View {
      pause
      Button {
      } label: {
        HStack(spacing: IntradaSpacing.small) {
          if case .finish = move {
            Image(systemName: "checkmark")
              .iconSize(.inline, weight: shareUp ? .semibold : .regular)
              .accessibilityHidden(true)
          }
          Text(title)
            .font(shareUp ? IntradaFont.label : IntradaFont.secondary)
        }
        .foregroundStyle(tint)
        .frame(minHeight: 44)
        .contentShape(Rectangle())
      }
      .buttonStyle(PressRebound())
      .accessibilityLabel(spoken)
      if case .finish = move {
        afterLine
      }
    }

    // Look B: a two line text control left of pause, away from where skip sits today.
    @ViewBuilder private var besideLook: some View {
      HStack(spacing: 0) {
        HStack(spacing: 0) {
          Spacer(minLength: 0)
          Button {
          } label: {
            VStack(alignment: .trailing, spacing: IntradaSpacing.xxSmall) {
              Text(besideCaption)
                .font(IntradaFont.small)
                .foregroundStyle(IntradaColor.inkSecondary)
              Text(besideName)
                .font(shareUp ? IntradaFont.label : IntradaFont.secondary)
                .foregroundStyle(tint)
            }
            .multilineTextAlignment(.trailing)
            .frame(minHeight: 44)
            .contentShape(Rectangle())
          }
          .buttonStyle(PressRebound())
          .accessibilityLabel(spoken)
        }
        .frame(maxWidth: .infinity)
        .padding(.trailing, IntradaSpacing.section)
        pause
        Color.clear.frame(maxWidth: .infinity, maxHeight: 1)
          .padding(.leading, IntradaSpacing.section)
      }
      if case .finish = move {
        afterLine
      }
    }

    private var besideCaption: String {
      switch move {
      case .nextKey: "Next key"
      case .finish: "Last key"
      }
    }

    private var besideName: String {
      switch move {
      case .nextKey(let name, _): name
      case .finish: "Finish exercise"
      }
    }

    // Look C: a thin ring beside pause, as a stepper's sign (T39), named underneath.
    @ViewBuilder private var ringLook: some View {
      HStack(spacing: IntradaSpacing.section + IntradaSpacing.controlGap) {
        Color.clear.frame(width: 48, height: 1)
        pause
        Button {
        } label: {
          ZStack {
            Circle()
              .stroke(shareUp ? IntradaColor.ink : IntradaColor.divider, lineWidth: 1)
            switch move {
            case .nextKey(_, let short):
              Text(short)
                .font(IntradaFont.label)
                .foregroundStyle(tint)
            case .finish:
              Image(systemName: "checkmark")
                .iconSize(.inline, weight: .medium)
                .foregroundStyle(tint)
            }
          }
          .frame(width: 40, height: 40)
          .frame(width: 48, height: 48)
          .contentShape(Rectangle())
        }
        .buttonStyle(PressRebound())
        .accessibilityLabel(spoken)
      }
      Text(ringLine)
        .font(IntradaFont.secondary)
        .foregroundStyle(IntradaColor.inkSecondary)
    }

    private var ringLine: String {
      switch move {
      case .nextKey(let name, _): "Next key · \(name)"
      case .finish: "Next · \(KeyShareSketch.nextItem)"
      }
    }
  }

  /// A copy of `SessionOrientationBand` with the split strip in place of `SegmentedProgress`.
  struct SketchOrientationBand: View {
    let parts: [Double]

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
        SketchSplitProgress(
          items: KeyShareSketch.items, current: KeyShareSketch.currentItem, parts: parts)
      }
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

    var body: some View {
      HStack(spacing: IntradaSpacing.controlGap) {
        Text(label)
          .font(IntradaFont.segment)
          .foregroundStyle(IntradaColor.ink)
        Image(systemName: "chevron.down")
          .font(IntradaFont.small.weight(.semibold))
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .frame(minHeight: 44)
      .background(IntradaColor.cardFill, in: Capsule())
      .overlay(Capsule().stroke(IntradaColor.hairline, lineWidth: 1))
      .cardShadow()
    }
  }

  /// A copy of the player's private `TimerRing` at its usual 200 points.
  struct SketchTimerRing: View {
    let elapsed: Int
    let planned: Int?

    private var fraction: Double {
      guard let planned, planned > 0 else { return 0 }
      return min(Double(elapsed) / Double(planned), 1)
    }

    var body: some View {
      ZStack {
        ZStack {
          Circle().stroke(IntradaColor.track, lineWidth: 10)
          if planned != nil {
            Circle()
              .trim(from: 0, to: fraction)
              .stroke(IntradaColor.ink, style: StrokeStyle(lineWidth: 10, lineCap: .round))
              .rotationEffect(.degrees(-90))
          }
        }
        .padding(IntradaSpacing.card + IntradaSpacing.xxSmall)
        VStack(spacing: IntradaSpacing.xSmall) {
          Text(SessionClock.clockDisplay(elapsed))
            .font(IntradaFont.timer(48))
            .monospacedDigit()
            .foregroundStyle(IntradaColor.ink)
          if let planned {
            Text("of \(SessionClock.clockDisplay(planned))")
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
        }
      }
      .frame(width: 200, height: 200)
    }
  }

  /// The strip at a readable size, so the key and item gaps can be compared.
  struct KeyShareStripSketch: View {
    private let rows: [(String, [Double])] = [
      ("Filling", KeyShareSketch.filling.parts), ("Share up", KeyShareSketch.over.parts),
      ("Last key", KeyShareSketch.lastKey.parts),
    ]

    var body: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        ForEach(rows, id: \.0) { label, parts in
          HStack(spacing: IntradaSpacing.cardCompact) {
            Text(label)
              .font(IntradaFont.small)
              .foregroundStyle(IntradaColor.inkSecondary)
              .frame(width: 90, alignment: .leading)
            SketchSplitProgress(
              items: KeyShareSketch.items, current: KeyShareSketch.currentItem, parts: parts)
          }
        }
      }
      .frame(width: 342)
      .padding(IntradaSpacing.card)
      .background(IntradaColor.paperTop)
    }
  }

  #Preview("Hint, share up") {
    KeySharePlayerSketch(look: .hint, state: KeyShareSketch.over)
  }
#endif
