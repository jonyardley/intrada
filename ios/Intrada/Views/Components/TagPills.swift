import SwiftUI

/// A compact, truncating row of tag chips for list rows: shows up to `limit`
/// tags and a `+N` overflow chip, so a card with many tags stays the same
/// height as one with few. The detail screen shows the full set; this is the
/// at-a-glance version.
struct TagPills: View {
  let tags: [String]
  var limit: Int = 3

  var body: some View {
    let shown = Array(tags.prefix(limit))
    let overflow = tags.count - shown.count
    HStack(spacing: 6) {
      ForEach(shown, id: \.self) { TagChip(text: $0) }
      if overflow > 0 {
        TagChip(text: "+\(overflow)")
      }
    }
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Tags: \(tags.joined(separator: ", "))")
  }
}

private struct TagChip: View {
  let text: String

  var body: some View {
    Text(text)
      .font(IntradaFont.metaMedium)
      .foregroundStyle(IntradaColor.inkSecondary)
      .lineLimit(1)
      .padding(.vertical, 3)
      .padding(.horizontal, 8)
      .background(IntradaColor.surfaceSunken, in: Capsule())
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: 12) {
        TagPills(tags: ["classical", "piano"])
        TagPills(tags: ["jazz", "improv", "bebop", "ii-V-I", "comping"])
      }
      .padding(16)
    }
  }
#endif
