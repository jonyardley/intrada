import SwiftUI

/// Circular progress ring for countdown timers.
///
/// Uses `Circle().trim()` for clean stroke-only rendering.
/// The ring fills clockwise from the top as progress increases.
///
///     ProgressRingView(progress: 0.6, lineWidth: 5)
struct ProgressRingView: View {

    /// Progress from 0.0 (empty) to 1.0 (full).
    let progress: Double

    /// Stroke width for the ring.
    var lineWidth: CGFloat = 5

    /// Opacity for the background track.
    var trackOpacity: Double = 0.2

    /// Opacity for the progress fill.
    var fillOpacity: Double = 0.15

    var body: some View {
        ZStack {
            // Background track
            Circle()
                .stroke(Color.surfacePrimary, lineWidth: lineWidth)
                .opacity(trackOpacity)

            // Progress fill — starts from top (rotated -90°), fills clockwise
            Circle()
                .trim(from: 0, to: min(progress, 1.0))
                .stroke(
                    Color.accent,
                    style: StrokeStyle(lineWidth: lineWidth, lineCap: .round)
                )
                .rotationEffect(.degrees(-90))
                .opacity(fillOpacity)
                .animation(.linear(duration: 0.5), value: progress)
        }
    }
}

#Preview("ProgressRingView") {
    VStack(spacing: 24) {
        ProgressRingView(progress: 0.0)
            .frame(width: 160, height: 160)

        ProgressRingView(progress: 0.6)
            .frame(width: 160, height: 160)

        ProgressRingView(progress: 1.0)
            .frame(width: 160, height: 160)
    }
    .padding()
    .background(Color.backgroundApp)
}
