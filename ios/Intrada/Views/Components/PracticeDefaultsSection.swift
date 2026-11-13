import SharedTypes
import SwiftUI

struct PracticeDefaultsSection: View {
  let defaults: PracticeDefaults
  let limits: LimitsView
  let onSave: (PracticeDefaults) -> Void

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionTitle("Practice defaults")
      VStack(alignment: .leading, spacing: 0) {
        VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
          FieldLabel("Metronome starts on")
          SegmentedPills(
            options: [ClickStart.everyBeat, .twoAndFour],
            selection: Binding(
              get: { defaults.click },
              set: {
                save(
                  PracticeDefaults(click: $0, sessionLengthMins: defaults.sessionLengthMins))
              }),
            label: \.title,
            identifier: { "profile.click.\($0.identifierSuffix)" },
            font: IntradaFont.segment,
            layout: .fullWidthTrack)
        }
        .padding(IntradaSpacing.card)
        HairlineDivider()
          .padding(.leading, IntradaSpacing.card)
        SessionLengthControl(
          title: "Session length",
          lengthMins: defaults.sessionLengthMins,
          detail: defaults.sessionLengthMins == nil ? "Off" : nil,
          valueLabel: "\(defaults.sessionLengthMins ?? 0) min",
          limits: limits,
          identifier: "profile.sessionLength"
        ) {
          save(
            PracticeDefaults(click: defaults.click, sessionLengthMins: $0))
        }
        .padding(IntradaSpacing.card)
      }
      .cardSurface()
      Text("New sessions start with these. Change them in the session.")
        .font(IntradaFont.small)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
  }

  private func save(_ next: PracticeDefaults) {
    guard next != defaults else { return }
    onSave(next)
  }
}

extension ClickStart {
  fileprivate var title: String {
    switch self {
    case .everyBeat: ClickPreset.everyBeat.title
    case .twoAndFour: ClickPreset.backbeat.title
    }
  }

  fileprivate var identifierSuffix: String {
    switch self {
    case .everyBeat: "everyBeat"
    case .twoAndFour: "twoAndFour"
    }
  }
}
