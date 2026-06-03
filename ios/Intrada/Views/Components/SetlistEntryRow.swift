import SharedTypes
import SwiftUI

struct SetlistEntryRow: View {
  let entry: SetlistEntryView

  var body: some View {
    Text(entry.itemTitle)
      .font(IntradaFont.cardTitle())
      .foregroundStyle(IntradaColor.ink)
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(.vertical, IntradaSpacing.row)
      .padding(.leading, 20)
      .padding(.trailing, IntradaSpacing.row)
      .background(IntradaColor.cardFill)
      .overlay(alignment: .leading) { entry.itemType.bar.frame(width: 4) }
      .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
      .overlay(
        RoundedRectangle(cornerRadius: IntradaRadius.card)
          .stroke(IntradaColor.hairline, lineWidth: 1)
      )
      .accessibilityElement(children: .combine)
      .accessibilityLabel("\(entry.itemType.label), \(entry.itemTitle)")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.row) {
        SetlistEntryRow(entry: .previewPiece)
        SetlistEntryRow(entry: .previewExercise)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
