import SharedTypes
import SwiftUI

/// Read straight from the core so a refused value never shows (#1736).
struct SessionLengthControl: View {
  let title: String
  let lengthMins: UInt16?
  let detail: String?
  let valueLabel: String
  let limits: LimitsView
  let identifier: String
  let onChange: (UInt16?) -> Void
  @Environment(\.dynamicTypeSize) private var typeSize

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      Toggle(
        isOn: Binding(
          get: { lengthMins != nil },
          set: { on in onChange(on ? limits.sessionLengthDefaultMins : nil) })
      ) {
        VStack(alignment: .leading, spacing: 2) {
          Text(title)
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.ink)
          if let detail {
            Text(detail)
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
        }
      }
      .tint(IntradaColor.accent)
      .accessibilityIdentifier("\(identifier).toggle")
      if let lengthMins {
        let rowLayout =
          typeSize.isAccessibilitySize
          ? AnyLayout(VStackLayout(alignment: .leading, spacing: IntradaSpacing.controlGap))
          : AnyLayout(HStackLayout(spacing: IntradaSpacing.cardCompact))
        rowLayout {
          Text(valueLabel)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
            .accessibilityHidden(true)
          if !typeSize.isAccessibilitySize { Spacer(minLength: 0) }
          Stepper(
            "\(title), \(valueLabel)",
            value: Binding(
              get: { Int(lengthMins) },
              set: { onChange(UInt16($0)) }),
            in: Int(limits.sessionLengthMinMins)...Int(limits.sessionLengthMaxMins),
            step: Int(limits.sessionLengthStepMins)
          )
          .labelsHidden()
          .fixedSize()
          .accessibilityIdentifier("\(identifier).stepper")
        }
      }
    }
  }
}
