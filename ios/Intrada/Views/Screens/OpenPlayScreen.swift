import SharedTypes
import SwiftUI

/// B2 and B3 — the two lower altitudes (decision 16). Off-piste logs the time
/// and the piece it was on; unmonitored logs minutes and nothing else. Neither
/// has a gate, a verdict or a prompt, and the absence of instrumentation is
/// itself the consent signal, so nothing may quietly appear here later.
struct OpenPlayScreen: View {
  let state: OpenPlayView
  /// Set only by previews and snapshots, to freeze the clock at a known
  /// instant. Live, the screen reads the wall clock through `TimelineView`.
  var frozenNow: Date?
  var onDone: () -> Void

  @Environment(\.horizontalSizeClass) private var sizeClass

  private var scale: CoachScale { sizeClass == .regular ? .regular : .compact }
  private var gutter: CGFloat { scale == .compact ? IntradaSpacing.card : IntradaSpacing.stage }
  private var offPiste: Bool { state.altitude == .offPiste }

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      if let frozenNow {
        content(elapsed: elapsedSeconds(at: frozenNow))
      } else {
        // One second is the finest either screen reads at, and the core has no
        // tick at these altitudes on purpose — there is nothing here for it to
        // decide, so the clock is the shell's to draw from `startedAt`.
        TimelineView(.periodic(from: .now, by: 1)) { context in
          content(elapsed: elapsedSeconds(at: context.date))
        }
      }
    }
    .environment(\.coachScale, scale)
    .dynamicTypeSize(.xSmall ... .accessibility5)
  }

  private func content(elapsed: Int) -> some View {
    VStack(spacing: 0) {
      OrientationStrip(
        // The big face below is the clock here; a second one in the strip
        // would be the same number twice.
        elapsedSeconds: nil, blockKinds: [], blockIndex: 0,
        altitude: state.altitude, onDismiss: onDone)
      Spacer(minLength: IntradaSpacing.card)
      centre(elapsed: elapsed)
      Spacer(minLength: IntradaSpacing.card)
      footer
    }
    .padding(.horizontal, gutter)
    .padding(.bottom, IntradaSpacing.section)
  }

  @ViewBuilder private func centre(elapsed: Int) -> some View {
    VStack(spacing: IntradaSpacing.card) {
      if let title = state.title {
        Text(title)
          .font(IntradaFont.drillTitle(scale.drillTitle))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
      }
      Text(face(elapsed))
        .font(IntradaFont.timer(scale.tempo))
        // Ink for a piece you are working on, inkSecondary off the record —
        // even the number refuses to perform there.
        .foregroundStyle(offPiste ? IntradaColor.ink : IntradaColor.inkSecondary)
        .monospacedDigit()
        .accessibilityLabel(spokenClock(elapsed))
      if !offPiste {
        Text("Just you and the piano.")
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
      }
    }
  }

  /// Off-piste counts in seconds, because a voice note will one day land at one
  /// (#1256 Phase D). Off the record rounds to minutes: a second-by-second
  /// count is a measurement, and this altitude takes none.
  private func face(_ elapsed: Int) -> String {
    guard offPiste else { return "\(max(0, elapsed) / 60) min" }
    return SessionClock.clockDisplay(elapsed)
  }

  private func spokenClock(_ elapsed: Int) -> String {
    let minutes = max(0, elapsed) / 60
    let unit = minutes == 1 ? "minute" : "minutes"
    return "\(minutes) \(unit) so far"
  }

  /// One primary off-piste, and deliberately none off the record: there is
  /// nothing the app wants from you there, so Done is quiet too.
  @ViewBuilder private var footer: some View {
    if offPiste {
      BrandBarButton(prominent: true, action: onDone) {
        Text("I'm done")
      }
    } else {
      Button("Done", action: onDone)
        .buttonStyle(QuietAction())
    }
  }

  private func elapsedSeconds(at instant: Date) -> Int {
    guard let started = SessionClock.parseRFC3339(state.startedAt) else { return 0 }
    return Int(instant.timeIntervalSince(started))
  }
}

#if DEBUG
  extension OpenPlayView {
    static func preview(
      altitude: Altitude = .offPiste, title: String? = "Alice in Wonderland"
    ) -> OpenPlayView {
      OpenPlayView(
        altitude: altitude,
        itemId: altitude == .offPiste ? "p1" : nil,
        title: altitude == .offPiste ? title : nil,
        startedAt: "2026-08-07T10:00:00Z")
    }
  }

  /// 11:38 into the run, matching the design's worked example.
  private let previewNow = Date(timeIntervalSince1970: 1_786_226_298)

  #Preview("Off-piste") {
    OpenPlayScreen(state: .preview(), frozenNow: previewNow, onDone: {})
  }

  #Preview("Off the record") {
    OpenPlayScreen(state: .preview(altitude: .unmonitored), frozenNow: previewNow, onDone: {})
  }

  #Preview("Off-piste, largest accessibility size") {
    OpenPlayScreen(state: .preview(), frozenNow: previewNow, onDone: {})
      .environment(\.dynamicTypeSize, .accessibility5)
  }
#endif
