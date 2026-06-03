import SwiftUI

/// A truncating row of tag chips for list rows: up to `limit` tags plus a `+N`
/// overflow chip, so a card with many tags stays the same height as one with few.
struct TagPills: View {
  let tags: [String]
  var limit: Int = 3

  var body: some View {
    let shown = Array(tags.prefix(limit))
    let overflow = tags.count - shown.count
    HStack(spacing: 6) {
      ForEach(shown, id: \.self) { TagChip($0) }
      if overflow > 0 {
        TagChip("+\(overflow)")
      }
    }
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Tags: \(tags.joined(separator: ", "))")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        TagPills(tags: ["classical", "piano"])
        TagPills(tags: ["jazz", "improv", "bebop", "ii-V-I", "comping"])
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
