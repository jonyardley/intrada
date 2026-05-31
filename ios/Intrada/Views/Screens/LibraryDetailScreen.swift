import SharedTypes
import SwiftUI

/// Read-only detail for a library item: type badge, key/tempo, notes, tags.
/// Pushed from `LibraryScreen`. Edit/delete come in a later increment.
struct LibraryDetailScreen: View {
  let item: LibraryItemView

  var body: some View {
    ScreenScaffold(title: item.title, subtitle: subtitle) {
      ScrollView {
        VStack(alignment: .leading, spacing: 16) {
          TypeBadge(kind: item.itemType)

          if !detailRows.isEmpty {
            VStack(spacing: 0) {
              ForEach(Array(detailRows.enumerated()), id: \.offset) { index, row in
                if index > 0 {
                  Rectangle().fill(IntradaColor.hairline).frame(height: 1)
                }
                DetailRow(label: row.label, value: row.value)
              }
            }
            .cardSurface()
          }

          if let notes = item.notes, !notes.isEmpty {
            Text(notes)
              .font(IntradaFont.body)
              .foregroundStyle(IntradaColor.inkSecondary)
              .frame(maxWidth: .infinity, alignment: .leading)
              .padding(16)
              .cardSurface()
          }

          if !item.tags.isEmpty {
            tags
          }
        }
        .padding(16)
      }
    }
    .navigationBarTitleDisplayMode(.inline)
  }

  private var subtitle: String? {
    item.subtitle.isEmpty ? nil : item.subtitle
  }

  private var detailRows: [(label: String, value: String)] {
    var rows: [(String, String)] = []
    if let key = item.key, !key.isEmpty { rows.append(("Key", key)) }
    if let tempo = item.tempoDisplay { rows.append(("Tempo", tempo)) }
    return rows
  }

  private var tags: some View {
    ScrollView(.horizontal, showsIndicators: false) {
      HStack(spacing: 8) {
        ForEach(item.tags, id: \.self) { tag in
          Text(tag)
            .font(.system(size: 12, weight: .medium))
            .foregroundStyle(IntradaColor.inkSecondary)
            .padding(.vertical, 5)
            .padding(.horizontal, 10)
            .background(IntradaColor.cardFill, in: Capsule())
            .overlay(Capsule().stroke(IntradaColor.hairline, lineWidth: 1))
        }
      }
    }
  }
}

private struct DetailRow: View {
  let label: String
  let value: String

  var body: some View {
    HStack {
      Text(label)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkSecondary)
      Spacer(minLength: 16)
      Text(value)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.trailing)
    }
    .padding(.vertical, 12)
    .padding(.horizontal, 16)
    .accessibilityElement(children: .combine)
    .accessibilityLabel("\(label), \(value)")
  }
}

#if DEBUG
  #Preview("Piece") {
    NavigationStack {
      LibraryDetailScreen(item: .previewDetail)
    }
  }

  #Preview("Minimal") {
    NavigationStack {
      LibraryDetailScreen(item: .previewMinimal)
    }
  }
#endif
