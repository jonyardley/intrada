import SharedTypes
import SwiftUI

struct PracticeDefaultsSection: View {
  let defaults: PracticeDefaults
  let limits: LimitsView
  let onSave: (PracticeDefaults) -> Void
  @Environment(\.dynamicTypeSize) private var typeSize

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Eyebrow("Practice defaults")
      VStack(alignment: .leading, spacing: 0) {
        let rowLayout =
          typeSize.isAccessibilitySize
          ? AnyLayout(VStackLayout(alignment: .leading, spacing: IntradaSpacing.controlGap))
          : AnyLayout(HStackLayout(spacing: IntradaSpacing.cardCompact))
        rowLayout {
          VStack(alignment: .leading, spacing: 2) {
            Text("Repetitions")
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.ink)
            Text("\(defaults.repTarget) per item")
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
          .accessibilityHidden(true)
          if !typeSize.isAccessibilitySize { Spacer(minLength: 0) }
          Stepper(
            "Repetitions, \(defaults.repTarget) per item",
            value: Binding(
              get: { Int(defaults.repTarget) },
              set: {
                save(
                  PracticeDefaults(
                    repTarget: UInt8($0), click: defaults.click,
                    sessionLengthMins: defaults.sessionLengthMins))
              }),
            in: Int(limits.repTargetMin)...Int(limits.repTargetMax)
          )
          .labelsHidden()
          .fixedSize()
          .accessibilityIdentifier("profile.repTarget")
        }
        .padding(.vertical, IntradaSpacing.cardCompact)
        .padding(.horizontal, IntradaSpacing.card)
        HairlineDivider()
          .padding(.leading, IntradaSpacing.card)
        VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
          Text("Metronome starts on")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.ink)
          SegmentedPills(
            options: [ClickStart.everyBeat, .twoAndFour],
            selection: Binding(
              get: { defaults.click },
              set: {
                save(
                  PracticeDefaults(
                    repTarget: defaults.repTarget, click: $0,
                    sessionLengthMins: defaults.sessionLengthMins))
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
            PracticeDefaults(
              repTarget: defaults.repTarget, click: defaults.click, sessionLengthMins: $0))
        }
        .padding(IntradaSpacing.card)
      }
      .cardSurface()
      Text("New sessions start with these. Change them in the session.")
        .font(IntradaFont.micro)
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
