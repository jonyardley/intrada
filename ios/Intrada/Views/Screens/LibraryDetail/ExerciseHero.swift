import SharedTypes
import SwiftUI

struct ExerciseHero: View {
  let item: LibraryItemView

  var body: some View {
    VStack(spacing: 6) {
      ScoreRing(
        score: item.practice?.latestScore.map(Int.init), size: 132, showsScale: true)
      // Names the hero as the score across every piece, so it can't be read as
      // one piece's: the distinction the "Used in" rows below make (#1087 B2).
      if !item.usedIn.isEmpty {
        Eyebrow("Overall")
      }
    }
    .frame(maxWidth: .infinity)
    .padding(.vertical, IntradaSpacing.controlGap)
  }
}
