import SwiftUI

/// Rep counter with Got it / Missed buttons.
///
/// Shows "X / Y" counter, progress bar, and action buttons.
/// Displays a celebration state when the target is reached.
///
///     RepCounterView(count: 3, target: 5, targetReached: false,
///                    onGotIt: { }, onMissed: { })
struct RepCounterView: View {

    let count: UInt8
    let target: UInt8
    let targetReached: Bool
    let onGotIt: () -> Void
    let onMissed: () -> Void

    private var progress: Double {
        guard target > 0 else { return 0 }
        return Double(count) / Double(target)
    }

    var body: some View {
        VStack(spacing: 10) {
            // Label
            Text("CONSECUTIVE REPS")
                .font(.system(size: 9, weight: .semibold))
                .tracking(1.5)
                .foregroundStyle(Color.textFaint)

            if targetReached {
                // Celebration state
                HStack(spacing: 8) {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundStyle(Color.successText)
                    Text("Target reached!")
                        .font(.system(size: 16, weight: .bold))
                        .foregroundStyle(Color.successText)
                }
            } else {
                // Counter
                Text("\(count) / \(target)")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundStyle(Color.textPrimary)
            }

            // Progress bar
            GeometryReader { geometry in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: DesignRadius.pill)
                        .fill(Color.surfaceInput)
                        .frame(height: 5)

                    RoundedRectangle(cornerRadius: DesignRadius.pill)
                        .fill(targetReached ? Color.success : Color.accent)
                        .frame(width: geometry.size.width * progress, height: 5)
                        .animation(.easeInOut(duration: 0.3), value: count)
                }
            }
            .frame(height: 5)

            // Buttons
            if !targetReached {
                HStack(spacing: 10) {
                    ButtonView("Got it", variant: .primary) { onGotIt() }
                    ButtonView("Missed", variant: .dangerOutline) { onMissed() }
                }
            }
        }
        .padding(Spacing.card)
        .background(Color.surfaceSecondary)
        .clipShape(RoundedRectangle(cornerRadius: DesignRadius.card))
    }
}

#Preview("RepCounterView") {
    VStack(spacing: 16) {
        RepCounterView(count: 3, target: 5, targetReached: false,
                       onGotIt: { }, onMissed: { })
        RepCounterView(count: 5, target: 5, targetReached: true,
                       onGotIt: { }, onMissed: { })
    }
    .padding()
    .background(Color.backgroundApp)
}
