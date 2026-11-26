#if DEBUG
  import SharedTypes
  import SwiftUI

  struct PlanFromTimeSketch: View {
    enum Figure { case standard, timer }
    enum WarmUp {
      case off, empty
      case row(Row)
    }

    struct Row: Identifiable {
      let id = UUID()
      let title: String
      let reason: String
      let minutes: Int
      let kind: ItemKind
    }

    var figure: Figure = .standard
    let minutes: Int
    let warmUp: WarmUp
    let rows: [Row]
    private let range = 5...120

    var body: some View {
      ScreenScaffold(title: "Practice", subtitle: "Good morning", trailingPlacement: .header) {
        ProfileBadge(icon: .piano, size: IntradaGlyph.bar)
          .frame(width: 44, height: 44)
          .accessibilityLabel("Profile")
      } content: {
        ScrollView {
          VStack(alignment: .leading, spacing: IntradaSpacing.section) {
            lengthSection
            warmUpSection
            sessionSection
            SectionHeader(title: "This week", trailing: "2 of 5 days")
          }
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.top, IntradaSpacing.card)
          .padding(.bottom, IntradaSpacing.section)
        }
        .safeAreaInset(edge: .bottom, spacing: 0) { startBar }
      }
    }

    // MARK: - Length

    private var lengthSection: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        SectionTitle("How long have you got?")
        lengthControl
          .frame(maxWidth: .infinity)
          .padding(IntradaSpacing.card)
          .cardSurface()
      }
    }

    private var lengthControl: some View {
      HStack(spacing: 0) {
        TempoStepButton(systemImage: "minus", label: "Shorter", isEnabled: canLower) {}
        VStack(spacing: 0) {
          Text("\(minutes)")
            .font(figure == .standard ? IntradaFont.figureLarge : IntradaFont.timer(40))
            .monospacedDigit()
            .foregroundStyle(IntradaColor.ink)
          Text("minutes")
            .font(IntradaFont.small)
            .foregroundStyle(IntradaColor.inkSecondary)
            .fixedSize()
        }
        .padding(.horizontal, IntradaSpacing.controlGap)
        .frame(minWidth: StepperValue.width, minHeight: 44)
        TempoStepButton(systemImage: "plus", label: "Longer", isEnabled: canRaise) {}
      }
      .fixedSize()
      .accessibilityElement(children: .ignore)
      .accessibilityLabel("Session length")
      .accessibilityValue("\(minutes) minutes")
      .accessibilityAdjustableAction { direction in
        switch direction {
        case .increment where canRaise:
          Haptic.selection.play()
        case .decrement where canLower:
          Haptic.selection.play()
        default:
          break
        }
      }
    }

    private var canLower: Bool { minutes > range.lowerBound }
    private var canRaise: Bool { minutes < range.upperBound }

    // MARK: - Plan

    @ViewBuilder private var warmUpSection: some View {
      switch warmUp {
      case .off:
        EmptyView()
      case .row(let row):
        VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
          SectionTitle("Warm-up")
          planRow(row).cardSurface()
        }
      case .empty:
        VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
          SectionTitle("Warm-up")
          VStack(alignment: .leading, spacing: IntradaSpacing.xSmall) {
            Text("No warm-up items yet")
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.ink)
            Text(
              "Turn on \u{201C}Include in warm-up?\u{201D} for a piece or exercise in the Library."
            )
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .fixedSize(horizontal: false, vertical: true)
            Button("Choose warm-up items") {}
              .font(IntradaFont.button)
              .foregroundStyle(IntradaColor.accent)
              .padding(.top, IntradaSpacing.controlGap)
          }
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(IntradaSpacing.card)
          .cardSurface()
        }
      }
    }

    private var sessionSection: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        SectionHeader(title: "Session", action: .init(title: "Change", perform: {}))
        VStack(spacing: 0) {
          ForEach(Array(rows.enumerated()), id: \.element.id) { index, row in
            if index > 0 { HairlineDivider().padding(.leading, IntradaSpacing.card) }
            planRow(row)
          }
        }
        .cardSurface()
        AddRowButton(title: "Add piece or exercise", style: .plain) {}
      }
    }

    private func planRow(_ row: Row) -> some View {
      HStack(spacing: IntradaSpacing.cardCompact) {
        row.kind.bar.frame(width: 4, height: 34).clipShape(Capsule())
        VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
          Text(row.title)
            .font(IntradaFont.cardTitle)
            .foregroundStyle(IntradaColor.ink)
            .fixedSize(horizontal: false, vertical: true)
          Text(row.reason)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        Text("\(row.minutes) min")
          .font(IntradaFont.secondary)
          .monospacedDigit()
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .padding(IntradaSpacing.cardCompact)
    }

    private var planned: Int {
      let warm = if case .row(let row) = warmUp { row.minutes } else { 0 }
      return rows.map(\.minutes).reduce(warm, +)
    }

    private var startBar: some View {
      BrandBarButton {
      } label: {
        Image(systemName: "play.fill")
        Text("Start session · \(planned) min")
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.cardCompact)
      .padding(.bottom, IntradaSpacing.section)
      .background(IntradaColor.paperTop)
      .overlay(alignment: .top) { Rectangle().fill(IntradaColor.hairline).frame(height: 1) }
    }
  }

  // MARK: - Fixtures

  extension PlanFromTimeSketch {
    typealias R = Row
    static let scales = R(title: "Major scales", reason: "Exercise", minutes: 3, kind: .exercise)
    static let bach = R(
      title: "Bach Prelude in C", reason: "Piece · a priority", minutes: 10, kind: .piece)
    static let enclosures = R(
      title: "ii V i enclosures", reason: "Exercise · going cold after 3 weeks", minutes: 7,
      kind: .exercise)
    static let clair = R(
      title: "Clair de lune", reason: "Piece · cold for 5 weeks", minutes: 10, kind: .piece)

    static func typical(_ f: Figure = .standard) -> Self {
      Self(figure: f, minutes: 20, warmUp: .row(scales), rows: [bach, enclosures])
    }
    static func emptyWarmUp(_ f: Figure = .standard) -> Self {
      Self(figure: f, minutes: 20, warmUp: .empty, rows: [bach, enclosures])
    }
    static let warmUpOff = Self(minutes: 20, warmUp: .off, rows: [bach, clair])
    static let underLength = Self(
      minutes: 20, warmUp: .row(scales),
      rows: [R(title: "Bach Prelude in C", reason: "Piece · a priority", minutes: 14, kind: .piece)]
    )
    static let lowerLimit = Self(
      minutes: 5, warmUp: .off,
      rows: [
        R(
          title: "Major scales", reason: "Exercise · going cold after 3 weeks", minutes: 5,
          kind: .exercise)
      ])

    static func longest(_ f: Figure = .standard) -> Self {
      Self(
        figure: f, minutes: 60,
        warmUp: .row(
          R(
            title: "Chromatic scales in contrary motion, two octaves", reason: "Exercise",
            minutes: 5, kind: .exercise)),
        rows: [
          R(
            title: "Prelude and Fugue in C sharp minor, BWV 849", reason: "Piece · a priority",
            minutes: 15, kind: .piece),
          R(
            title: "Rachmaninoff Prelude in G minor, Op. 23 No. 5",
            reason: "Piece · cold for 5 weeks", minutes: 12, kind: .piece),
          R(
            title: "Arpeggios in tenths, hands together",
            reason: "Exercise · going cold after 3 weeks", minutes: 10, kind: .exercise),
          R(
            title: "Chopin Ballade No. 1 in G minor, Op. 23", reason: "Piece · a priority",
            minutes: 10, kind: .piece),
          R(
            title: "Hanon exercises 1 to 5", reason: "Exercise · going cold after 3 weeks",
            minutes: 8, kind: .exercise),
        ])
    }
  }

  #Preview("Typical, timer figure") { PlanFromTimeSketch.typical(.timer) }
#endif
