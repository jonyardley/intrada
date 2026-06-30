import SwiftUI

/// A 5-step ascending bar filled to the item's mastery level, trailing a Library
/// row. Monochrome indigo — the *count* of filled bars carries the meaning, so
/// it stays legible to colour-blind users (never recolour by level).
///
/// `level` is a 0–10 score (`practice.latestScore`); bars map proportionally
/// (each bar ≈ 2 points). `level == nil` → never practised.
struct MasteryMeter: View {
  let level: Int?
  var steps: Int = 5

  private static let heights: [CGFloat] = [8, 11, 14, 17, 20]

  var body: some View {
    VStack(spacing: 4) {
      HStack(alignment: .bottom, spacing: 3) {
        ForEach(0..<steps, id: \.self) { i in
          RoundedRectangle(cornerRadius: 2)
            .fill(filled(i) ? IntradaColor.masteryFill : IntradaColor.masteryTrack)
            .frame(width: 5, height: Self.heights[min(i, Self.heights.count - 1)])
        }
      }
      .frame(height: 20, alignment: .bottom)
      Text(caption)
        .font(IntradaFont.micro)
        .foregroundStyle(IntradaColor.inkFaint)
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(
      level == nil ? "Not yet practised" : "Mastery \(level ?? 0) of 10")
  }

  private func filled(_ index: Int) -> Bool {
    guard let level else { return false }
    // 5 bars across a 0–10 scale: each bar ≈ 2 points.
    return index < Int((Double(level) / 2.0).rounded())
  }

  private var caption: String {
    guard let level else { return "—" }
    return "\(level) / 10"
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      HStack(spacing: 20) {
        ForEach([1, 3, 6, 8, 10], id: \.self) { MasteryMeter(level: $0) }
        MasteryMeter(level: nil)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
