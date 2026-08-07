import SharedTypes
import SwiftUI

/// A2 (during play) and A3 (after a repetition) — one shell, so the eye lands
/// in the same place between reps. Rules in `design/Drill Loop.dc.html`.
struct DrillScreen: View {
  let state: DrillView
  /// `true` = "Yes — clean", `false` = "No — missed it".
  var onVerdict: (Bool) -> Void
  var onStuck: () -> Void
  /// Neither a pass nor a fail, so it never reaches the gate or the estimate.
  var onDiscard: () -> Void
  var onStart: () -> Void
  var onSkip: () -> Void
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
  private var isBlockEntry: Bool {
    if case .blockEntry = state.phase { return true }
    return false
  }
  /// l0: no click, no tempo, no count-in — the same fact `DrillView.tempoBpm`
  /// itself is built from, so this is a read of the core's own state, not the
  /// shell inferring one.
  private var isUntimed: Bool { state.tempoBpm == nil }

  /// On a block that has not run, a zeroed clock is noise. On one parked by an
  /// interruption, the time already spent is the most useful thing to show
  /// someone coming back from a phone call (#1223).
  private var cardAwareElapsed: Int? {
    if isBlockEntry && state.elapsedSeconds == 0 { return nil }
    return Int(state.elapsedSeconds)
  }

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: 0) {
        OrientationStrip(
          elapsedSeconds: cardAwareElapsed,
          // The block's clock starts when the hands do, so the card shows no
          // ceiling — a countdown against unstarted time reads as a deadline.
          ceilingSeconds: isBlockEntry ? nil : state.ceilingSeconds.map(Int.init),
          blockKinds: state.blockKinds, blockIndex: Int(state.blockIndex),
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
    case .blockEntry:
      // The card is read as one centred group, so its identity lives in `centre`.
      EmptyView()
    case .playing, .countIn:
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
          .fixedSize(horizontal: false, vertical: true)
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
    case .blockEntry:
      blockEntryCard
    case .countIn:
      VStack(spacing: IntradaSpacing.row + 2) {
        tempo
        clickPill
        BeatPosition(
          beat: Int(state.beat), beatsPerBar: Int(state.beatsPerBar), bar: Int(state.bar),
          bars: Int(state.bars)
        )
        .padding(.top, IntradaSpacing.cardCompact)
      }
    case .playing:
      if isUntimed {
        // No tempo means no beat to count either — a position frozen at bar 1
        // beat 1 would claim a pulse that never runs. The gate is the one fact
        // left worth a glance mid-play, same as everywhere else it's asked.
        GateDots(filled: Int(state.gateFilled), target: Int(state.gateTarget))
      } else {
        VStack(spacing: IntradaSpacing.row + 2) {
          tempo
          clickPill
          BeatPosition(
            beat: Int(state.beat), beatsPerBar: Int(state.beatsPerBar), bar: Int(state.bar),
            bars: Int(state.bars)
          )
          .padding(.top, IntradaSpacing.cardCompact)
        }
      }
    case .awaitingVerdict:
      VStack(spacing: IntradaSpacing.section) {
        Text(state.gateQuestion)
          .font(IntradaFont.verdict(scale.question))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
          // Without this it truncates to "Clean at…" at accessibility sizes,
          // which asks nothing.
          .fixedSize(horizontal: false, vertical: true)
        GateDots(filled: Int(state.gateFilled), target: Int(state.gateTarget))
      }
    case .acknowledged(let clean):
      VStack(spacing: IntradaSpacing.section + 2) {
        RepVerdict(outcome: clean ? .clean : .missed)
        GateDots(filled: Int(state.gateFilled), target: Int(state.gateTarget))
      }
    case .gateOpen:
      VStack(spacing: IntradaSpacing.section) {
        RepVerdict(outcome: .clean, fact: "Gate open")
        GateDots(
          filled: Int(state.gateFilled), target: Int(state.gateTarget),
          caption: state.gateSummary)
      }
    }
  }

  /// The T1 intention beat — read in a glance, never a config surface.
  private var blockEntryCard: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      if showsIdentityDetail {
        TypeBadge(kind: state.kind, label: "Up next")
      }
      Text(state.drillTitle)
        .font(IntradaFont.drillTitle(scale.drillTitle))
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)
      Text(blockShape)
        .font(IntradaFont.ambient(scale == .compact ? 14 : 20))
        .foregroundStyle(IntradaColor.inkSecondary)
        .multilineTextAlignment(.center)
        .accessibilityLabel(spoken(blockShape))
      Label(state.why, systemImage: "arrow.turn.down.right")
        .font(IntradaFont.ambient(scale == .compact ? 14 : 19))
        .foregroundStyle(IntradaColor.exerciseBadgeFg)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)
        .padding(.top, IntradaSpacing.controlGap)
        .accessibilityLabel(spoken(state.why))
    }
  }

  private var blockShape: String {
    var parts = [state.section, "about \(state.minutes) min"].compactMap { $0 }
    // l0 has no click to leave unsaid — the entry card names it same as the
    // running screen does, rather than a start that surprises with silence.
    if isUntimed { parts.append(state.clickLevel) }
    return parts.joined(separator: " · ")
  }

  /// Nothing at all where the rung has no tempo (l0), which the core says by
  /// sending none.
  @ViewBuilder private var tempo: some View {
    if let bpm = state.tempoBpm {
      HStack(alignment: .firstTextBaseline, spacing: scale == .compact ? 9 : 12) {
        Text("♩")
          .font(IntradaFont.drillTitle(scale.tempo * 0.88))
        Text("= \(bpm)")
          .font(IntradaFont.drillTitle(scale.tempo))
          .monospacedDigit()
      }
      .foregroundStyle(IntradaColor.ink)
      .accessibilityElement(children: .ignore)
      .accessibilityLabel("\(bpm) beats per minute")
    }
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
    case .blockEntry:
      VStack(spacing: 0) {
        CoachAction(
          title: "Start", emphasis: .primary, hint: "Begins the count-in for this block",
          action: onStart)
        CoachAction(title: "Skip this block", emphasis: .quiet, action: onSkip)
          .padding(.top, IntradaSpacing.controlGap)
      }
    case .playing:
      if isUntimed {
        // No `.awaitingVerdict` to reach at l0 — the tap bounds the attempt in
        // place, so the same tap-verdict footer that phase builds has to be
        // reachable from here instead.
        tapVerdictFooter
      } else {
        StuckTarget(action: onStuck)
      }
    case .countIn(let remaining):
      CountIn(remaining: Int(remaining), total: Int(state.countInBeats))
    case .awaitingVerdict:
      tapVerdictFooter
    case .acknowledged:
      // Deliberately empty: the glance carries the verdict alone (#1184).
      EmptyView()
    case .gateOpen:
      Text("moving on")
        .font(IntradaFont.ambient())
        .foregroundStyle(IntradaColor.inkSecondary)
    }
  }

  /// The tap-verdict pair plus its escapes — reachable from both A3's
  /// `.awaitingVerdict` and l0's `.playing`, since l0 has no separate verdict
  /// phase to arrive at (the tap bounds the attempt in place).
  private var tapVerdictFooter: some View {
    VStack(spacing: 0) {
      TapVerdict(onClean: { onVerdict(true) }, onMissed: { onVerdict(false) })
      HairlineDivider()
        .padding(.top, IntradaSpacing.row - 2)
        .padding(.bottom, IntradaSpacing.controlGap + 2)
      escapes
    }
  }

  /// Both quiet: the verdict is still the ask.
  @ViewBuilder private var escapes: some View {
    let stuck = StuckTarget(emphasis: .quiet, action: onStuck)
    let discard = CoachAction(
      title: "Don't count that", emphasis: .quiet,
      hint: "Throws the attempt away — neither clean nor missed", action: onDiscard)
    if typeSize.isAccessibilitySize {
      // Borderless, so ungapped they read as one 92pt target doing two things,
      // and hitting "I'm stuck" by mistake costs a rung of tempo.
      VStack(spacing: IntradaSpacing.cardCompact) {
        stuck
        discard
      }
    } else {
      HStack(spacing: IntradaSpacing.controlGap) {
        stuck
        discard
      }
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
    .accessibilityLabel(spokenCountIn)
  }

  private var spokenCountIn: String {
    switch remaining {
    case 0: "Count-in, last beat"
    case 1: "Count-in, 1 beat to go"
    default: "Count-in, \(remaining) beats to go"
    }
  }
}

#if DEBUG
  extension DrillView {
    /// The design's worked example: *Strasbourg / St. Denis*. A fixture for
    /// previews and snapshots only — at runtime every field comes from the
    /// core's `CoachView`.
    static func preview(
      phase: DrillPhase = .playing, gateFilled: UInt8 = 2, elapsedSeconds: UInt32 = 724,
      tempoBpm: UInt16? = 120, clickLevel: String = "beats 2 & 4"
    ) -> DrillView {
      DrillView(
        phase: phase,
        drillTitle: "Rootless voicings",
        section: "A section",
        destination: "Strasbourg / St. Denis",
        kind: .exercise,
        tempoBpm: tempoBpm,
        clickLevel: clickLevel,
        beat: 2, beatsPerBar: 4, bar: 3, bars: 8,
        countInBeats: 4, phraseBeats: 32,
        pulseSeq: 1, pulseRunning: tempoBpm != nil, clickPattern: [false, true, false, true],
        elapsedSeconds: elapsedSeconds,
        minutes: 8,
        why: "Shells and rootless are what sit between you and improvising over Strasbourg.",
        ceilingSeconds: 360,
        blockKinds: [.piece, .exercise, .exercise, .piece, .piece], blockIndex: 1,
        gateQuestion: "Clean at 120?",
        gateSummary: "3 clean at 120",
        gateFilled: gateFilled, gateTarget: 3)
    }
  }

  private struct DrillPreview: View {
    let state: DrillView
    var body: some View {
      DrillScreen(
        state: state, onVerdict: { _ in }, onStuck: {}, onDiscard: {}, onStart: {}, onSkip: {},
        onDismiss: {})
    }
  }

  #Preview("Block entry") {
    DrillPreview(state: .preview(phase: .blockEntry, elapsedSeconds: 0))
  }

  /// The same card after an interruption parked the block: three minutes in,
  /// so the clock stays on screen.
  #Preview("Block entry, resumed") {
    DrillPreview(state: .preview(phase: .blockEntry, elapsedSeconds: 184))
  }

  #Preview("A2 during play") { DrillPreview(state: .preview()) }

  #Preview("A2 l0 during play") {
    DrillPreview(state: .preview(tempoBpm: nil, clickLevel: "no click"))
  }

  #Preview("Block entry, l0") {
    DrillPreview(
      state: .preview(phase: .blockEntry, elapsedSeconds: 0, tempoBpm: nil, clickLevel: "no click")
    )
  }

  #Preview("A2 largest accessibility size") {
    DrillPreview(state: .preview()).environment(\.dynamicTypeSize, .accessibility5)
  }

  #Preview("A3 tap-verdict") { DrillPreview(state: .preview(phase: .awaitingVerdict)) }

  #Preview("A3 first rep") {
    DrillPreview(state: .preview(phase: .awaitingVerdict, gateFilled: 0, elapsedSeconds: 192))
  }

  #Preview("A3 pass tap") {
    DrillPreview(
      state: .preview(phase: .acknowledged(clean: true), gateFilled: 3, elapsedSeconds: 761))
  }

  #Preview("A3 fail tap") {
    DrillPreview(state: .preview(phase: .acknowledged(clean: false), elapsedSeconds: 798))
  }

  #Preview("A2 count-in") {
    DrillPreview(state: .preview(phase: .countIn(remaining: 2), elapsedSeconds: 803))
  }

  #Preview("A3 gate open") {
    DrillPreview(state: .preview(phase: .gateOpen, gateFilled: 3, elapsedSeconds: 842))
  }
#endif
