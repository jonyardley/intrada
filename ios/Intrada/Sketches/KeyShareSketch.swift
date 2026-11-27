#if DEBUG
  import SharedTypes
  import SwiftUI

  struct KeyShareSketch {
    enum Look { case card, inline }

    struct State {
      let title: String
      let keyLabel: String
      let keyChoices: Int
      let itemElapsed: Int
      let itemPlanned: Int
      let shares: [Double]
      let shareLine: String
      let shareSide: String
      let next: String?
    }

    static let filling = State(
      title: "Major scales", keyLabel: "C major", keyChoices: 3,
      itemElapsed: 110, itemPlanned: 540, shares: [110.0 / 180, 0, 0],
      shareLine: "C major, 1:50 of 3:00", shareSide: "3 minutes each", next: nil)

    static let over = State(
      title: "Major scales", keyLabel: "C major", keyChoices: 3,
      itemElapsed: 185, itemPlanned: 540, shares: [1, 0, 0],
      shareLine: "C major, 3:05 of 3:00", shareSide: "Still timing C", next: "Next: G major")

    static let lastOver = State(
      title: "Major scales", keyLabel: "D major", keyChoices: 3,
      itemElapsed: 550, itemPlanned: 540, shares: [1, 1, 1],
      shareLine: "D major, 3:10 of 3:00", shareSide: "Still timing D", next: nil)

    static let oneKey = State(
      title: "Major scales", keyLabel: "C major", keyChoices: 1,
      itemElapsed: 110, itemPlanned: 540, shares: [],
      shareLine: "", shareSide: "", next: nil)

    static let longest = State(
      title: "Scales in thirds and sixths", keyLabel: "B♭ melodic minor", keyChoices: 2,
      itemElapsed: 605, itemPlanned: 1200, shares: [1, 0],
      shareLine: "B♭ melodic minor, 10:05 of 10:00", shareSide: "Still timing B♭",
      next: "Next: E♭ melodic minor")
  }

  // ── The player, drawn from its public parts with fixture values ──

  struct KeySharePlayerSketch: View {
    var look: KeyShareSketch.Look = .card
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
          SketchTimerRing(elapsed: state.itemElapsed, planned: state.itemPlanned)
            .padding(.top, IntradaSpacing.section)
          if state.shares.count > 1 {
            shareBar.padding(.top, IntradaSpacing.cardCompact)
            if look == .card {
              nextCard
                .padding(.top, IntradaSpacing.cardCompact)
                .opacity(state.next == nil ? 0 : 1)
                .accessibilityHidden(state.next == nil)
            }
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
        Text(state.title)
          .font(IntradaFont.pageTitle)
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
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

    private var shareBar: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.small) {
        SketchShareStrip(shares: state.shares, height: look == .card ? 6 : 4)
        ViewThatFits(in: .horizontal) {
          HStack {
            shareLineText
            Spacer(minLength: IntradaSpacing.controlGap)
            shareSide
          }
          VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
            shareLineText
            shareSide
          }
        }
      }
    }

    private var shareLineText: some View {
      Text(state.shareLine)
        .font(IntradaFont.secondary)
        .monospacedDigit()
        .foregroundStyle(IntradaColor.ink)
    }

    @ViewBuilder private var shareSide: some View {
      if look == .inline, let next = state.next {
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
      } else {
        Text(state.shareSide)
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }

    private var nextCard: some View {
      Button {
      } label: {
        HStack(spacing: IntradaSpacing.cardCompact) {
          Text(state.next ?? " ")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.ink)
            .frame(maxWidth: .infinity, alignment: .leading)
          Image(systemName: "arrow.right")
            .iconSize(.inline, weight: .semibold)
            .foregroundStyle(IntradaColor.ink)
            .accessibilityHidden(true)
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.vertical, IntradaSpacing.cardCompact)
        .frame(minHeight: 48)
        .fixedSize(horizontal: false, vertical: true)
        .cardSurface()
        .contentShape(Rectangle())
      }
      .buttonStyle(PressRebound())
      .accessibilityHint("Moves on to the next key")
    }

    // ── Bottom: the transport ──

    private var controls: some View {
      VStack(spacing: IntradaSpacing.cardCompact) {
        HStack(spacing: IntradaSpacing.section + IntradaSpacing.controlGap) {
          TransportButton(systemImage: "play.fill", prominence: .primary, label: "Next item") {}
          TransportButton(
            systemImage: "forward.end", prominence: .secondary, label: "Skip this item"
          ) {}
        }
        Text("Then Bach Prelude in C")
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .padding(.bottom, IntradaSpacing.card)
    }
  }

  /// One segment per key, the current one part filled: a time, unlike
  /// `SegmentedProgress`, which only steps.
  struct SketchShareStrip: View {
    let shares: [Double]
    let height: CGFloat

    var body: some View {
      HStack(spacing: IntradaSpacing.segmentGap) {
        ForEach(Array(shares.enumerated()), id: \.offset) { _, fraction in
          GeometryReader { geo in
            ZStack(alignment: .leading) {
              Capsule().fill(IntradaColor.track)
              Capsule()
                .fill(IntradaColor.ink)
                .frame(width: geo.size.width * min(max(fraction, 0), 1))
            }
          }
          .frame(height: height)
          .clipShape(Capsule())
        }
      }
      .frame(height: height)
      .accessibilityHidden(true)
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
    let planned: Int

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

  #Preview("Inline, over") {
    KeySharePlayerSketch(look: .inline, state: KeyShareSketch.over)
  }
#endif
