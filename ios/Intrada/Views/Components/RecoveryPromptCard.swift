import SharedTypes
import SwiftUI

/// The crash-recovery prompt shown on the Practice tab when a session-in-progress
/// blob survives a relaunch. One card, two blobs: the legacy `ActiveSession`
/// (#962) and the coach session (#1193, #1305), whose wording is the core's.
struct RecoveryPromptCard: View {
  @Environment(\.calendar) private var calendar
  @Environment(\.locale) private var locale

  private let altitude: Altitude?
  private let title: String
  private let detail: String
  private let startedAt: Date?
  /// Injected for deterministic snapshots; production passes "now".
  private let referenceDate: Date
  private let onResume: () -> Void
  private let onDiscard: () -> Void

  init(
    session: ActiveSession, referenceDate: Date = Date(), onResume: @escaping () -> Void,
    onDiscard: @escaping () -> Void
  ) {
    let position = min(Int(session.currentIndex) + 1, session.entries.count)
    altitude = nil
    title = "Pick up where you left off?"
    detail = "\(position) of \(session.entries.count) items"
    startedAt = SessionClock.parseRFC3339(session.sessionStartedAt)
    self.referenceDate = referenceDate
    self.onResume = onResume
    self.onDiscard = onDiscard
  }

  /// Headline and detail are written by the core, per altitude — the shell
  /// renders them and works out nothing about what was being practised.
  init(
    recovery: RecoveryView, referenceDate: Date = Date(), onResume: @escaping () -> Void,
    onDiscard: @escaping () -> Void
  ) {
    altitude = recovery.altitude
    title = recovery.headline
    detail = recovery.detail
    startedAt = SessionClock.parseRFC3339(recovery.startedAt)
    self.referenceDate = referenceDate
    self.onResume = onResume
    self.onDiscard = onDiscard
  }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      eyebrow

      Text(title)
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)

      Text(meta)
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.inkSecondary)
        .fixedSize(horizontal: false, vertical: true)

      HStack(spacing: IntradaSpacing.controlGap) {
        Button(action: onResume) {
          Label("Resume", systemImage: "play.fill")
            .font(IntradaFont.bodyMedium)
            .frame(maxWidth: .infinity)
            .padding(.vertical, IntradaSpacing.cardCompact)
        }
        .buttonStyle(.borderedProminent)
        .tint(IntradaColor.accent)
        .accessibilityLabel("Resume the interrupted session")

        Button("Discard", action: onDiscard)
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.inkSecondary)
          .padding(.horizontal, IntradaSpacing.cardCompact)
          .accessibilityLabel("Discard the interrupted session")
      }
    }
    .padding(IntradaSpacing.card)
    .frame(maxWidth: .infinity, alignment: .leading)
    .background(IntradaColor.cardFill)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .accessibilityElement(children: .contain)
  }

  /// The altitude chip rather than a word, where there is one: what was being
  /// recorded is the half of the offer the user is actually agreeing to.
  @ViewBuilder private var eyebrow: some View {
    if let altitude {
      AltitudeChip(altitude: altitude)
    } else {
      Eyebrow("Session in progress", tint: IntradaColor.celebrationInk)
    }
  }

  private var meta: String {
    guard let startedAt else { return detail }
    let formatter = DateFormatter()
    // An old blob saying just "9:02 AM" reads as today — show the date too.
    formatter.dateStyle = calendar.isDate(startedAt, inSameDayAs: referenceDate) ? .none : .medium
    formatter.timeStyle = .short
    formatter.calendar = calendar
    formatter.timeZone = calendar.timeZone
    formatter.locale = locale
    return "\(detail) · started \(formatter.string(from: startedAt))"
  }
}

#if DEBUG
  extension RecoveryView {
    /// Wording as the core writes it — copied from `CoachState::recovery_view`
    /// so previews and snapshots read what the device would show.
    static var previewDrill: RecoveryView {
      RecoveryView(
        altitude: nil, headline: "Pick up where you left off?",
        detail: "Rootless voicings · block 2 of 5", startedAt: "2026-06-16T09:02:00Z")
    }

    static var previewOffPiste: RecoveryView {
      RecoveryView(
        altitude: .offPiste, headline: "Back to exploring?",
        detail: "Time logged, nothing scored.", startedAt: "2026-06-16T09:02:00Z")
    }
  }

  #Preview("Coach session") {
    RecoveryPromptCard(
      recovery: .previewDrill,
      referenceDate: SessionClock.parseRFC3339("2026-06-16T11:00:00Z") ?? Date(),
      onResume: {}, onDiscard: {}
    )
    .padding(IntradaSpacing.card)
    .background(IntradaColor.paperTop)
  }

  #Preview("Altitude") {
    RecoveryPromptCard(
      recovery: .previewOffPiste,
      referenceDate: SessionClock.parseRFC3339("2026-06-16T11:00:00Z") ?? Date(),
      onResume: {}, onDiscard: {}
    )
    .padding(IntradaSpacing.card)
    .background(IntradaColor.paperTop)
  }
#endif
