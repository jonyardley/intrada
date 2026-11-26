#if DEBUG
  import SharedTypes
  import SwiftUI

  struct TodaysKeysSketch {
    enum Labels { case chips, line }
    enum Marking { case caption, grouped }

    struct KeyLine: Identifiable {
      var id: String { name }
      let name: String
      let score: Int?
      let when: String
      let suggested: Bool

      var record: String {
        guard let score else { return "Not practised yet" }
        return "Scored \(score), \(when)"
      }
    }

    struct Row: Identifiable {
      let id = UUID()
      let title: String
      let reason: String
      let minutes: Int
      let kind: ItemKind
      let keys: [String]
    }

    static var scaleKeys: [KeyLine] {
      [
        KeyLine(name: "C major", score: 7, when: "yesterday", suggested: false),
        KeyLine(name: "G major", score: 3, when: "2 weeks ago", suggested: true),
        KeyLine(name: "D major", score: 6, when: "4 days ago", suggested: false),
        KeyLine(name: "A major", score: 8, when: "yesterday", suggested: false),
        KeyLine(name: "E major", score: 5, when: "last week", suggested: false),
        KeyLine(name: "B major", score: 6, when: "last week", suggested: false),
        KeyLine(name: "F\u{266F} major", score: nil, when: "", suggested: false),
        KeyLine(name: "F major", score: 7, when: "3 days ago", suggested: false),
        KeyLine(name: "B\u{266D} major", score: 4, when: "3 weeks ago", suggested: true),
        KeyLine(name: "E\u{266D} major", score: 6, when: "last week", suggested: false),
        KeyLine(name: "A minor", score: 7, when: "4 days ago", suggested: false),
        KeyLine(name: "D minor", score: 2, when: "2 weeks ago", suggested: true),
      ]
    }

    static var scales: Row {
      Row(
        title: "Scales", reason: "Exercise \u{00B7} weakest keys first", minutes: 9,
        kind: .exercise, keys: ["D minor", "G major", "B\u{266D} major"])
    }
    static var bach: Row {
      Row(
        title: "Bach Prelude in C", reason: "Piece \u{00B7} a priority", minutes: 10, kind: .piece,
        keys: [])
    }
    static var enclosures: Row {
      Row(
        title: "ii V i enclosures", reason: "Exercise \u{00B7} going cold after 3 weeks",
        minutes: 7, kind: .exercise, keys: ["F major", "E\u{266D} major"])
    }
  }

  struct TodaysKeysRowSketch: View {
    var labels: TodaysKeysSketch.Labels = .chips
    var rows: [TodaysKeysSketch.Row] = [
      TodaysKeysSketch.bach, TodaysKeysSketch.scales, TodaysKeysSketch.enclosures,
    ]

    var body: some View {
      ScreenScaffold(title: "Practice", subtitle: "Good morning", trailingPlacement: .header) {
        ProfileBadge(icon: .piano, size: IntradaGlyph.bar)
          .frame(width: 44, height: 44)
          .accessibilityLabel("Profile")
      } content: {
        ScrollView {
          VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
            SectionHeader(title: "Session", action: .init(title: "Change", perform: {}))
            VStack(spacing: 0) {
              ForEach(Array(rows.enumerated()), id: \.element.id) { index, row in
                if index > 0 { HairlineDivider().padding(.leading, IntradaSpacing.card) }
                planRow(row)
              }
            }
            .cardSurface()
          }
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.top, IntradaSpacing.card)
          .padding(.bottom, IntradaSpacing.section)
        }
      }
    }

    private func planRow(_ row: TodaysKeysSketch.Row) -> some View {
      HStack(alignment: .top, spacing: IntradaSpacing.cardCompact) {
        row.kind.bar.frame(width: 4, height: 34).clipShape(Capsule())
          .padding(.top, IntradaSpacing.xSmall)
        VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
          Text(row.title)
            .font(IntradaFont.cardTitle)
            .foregroundStyle(IntradaColor.ink)
            .fixedSize(horizontal: false, vertical: true)
          Text(row.reason)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .fixedSize(horizontal: false, vertical: true)
          if !row.keys.isEmpty { keysButton(row) }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        Text("\(row.minutes) min")
          .font(IntradaFont.secondary)
          .monospacedDigit()
          .foregroundStyle(IntradaColor.inkSecondary)
          .padding(.top, IntradaSpacing.xxSmall)
      }
      .padding(IntradaSpacing.cardCompact)
    }

    private func keysButton(_ row: TodaysKeysSketch.Row) -> some View {
      Button {
      } label: {
        Group {
          switch labels {
          case .chips:
            FlowLayout(spacing: IntradaSpacing.xSmall) {
              ForEach(row.keys, id: \.self) { TagChip($0) }
            }
          case .line:
            HStack(spacing: IntradaSpacing.xSmall) {
              Text(row.keys.joined(separator: ", "))
                .font(IntradaFont.smallMedium)
                .foregroundStyle(IntradaColor.ink)
                .multilineTextAlignment(.leading)
                .fixedSize(horizontal: false, vertical: true)
              Image(systemName: "chevron.right")
                .iconSize(.caption, weight: .semibold)
                .foregroundStyle(IntradaColor.inkFaintIcon)
            }
          }
        }
        .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
        .contentShape(Rectangle())
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Today's keys: \(row.keys.joined(separator: ", "))")
      .accessibilityHint("Changes today's keys for \(row.title)")
      .accessibilityIdentifier("practice.plan.keys")
    }
  }

  struct TodaysKeysSheetSketch: View {
    var marking: TodaysKeysSketch.Marking = .caption
    var keys: [TodaysKeysSketch.KeyLine] = TodaysKeysSketch.scaleKeys

    var body: some View {
      BottomSheet(title: "Today's keys", detents: [.large]) {
        ScrollView {
          VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
            VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
              Text("Scales, 9 min")
                .font(IntradaFont.cardTitle)
                .foregroundStyle(IntradaColor.ink)
              Text("Your three weakest keys, about 3 minutes each.")
                .font(IntradaFont.secondary)
                .foregroundStyle(IntradaColor.inkSecondary)
                .fixedSize(horizontal: false, vertical: true)
            }
            .padding(.horizontal, IntradaSpacing.card)
            switch marking {
            case .caption:
              list(keys)
            case .grouped:
              SectionTitle("Suggested").padding(.horizontal, IntradaSpacing.card)
              list(keys.filter(\.suggested))
              SectionTitle("Other keys").padding(.horizontal, IntradaSpacing.card)
              list(keys.filter { !$0.suggested })
            }
            Text("Ticking more keys keeps 9 minutes and gives each key less time.")
              .font(IntradaFont.small)
              .foregroundStyle(IntradaColor.inkSecondary)
              .fixedSize(horizontal: false, vertical: true)
              .padding(.horizontal, IntradaSpacing.card)
          }
          .padding(.vertical, IntradaSpacing.card)
        }
      }
    }

    private func list(_ lines: [TodaysKeysSketch.KeyLine]) -> some View {
      VStack(spacing: 0) {
        ForEach(Array(lines.enumerated()), id: \.element.id) { index, line in
          if index > 0 { HairlineDivider().padding(.leading, IntradaSpacing.card) }
          TickRow(
            label: line.name, caption: caption(line), chosen: line.suggested,
            identifier: "todaysKeys.row", toggle: {})
        }
      }
      .cardSurface()
      .padding(.horizontal, IntradaSpacing.card)
    }

    private func caption(_ line: TodaysKeysSketch.KeyLine) -> String {
      guard marking == .caption, line.suggested else { return line.record }
      return "Suggested, \(line.record.prefix(1).lowercased())\(line.record.dropFirst())"
    }
  }
#endif
