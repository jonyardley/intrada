import SwiftUI

/// Horizontal row of 1–5 score dots for confidence rating.
///
/// Selected dot shows accent fill + bold white text.
/// Tap toggles selection (tap again to deselect).
///
///     @State var score: UInt8? = nil
///     ScoreSelectorView(selectedScore: $score)
struct ScoreSelectorView: View {

    @Binding var selectedScore: UInt8?

    var body: some View {
        HStack(spacing: 12) {
            ForEach(1...5, id: \.self) { (value: Int) in
                let isSelected = selectedScore == UInt8(value)

                Button {
                    if isSelected {
                        selectedScore = nil
                    } else {
                        selectedScore = UInt8(value)
                    }
                } label: {
                    Text("\(value)")
                        .font(.system(size: 16, weight: isSelected ? .bold : .semibold))
                        .foregroundStyle(isSelected ? Color.textPrimary : Color.textMuted)
                        .frame(width: 40, height: 40)
                        .background(isSelected ? Color.accent : Color.surfacePrimary)
                        .clipShape(Circle())
                }
                .accessibilityLabel("Score \(value)")
                .accessibilityAddTraits(isSelected ? .isSelected : [])
            }
        }
    }
}

#Preview("ScoreSelectorView") {
    struct PreviewWrapper: View {
        @State private var score: UInt8? = 4
        var body: some View {
            VStack(spacing: 24) {
                ScoreSelectorView(selectedScore: $score)
                Text("Selected: \(score.map { String($0) } ?? "none")")
                    .foregroundStyle(Color.textMuted)
            }
            .padding()
            .background(Color.backgroundApp)
        }
    }
    return PreviewWrapper()
}
