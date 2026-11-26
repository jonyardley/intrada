import SharedTypes
import SwiftUI

/// Read straight from the core so a refused value never shows (#1736).
struct SessionLengthControl: View {
  let title: String
  let lengthMins: UInt16?
  let detail: String?
  let limits: LimitsView
  let identifier: String
  let onChange: (UInt16?) -> Void

  static func spokenMinutes(_ minutes: Int) -> String {
    "\(minutes) \(minutes == 1 ? "minute" : "minutes")"
  }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      Toggle(
        isOn: Binding(
          get: { lengthMins != nil },
          set: { on in onChange(on ? limits.sessionLengthDefaultMins : nil) })
      ) {
        VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
          FieldLabel(title)
          if let detail {
            Text(detail)
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
        }
      }
      .tint(IntradaColor.accent)
      .accessibilityIdentifier("\(identifier).toggle")
      if let lengthMins {
        let minutes = Int(lengthMins)
        ScoreStepper(
          score: minutes, caption: .unit("minutes"),
          range: Int(limits.sessionLengthMinMins)...Int(limits.sessionLengthMaxMins),
          step: Int(limits.sessionLengthStepMins), figure: IntradaFont.timer(32), typing: true,
          accessibilityLabel: title, accessibilityValue: Self.spokenMinutes(minutes)
        ) { next in
          onChange(UInt16(clamping: next))
        }
        .accessibilityIdentifier("\(identifier).stepper")
        .frame(maxWidth: .infinity)
      }
    }
  }
}
