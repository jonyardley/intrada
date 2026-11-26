#if DEBUG
  import SharedTypes
  import SwiftUI

  struct UsualTimeSketch: View {
    enum Variation {
      case underBadge
      case withSessions
      case sameStyle
    }

    let item: LibraryItemView
    let variation: Variation

    var body: some View {
      NavigationStack {
        ScreenScaffold(
          title: item.title, subtitle: item.subtitle.isEmpty ? nil : item.subtitle,
          trailingContent: {
            HStack(spacing: IntradaSpacing.card) {
              Button {
              } label: {
                Image(systemName: item.priority ? "star.fill" : "star")
              }
              .tint(IntradaColor.inkSecondary)
              Button("Edit") {}
            }
          },
          content: {
            VStack(alignment: .leading, spacing: IntradaSpacing.card) {
              if item.itemType == .exercise {
                ExerciseHero(item: item)
              }
              VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
                badgeRow
                usualLine
              }
            }
            .padding(IntradaSpacing.card)
          }
        )
      }
    }

    private var badgeRow: some View {
      ScrollView(.horizontal, showsIndicators: false) {
        HStack(spacing: IntradaSpacing.controlGap) {
          TypeBadge(kind: item.itemType)
          ForEach(item.tags, id: \.self) { tag in
            TagChip(tag, style: .outlined)
          }
        }
      }
    }

    private var usualLine: some View {
      Text(usualCopy)
        .font(IntradaFont.secondary)
        .foregroundStyle(IntradaColor.inkSecondary)
        .fixedSize(horizontal: false, vertical: true)
    }

    private var usualCopy: String {
      let minutes = "Usually \(item.usualMinutes) min"
      guard item.usualMinutesFromHistory else { return "\(minutes), from your starting time" }
      switch variation {
      case .underBadge, .sameStyle:
        return minutes
      case .withSessions:
        let plays = Int(item.practice?.sessionCount ?? 0)
        return "\(minutes), from \(plays) \(plays == 1 ? "session" : "sessions")"
      }
    }
  }

  extension UsualTimeSketch {
    static var learned: LibraryItemView {
      .fixture(
        id: "ex-scales", itemType: .exercise, title: "Major scales",
        practice: .fixture(sessionCount: 6, totalMinutes: 49, latestScore: 6),
        usualMinutes: 8, usualMinutesFromHistory: true)
    }

    static var notYetLearned: LibraryItemView {
      .fixture(
        id: "ex-enclosures", itemType: .exercise, title: "ii V i enclosures",
        usualMinutes: 10, usualMinutesFromHistory: false)
    }

    static var longTitle: LibraryItemView {
      .fixture(
        id: "ex-contrary", itemType: .exercise,
        title: "Major and minor scales in contrary motion, four octaves",
        tags: ["technique", "warm-up"],
        practice: .fixture(sessionCount: 6, totalMinutes: 49, latestScore: 6),
        usualMinutes: 8, usualMinutesFromHistory: true)
    }
  }

  #Preview("Usual time · A learned") {
    UsualTimeSketch(item: UsualTimeSketch.learned, variation: .underBadge)
  }
  #Preview("Usual time · B learned") {
    UsualTimeSketch(item: UsualTimeSketch.learned, variation: .withSessions)
  }
  #Preview("Usual time · not yet learned") {
    UsualTimeSketch(item: UsualTimeSketch.notYetLearned, variation: .sameStyle)
  }
#endif
