import SharedTypes
import SwiftUI

/// Everything the drill screen draws, in one value. The coach engine's
/// `CoachView` fills it from Phase 2a (`specs/intrada-coach-engine.md` §4, §6);
/// no counting, gating or sequencing happens in Swift.
struct DrillLoopState: Equatable {
  enum Phase: Equatable {
    case playing
    case awaitingVerdict
    case acknowledged(clean: Bool, countInRemaining: Int)
    case gateOpen
  }

  var phase: Phase = .playing

  // Identity
  var drillTitle: String
  /// Where in the material it sits — "A section", "key of F".
  var section: String?
  /// The in-flight tune or campaign — "Strasbourg / St. Denis".
  var destination: String?
  var kind: ItemKind = .exercise

  // The click
  var tempoBpm: Int
  /// The running click level, in the musician's words: "beats 2 & 4".
  var clickLevel: String
  var beat: Int = 1
  var beatsPerBar: Int = 4
  var bar: Int = 1
  var bars: Int = 8
  var countInBeats: Int = 4

  // Orientation
  var elapsedSeconds: Int
  var ceilingSeconds: Int?
  var blockKinds: [ItemKind]
  var blockIndex: Int

  // The gate
  /// The criterion restated as the question: "Clean at 120?".
  var gateQuestion: String
  /// The criterion in the past tense, at the moment it is met: "3 clean at 120".
  var gateSummary: String
  var gateFilled: Int
  var gateTarget: Int
}

/// A2 (during play) and A3 (after a repetition) — one shell, so the eye lands
/// in the same place between reps. Rules in `design/Drill Loop.dc.html`.
struct DrillScreen: View {
  let state: DrillLoopState
  /// `true` = "Yes — clean", `false` = "No — missed it".
  var onVerdict: (Bool) -> Void
  var onStuck: () -> Void
  var onDismiss: () -> Void

  @Environment(\.horizontalSizeClass) private var sizeClass
  @Environment(\.dynamicTypeSize) private var typeSize

  private var scale: CoachScale { sizeClass == .regular ? .regular : .compact }
  private var gutter: CGFloat {
    scale == .compact ? IntradaSpacing.card : IntradaSpacing.stage
  }
  /// The chip and tune line give way first; drill, tempo, position and target
  /// survive at any size.
  private var showsIdentityDetail: Bool { !typeSize.isAccessibilitySize }

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: 0) {
        OrientationStrip(
          elapsedSeconds: state.elapsedSeconds, ceilingSeconds: state.ceilingSeconds,
          blockKinds: state.blockKinds, blockIndex: state.blockIndex,
          onDismiss: onDismiss)
        identity
        Spacer(minLength: IntradaSpacing.card)
        centre
        Spacer(minLength: IntradaSpacing.card)
        footer
      }
      .padding(.horizontal, gutter)
      .padding(.bottom, IntradaSpacing.section)
    }
    .environment(\.coachScale, scale)
    .dynamicTypeSize(.xSmall ... .accessibility5)
  }

  // ── Identity: which drill, and what it serves ──

  @ViewBuilder private var identity: some View {
    switch state.phase {
    case .playing:
      VStack(spacing: IntradaSpacing.cardCompact) {
        if showsIdentityDetail {
          TypeBadge(kind: state.kind, label: "Drill")
        }
        Text(state.drillTitle)
          .font(IntradaFont.drillTitle(scale.drillTitle))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
        if let serves, showsIdentityDetail {
          Label(serves, systemImage: "arrow.turn.down.right")
            .font(IntradaFont.ambient(scale == .compact ? 12 : 17))
            .foregroundStyle(IntradaColor.exerciseBadgeFg)
            .accessibilityLabel(spoken(serves))
        }
      }
      .padding(.top, IntradaSpacing.section)
    case .awaitingVerdict:
      VStack(spacing: IntradaSpacing.cardCompact) {
        if showsIdentityDetail {
          TypeBadge(kind: state.kind, label: "Drill")
        }
        Text(subtitleLine)
          .font(IntradaFont.ambient(scale == .compact ? 14 : 20))
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
          .accessibilityLabel(spoken(subtitleLine))
      }
      .padding(.top, IntradaSpacing.section)
    case .acknowledged, .gateOpen:
      // Deliberately bare: a one-second glance shouldn't carry anything to read.
      EmptyView()
    }
  }

  /// Most voices read "·" aloud as "middle dot".
  private func spoken(_ line: String) -> String {
    line.replacingOccurrences(of: " · ", with: ", ")
  }

  private var serves: String? {
    let parts = [state.section, state.destination].compactMap { $0 }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  /// On A3 the criterion is the headline, so the identity drops to one quiet
  /// line and the tune drops off it.
  private var subtitleLine: String {
    [state.drillTitle, state.section].compactMap { $0 }.joined(separator: " · ")
  }

  // ── Centre: the five facts, or the one glance ──

  @ViewBuilder private var centre: some View {
    switch state.phase {
    case .playing:
      VStack(spacing: IntradaSpacing.row + 2) {
        tempo
        clickPill
        BeatPosition(
          beat: state.beat, beatsPerBar: state.beatsPerBar, bar: state.bar, bars: state.bars
        )
        .padding(.top, IntradaSpacing.cardCompact)
      }
    case .awaitingVerdict:
      VStack(spacing: IntradaSpacing.section) {
        Text(state.gateQuestion)
          .font(IntradaFont.verdict(scale.question))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
        GateDots(filled: state.gateFilled, target: state.gateTarget)
      }
    case .acknowledged(let clean, _):
      VStack(spacing: IntradaSpacing.section + 2) {
        RepVerdict(outcome: clean ? .clean : .missed)
        GateDots(filled: state.gateFilled, target: state.gateTarget)
      }
    case .gateOpen:
      VStack(spacing: IntradaSpacing.section) {
        RepVerdict(outcome: .clean, fact: "Gate open")
        GateDots(
          filled: state.gateFilled, target: state.gateTarget, caption: state.gateSummary)
      }
    }
  }

  private var tempo: some View {
    HStack(alignment: .firstTextBaseline, spacing: scale == .compact ? 9 : 12) {
      Text("♩")
        .font(IntradaFont.drillTitle(scale.tempo * 0.88))
      Text("= \(state.tempoBpm)")
        .font(IntradaFont.drillTitle(scale.tempo))
        .monospacedDigit()
    }
    .foregroundStyle(IntradaColor.ink)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("\(state.tempoBpm) beats per minute")
  }

  private var clickPill: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      Text("CLICK")
        .font(IntradaFont.ambientStrong(scale == .compact ? 10 : 13))
        .tracking(0.8)
        .foregroundStyle(IntradaColor.exerciseBadgeFg)
      Text(state.clickLevel)
        .font(IntradaFont.ambientStrong(scale == .compact ? 13 : 19))
        .foregroundStyle(IntradaColor.ink)
    }
    .padding(.vertical, scale == .compact ? 6 : 10)
    .padding(.horizontal, scale == .compact ? 14 : 20)
    .background(IntradaColor.surfaceSunken, in: Capsule())
    .overlay(Capsule().strokeBorder(IntradaColor.divider, lineWidth: 1))
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Click on \(state.clickLevel)")
  }

  // ── Footer: the one target, or the tap-verdict ──

  @ViewBuilder private var footer: some View {
    switch state.phase {
    case .playing:
      StuckTarget(action: onStuck)
    case .awaitingVerdict:
      VStack(spacing: 0) {
        TapVerdict(onClean: { onVerdict(true) }, onMissed: { onVerdict(false) })
        HairlineDivider()
          .padding(.top, IntradaSpacing.row - 2)
          .padding(.bottom, IntradaSpacing.controlGap + 2)
        StuckTarget(emphasis: .quiet, action: onStuck)
      }
    case .acknowledged(_, let remaining):
      CountIn(remaining: remaining, total: state.countInBeats)
    case .gateOpen:
      Text("moving on")
        .font(IntradaFont.ambient())
        .foregroundStyle(IntradaColor.inkSecondary)
    }
  }
}

/// The beats between the tap and the next attempt — drawn, not counted down in
/// numerals, which would pull the eyes back as they should be leaving.
private struct CountIn: View {
  let remaining: Int
  let total: Int

  var body: some View {
    HStack(spacing: 9) {
      Text("count-in")
        .font(IntradaFont.ambient(13))
        .foregroundStyle(IntradaColor.inkSecondary)
      ForEach(0..<max(total, 0), id: \.self) { index in
        Circle()
          .fill(index < total - remaining ? IntradaColor.repMissedFg : Color.clear)
          .frame(width: 10, height: 10)
          .overlay(
            Circle().strokeBorder(
              index < total - remaining ? Color.clear : IntradaColor.slotOutline,
              lineWidth: 1.5))
      }
    }
    .animation(nil, value: remaining)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Count-in, \(remaining) beats to go")
  }
}

#if DEBUG
  extension DrillLoopState {
    /// The design's worked example: *Strasbourg / St. Denis*.
    static func preview(
      phase: Phase = .playing, gateFilled: Int = 2, elapsedSeconds: Int = 724
    ) -> DrillLoopState {
      DrillLoopState(
        phase: phase,
        drillTitle: "Rootless voicings",
        section: "A section",
        destination: "Strasbourg / St. Denis",
        tempoBpm: 120,
        clickLevel: "beats 2 & 4",
        beat: 2, bar: 3, bars: 8,
        elapsedSeconds: elapsedSeconds, ceilingSeconds: 360,
        blockKinds: [.piece, .exercise, .exercise, .piece, .piece], blockIndex: 1,
        gateQuestion: "Clean at 120?",
        gateSummary: "3 clean at 120",
        gateFilled: gateFilled, gateTarget: 3)
    }
  }

  private struct DrillPreview: View {
    let state: DrillLoopState
    var body: some View {
      DrillScreen(state: state, onVerdict: { _ in }, onStuck: {}, onDismiss: {})
    }
  }

  #Preview("A2 during play") { DrillPreview(state: .preview()) }

  #Preview("A2 largest accessibility size") {
    DrillPreview(state: .preview()).environment(\.dynamicTypeSize, .accessibility5)
  }

  #Preview("A3 tap-verdict") { DrillPreview(state: .preview(phase: .awaitingVerdict)) }

  #Preview("A3 first rep") {
    DrillPreview(state: .preview(phase: .awaitingVerdict, gateFilled: 0, elapsedSeconds: 192))
  }

  #Preview("A3 pass tap") {
    DrillPreview(
      state: .preview(
        phase: .acknowledged(clean: true, countInRemaining: 2), gateFilled: 3,
        elapsedSeconds: 761))
  }

  #Preview("A3 fail tap") {
    DrillPreview(
      state: .preview(
        phase: .acknowledged(clean: false, countInRemaining: 2), elapsedSeconds: 798))
  }

  #Preview("A3 gate open") {
    DrillPreview(state: .preview(phase: .gateOpen, gateFilled: 3, elapsedSeconds: 842))
  }
#endif
