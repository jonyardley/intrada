#if DEBUG
  import SharedTypes
  import SwiftUI

  struct KeyShareSketch {
    enum Look { case canvas, stepped, quiet }

    struct Share {
      let key: String
      let fraction: Double
    }

    struct State {
      let keyLabel: String
      let keyChoices: Int
      let itemElapsed: Int
      let itemPlanned: Int
      let shares: [Share]
      let current: Int
      let shareLine: String
      let shareSide: String
      let next: String?
      let nextLine: String?
      let isOver: Bool
    }

    static let filling = State(
      keyLabel: "C major", keyChoices: 3, itemElapsed: 110, itemPlanned: 540,
      shares: [
        Share(key: "C major", fraction: 110.0 / 180), Share(key: "G major", fraction: 0),
        Share(key: "D major", fraction: 0),
      ],
      current: 0, shareLine: "C major, 1:50 of 3:00", shareSide: "3 min each",
      next: nil, nextLine: nil, isOver: false)

    static let over = State(
      keyLabel: "C major", keyChoices: 3, itemElapsed: 185, itemPlanned: 540,
      shares: [
        Share(key: "C major", fraction: 1), Share(key: "G major", fraction: 0),
        Share(key: "D major", fraction: 0),
      ],
      current: 0, shareLine: "C major, 3:05 of 3:00", shareSide: "Still timing C",
      next: "Next: G major", nextLine: "C major's share is up. Tap to move on.", isOver: true)

    static let lastOver = State(
      keyLabel: "D major", keyChoices: 3, itemElapsed: 550, itemPlanned: 540,
      shares: [
        Share(key: "C major", fraction: 1), Share(key: "G major", fraction: 1),
        Share(key: "D major", fraction: 1),
      ],
      current: 2, shareLine: "D major, 3:10 of 3:00", shareSide: "Still timing D",
      next: nil, nextLine: nil, isOver: true)

    static let oneKey = State(
      keyLabel: "C major", keyChoices: 1, itemElapsed: 110, itemPlanned: 540,
      shares: [], current: 0, shareLine: "", shareSide: "",
      next: nil, nextLine: nil, isOver: false)

    static let longest = State(
      keyLabel: "F sharp major", keyChoices: 2, itemElapsed: 605, itemPlanned: 1200,
      shares: [Share(key: "F sharp major", fraction: 1), Share(key: "B major", fraction: 0)],
      current: 0, shareLine: "F sharp major, 10:05 of 10:00", shareSide: "Still timing F sharp",
      next: "Next: B major", nextLine: "F sharp major's share is up. Tap to move on.",
      isOver: true)
  }

  // ── The player, drawn from its public parts with fixture values ──

  struct KeySharePlayerSketch: View {
    var look: KeyShareSketch.Look = .canvas
    var state: KeyShareSketch.State = KeyShareSketch.filling

    var body: some View {
      ZStack {
        IntradaColor.paperTop.ignoresSafeArea()
        VStack(spacing: 0) {
          SessionOrientationBand(
            session: .remaining(1032), positionLabel: "Focus · 1 of 3",
            types: [.exercise, .piece, .exercise], filled: 1,
            menu: {
              Image(systemName: "ellipsis")
                .iconSize(.control)
                .foregroundStyle(IntradaColor.inkSecondary)
                .frame(width: 28, height: 28)
            })
          Spacer(minLength: IntradaSpacing.card)
          centerInfo
          SketchTimerRing(
            elapsed: state.itemElapsed, planned: state.itemPlanned,
            size: state.next == nil ? 200 : 180
          )
          .padding(.top, IntradaSpacing.section)
          if state.shares.count > 1 {
            shareBar.padding(.top, IntradaSpacing.cardCompact)
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

    // ── The share bar ──

    @ViewBuilder private var shareBar: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.small) {
        switch look {
        case .canvas:
          SketchShareStrip(shares: state.shares, height: 6)
        case .stepped:
          SegmentedProgress(
            count: state.shares.count, filled: state.current + 1, label: state.shareLine)
        case .quiet:
          SketchShareStrip(shares: state.shares, height: 4)
        }
        ViewThatFits(in: .horizontal) {
          HStack {
            shareLineText
            Spacer(minLength: IntradaSpacing.controlGap)
            shareSideText
          }
          VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
            shareLineText
            shareSideText
          }
        }
      }
      .accessibilityElement(children: .combine)
    }

    private var shareLineText: some View {
      Text(state.shareLine)
        .font(IntradaFont.small)
        .monospacedDigit()
        .foregroundStyle(IntradaColor.ink)
    }

    private var shareSideText: some View {
      Text(state.shareSide)
        .font(IntradaFont.small)
        .foregroundStyle(IntradaColor.inkSecondary)
    }

    // ── Bottom: the Next card, then the transport ──

    private var controls: some View {
      VStack(spacing: IntradaSpacing.cardCompact) {
        if let next = state.next {
          nextCard(next)
            .padding(.bottom, IntradaSpacing.controlGap)
        }
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

    @ViewBuilder private func nextCard(_ next: String) -> some View {
      Button {
      } label: {
        HStack(spacing: IntradaSpacing.cardCompact) {
          switch look {
          case .canvas, .stepped:
            VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
              Text(next)
                .font(IntradaFont.cardTitle)
                .foregroundStyle(IntradaColor.ink)
              if let line = state.nextLine {
                Text(line)
                  .font(IntradaFont.small)
                  .foregroundStyle(IntradaColor.inkSecondary)
                  .fixedSize(horizontal: false, vertical: true)
              }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
          case .quiet:
            Text(next)
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.ink)
              .frame(maxWidth: .infinity, alignment: .leading)
          }
          Image(systemName: "arrow.right")
            .iconSize(.control)
            .foregroundStyle(IntradaColor.ink)
            .accessibilityHidden(true)
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.vertical, look == .quiet ? IntradaSpacing.cardCompact : IntradaSpacing.controlGap)
        .frame(minHeight: look == .quiet ? 48 : 56)
        .fixedSize(horizontal: false, vertical: true)
        .cardSurface()
        .cardShadow()
        .contentShape(Rectangle())
      }
      .buttonStyle(PressRebound())
      .accessibilityElement(children: .combine)
      .accessibilityHint("Moves on to the next key")
    }
  }

  /// One segment per key, the current one part filled: a time, unlike
  /// `SegmentedProgress`, which only steps.
  struct SketchShareStrip: View {
    let shares: [KeyShareSketch.Share]
    let height: CGFloat

    var body: some View {
      HStack(spacing: IntradaSpacing.segmentGap) {
        ForEach(Array(shares.enumerated()), id: \.offset) { _, share in
          GeometryReader { geo in
            ZStack(alignment: .leading) {
              Capsule().fill(IntradaColor.track)
              Capsule()
                .fill(IntradaColor.ink)
                .frame(width: geo.size.width * min(max(share.fraction, 0), 1))
            }
          }
          .frame(height: height)
          .clipShape(Capsule())
        }
      }
      .frame(height: height)
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

  /// A copy of the player's private `TimerRing`.
  struct SketchTimerRing: View {
    let elapsed: Int
    let planned: Int
    let size: CGFloat

    private var fraction: Double {
      planned > 0 ? min(Double(elapsed) / Double(planned), 1) : 0
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
            .font(IntradaFont.timer(size < 200 ? 40 : 48))
            .monospacedDigit()
            .foregroundStyle(IntradaColor.ink)
          Text("of \(SessionClock.clockDisplay(planned))")
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(width: size, height: size)
    }
  }

  #Preview("Canvas, over") {
    KeySharePlayerSketch(look: .canvas, state: KeyShareSketch.over)
  }
#endif
