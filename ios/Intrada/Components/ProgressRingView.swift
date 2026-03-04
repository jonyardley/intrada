import SwiftUI

/// Circular progress ring for goal visualisation.
struct ProgressRingView: View {
    let progress: Double // 0.0 – 1.0
    var lineWidth: CGFloat = 6
    var size: CGFloat = 44

    var body: some View {
        ZStack {
            Circle()
                .stroke(.secondary.opacity(0.2), lineWidth: lineWidth)

            Circle()
                .trim(from: 0, to: min(progress, 1.0))
                .stroke(
                    progress >= 1.0 ? Color.green : Color.indigo,
                    style: StrokeStyle(lineWidth: lineWidth, lineCap: .round)
                )
                .rotationEffect(.degrees(-90))
                .animation(.easeInOut(duration: 0.6), value: progress)

            Text("\(Int(min(progress, 1.0) * 100))%")
                .font(.system(size: size * 0.22, weight: .semibold, design: .rounded))
                .foregroundStyle(.secondary)
        }
        .frame(width: size, height: size)
    }
}
