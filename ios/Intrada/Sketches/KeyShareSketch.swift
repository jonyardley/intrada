#if DEBUG
  import SharedTypes
  import SwiftUI

  struct KeyShareSketch {
    enum Mark: CaseIterable { case gaps, ticks, track }

    struct State {
      let keyLabel: String
      let keyChoices: Int
      let itemElapsed: Int
      let itemPlanned: Int?
      let parts: [Double]
      let partLine: String?
      let partSide: String?
      let next: String?
    }

    static let filling = State(
      keyLabel: "C major", keyChoices: 3, itemElapsed: 110, itemPlanned: 540,
      parts: [110.0 / 180, 0, 0], partLine: "C major, 1:50 of 3:00",
      partSide: "3 minutes each", next: nil)

    static let over = State(
      keyLabel: "C major", keyChoices: 3, itemElapsed: 185, itemPlanned: 540,
      parts: [1, 0, 0], partLine: "C major, 3:05 of 3:00",
      partSide: nil, next: "Next · G major")

    static let lastOver = State(
      keyLabel: "D major", keyChoices: 3, itemElapsed: 550, itemPlanned: 540,
      parts: [1, 1, 1], partLine: "D major, 3:10 of 3:00",
      partSide: "Still timing D", next: nil)

    static let oneKey = State(
      keyLabel: "C major", keyChoices: 1, itemElapsed: 110, itemPlanned: 540,
      parts: [], partLine: nil, partSide: nil, next: nil)

    static let untimed = State(
      keyLabel: "C major", keyChoices: 3, itemElapsed: 110, itemPlanned: nil,
      parts: [], partLine: nil, partSide: nil, next: nil)

    static let items = 3
    static let currentItem = 1
  }

  // ── The player, drawn from its public parts with fixture values ──

  struct KeySharePlayerSketch: View {
    var mark: KeyShareSketch.Mark = .gaps
    var state: KeyShareSketch.State = KeyShareSketch.filling

    var body: some View {
      ZStack {
        IntradaColor.paperTop.ignoresSafeArea()
        VStack(spacing: 0) {
          SketchOrientationBand(mark: mark, parts: state.parts)
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
          controls
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

    // ── The row under the ring ──

    private func partRow(_ line: String) -> some View {
      HStack {
        Text(line)
          .font(IntradaFont.secondary)
          .monospacedDigit()
          .foregroundStyle(IntradaColor.ink)
        Spacer(minLength: IntradaSpacing.controlGap)
        if let next = state.next {
          Button {
          } label: {
            HStack(spacing: IntradaSpacing.xSmall) {
              Text(next)
                .font(IntradaFont.label)
              Image(systemName: "arrow.right")
                .iconSize(.inline, weight: .semibold)
                .accessibilityHidden(true)
            }
            .foregroundStyle(IntradaColor.ink)
            .frame(minHeight: 44)
            .contentShape(Rectangle())
          }
          .buttonStyle(PressRebound())
          .accessibilityHint("Moves on to the next key")
        } else if let side = state.partSide {
          Text(side)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(minHeight: 44)
    }

    // ── Bottom: the transport, as shipped ──

    private var controls: some View {
      VStack(spacing: IntradaSpacing.cardCompact) {
        HStack(spacing: IntradaSpacing.section + IntradaSpacing.controlGap) {
          TransportButton(systemImage: "play.fill", prominence: .primary, label: "Next item") {}
          TransportButton(
            systemImage: "forward.end", prominence: .secondary, label: "Skip this item"
          ) {}
        }
        Text("Next · Bach Prelude in C")
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .padding(.bottom, IntradaSpacing.card)
    }
  }

  /// A copy of `SessionOrientationBand` with the split strip in place of `SegmentedProgress`.
  struct SketchOrientationBand: View {
    let mark: KeyShareSketch.Mark
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
          items: KeyShareSketch.items, current: KeyShareSketch.currentItem, parts: parts,
          mark: mark)
      }
    }
  }

  /// The session strip, with the current item's segment split into `parts`, each
  /// a fraction from 0 to 1. Fewer than two parts draws the strip as shipped.
  struct SketchSplitProgress: View {
    let items: Int
    let current: Int
    let parts: [Double]
    let mark: KeyShareSketch.Mark
    var height: CGFloat = 4

    var body: some View {
      HStack(spacing: IntradaSpacing.segmentGap) {
        ForEach(0..<items, id: \.self) { index in
          if index == current && parts.count > 1 {
            split
          } else {
            Capsule().fill(index <= current ? IntradaColor.ink : IntradaColor.divider)
          }
        }
      }
      .frame(height: height)
      .accessibilityHidden(true)
    }

    @ViewBuilder private var split: some View {
      switch mark {
      case .gaps:
        HStack(spacing: IntradaSpacing.xxSmall) {
          ForEach(Array(parts.enumerated()), id: \.offset) { _, fraction in
            part(fraction, empty: IntradaColor.divider).clipShape(Capsule())
          }
        }
      case .ticks:
        GeometryReader { geo in
          let width = geo.size.width / CGFloat(parts.count)
          ZStack(alignment: .leading) {
            Capsule().fill(IntradaColor.divider)
            ForEach(Array(parts.enumerated()), id: \.offset) { index, fraction in
              Rectangle()
                .fill(IntradaColor.ink)
                .frame(width: width * min(max(fraction, 0), 1))
                .offset(x: width * CGFloat(index))
            }
            ForEach(1..<parts.count, id: \.self) { index in
              Rectangle()
                .fill(IntradaColor.paperTop)
                .frame(width: 1)
                .offset(x: width * CGFloat(index))
            }
          }
          .clipShape(Capsule())
        }
      case .track:
        let currentPart = parts.firstIndex { $0 < 1 } ?? parts.count - 1
        HStack(spacing: 0) {
          ForEach(Array(parts.enumerated()), id: \.offset) { index, fraction in
            part(
              fraction, empty: index == currentPart ? IntradaColor.divider : IntradaColor.track)
          }
        }
        .clipShape(Capsule())
      }
    }

    private func part(_ fraction: Double, empty: Color) -> some View {
      GeometryReader { geo in
        ZStack(alignment: .leading) {
          Rectangle().fill(empty)
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

  /// The three marks side by side at a readable size, for the review page.
  struct KeyShareStripMarksSketch: View {
    private let rows: [(String, [Double])] = [
      ("Filling", KeyShareSketch.filling.parts), ("Over", KeyShareSketch.over.parts),
      ("Last key over", KeyShareSketch.lastOver.parts),
    ]
    private let marks: [(String, KeyShareSketch.Mark)] = [
      ("Gaps", .gaps), ("Ticks", .ticks), ("Lighter track", .track),
    ]

    var body: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        ForEach(marks, id: \.0) { name, mark in
          VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
            Text(name)
              .font(IntradaFont.badge)
              .foregroundStyle(IntradaColor.ink)
            ForEach(rows, id: \.0) { label, parts in
              HStack(spacing: IntradaSpacing.cardCompact) {
                Text(label)
                  .font(IntradaFont.small)
                  .foregroundStyle(IntradaColor.inkSecondary)
                  .frame(width: 90, alignment: .leading)
                SketchSplitProgress(
                  items: KeyShareSketch.items, current: KeyShareSketch.currentItem,
                  parts: parts, mark: mark)
              }
            }
          }
        }
      }
      .frame(width: 342)
      .padding(IntradaSpacing.card)
      .background(IntradaColor.paperTop)
    }
  }

  #Preview("Gaps, over") {
    KeySharePlayerSketch(mark: .gaps, state: KeyShareSketch.over)
  }
#endif
