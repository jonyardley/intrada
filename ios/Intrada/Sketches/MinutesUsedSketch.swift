#if DEBUG
  import SharedTypes
  import SwiftUI

  struct MinutesUsedSketch {
    enum Fit { case over, under, untimed }
    enum StepperPlace { case none, inline, below }

    struct Row: Identifiable {
      var id: String { title }
      let title: String
      let caption: String
      let minutes: Int?
      let kind: ItemKind
    }

    static let lengthMins = 60

    static func rows(for fit: Fit) -> [Row] {
      switch fit {
      case .over:
        [
          Row(title: "Major scales", caption: "Exercise", minutes: 20, kind: .exercise),
          Row(title: "Bach Prelude in C", caption: "Piece", minutes: 15, kind: .piece),
          Row(title: "ii V i enclosures", caption: "Exercise", minutes: 15, kind: .exercise),
          Row(title: "Clair de lune", caption: "Piece", minutes: 10, kind: .piece),
          Row(title: "Nocturne in E flat", caption: "Piece", minutes: 12, kind: .piece),
        ]
      case .under:
        [
          Row(title: "Bach Prelude in C", caption: "Piece", minutes: 10, kind: .piece),
          Row(title: "Hanon No. 1", caption: "Exercise", minutes: 5, kind: .exercise),
        ]
      case .untimed:
        [
          Row(title: "Bach Prelude in C", caption: "Piece", minutes: 15, kind: .piece),
          Row(title: "Major scales", caption: "Exercise", minutes: 10, kind: .exercise),
          Row(title: "Hanon No. 1", caption: "Exercise", minutes: nil, kind: .exercise),
          Row(title: "Clair de lune", caption: "Piece", minutes: 15, kind: .piece),
          Row(title: "Sight-reading", caption: "Exercise", minutes: nil, kind: .exercise),
        ]
      }
    }
  }

  struct MinutesUsedBuilderSketch: View {
    var fit: MinutesUsedSketch.Fit = .over
    var stepper: MinutesUsedSketch.StepperPlace = .none
    var opensSheet = false

    private var rows: [MinutesUsedSketch.Row] { MinutesUsedSketch.rows(for: fit) }
    private var used: Int { rows.compactMap(\.minutes).reduce(0, +) }
    private var untimed: Int { rows.filter { $0.minutes == nil }.count }
    private var length: Int { MinutesUsedSketch.lengthMins }
    private var isOver: Bool { used > length }

    var body: some View {
      ScreenScaffold(
        title: "Build session",
        subtitle: "\(rows.count) items",
        leadingContent: { Button("Cancel") {} },
        trailingContent: { Button("Edit") {} },
        content: {
          ScrollView {
            VStack(spacing: IntradaSpacing.controlGap) {
              lengthCard
              ForEach(Array(rows.enumerated()), id: \.element.id) { index, row in
                planRow(row, stepper: index == 1 ? stepper : .none)
              }
              AddRowButton(title: "Add piece or exercise") {}
            }
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.vertical, IntradaSpacing.card)
          }
          .safeAreaInset(edge: .bottom, spacing: 0) { footer }
        }
      )
    }

    // ── Length ───────────────────────────────────────────────────────────

    private var lengthCard: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        FieldLabel("Session length")
        ScoreStepper(
          score: length, caption: .unit("minutes"), range: 5...120, step: 5,
          figure: IntradaFont.timer(32), typing: true,
          accessibilityLabel: "Session length", accessibilityValue: "\(length) minutes"
        ) { _ in }
        .frame(maxWidth: .infinity)
      }
      .padding(IntradaSpacing.cardCompact)
      .cardSurface(cornerRadius: IntradaRadius.card)
    }

    // ── Rows ─────────────────────────────────────────────────────────────

    @ViewBuilder
    private func planRow(_ row: MinutesUsedSketch.Row, stepper: MinutesUsedSketch.StepperPlace)
      -> some View
    {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        HStack(spacing: IntradaSpacing.cardCompact) {
          Image(systemName: "line.3.horizontal")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.inkFaintIcon)
            .accessibilityHidden(true)
          row.kind.bar.frame(width: 4, height: 34).clipShape(Capsule())
          VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
            Text(row.title).font(IntradaFont.cardTitle).foregroundStyle(IntradaColor.ink)
              .lineLimit(2)
            Text(row.caption)
              .font(IntradaFont.small).foregroundStyle(IntradaColor.inkSecondary)
              .lineLimit(1)
          }
          Spacer(minLength: IntradaSpacing.controlGap)
          switch stepper {
          case .inline: minutesStepper(row)
          case .below: EmptyView()
          case .none: minutesFigure(row)
          }
          if opensSheet {
            Image(systemName: "chevron.right")
              .iconSize(.caption, weight: .semibold)
              .foregroundStyle(IntradaColor.inkFaintIcon)
              .frame(minHeight: 44)
              .accessibilityHidden(true)
          } else {
            Image(systemName: "xmark").font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkFaintIcon)
              .frame(width: 44, height: 44)
              .accessibilityLabel("Remove \(row.title)")
          }
        }
        .contentShape(Rectangle())
        .accessibilityElement(children: opensSheet ? .combine : .contain)
        .accessibilityAddTraits(opensSheet ? .isButton : [])
        .accessibilityHint(opensSheet ? "Opens settings" : "")
        if stepper == .below {
          HStack {
            Text("Minutes").font(IntradaFont.secondary).foregroundStyle(IntradaColor.inkSecondary)
            Spacer()
            minutesStepper(row)
          }
          .padding(.leading, IntradaSpacing.section + IntradaSpacing.card)
        }
      }
      .padding(IntradaSpacing.cardCompact)
      .cardSurface(cornerRadius: IntradaRadius.card)
    }

    private func minutesFigure(_ row: MinutesUsedSketch.Row) -> some View {
      Text(row.minutes.map { "\($0) min" } ?? "No time")
        .font(IntradaFont.secondary)
        .monospacedDigit()
        .foregroundStyle(IntradaColor.inkSecondary)
        .fixedSize()
    }

    private func minutesStepper(_ row: MinutesUsedSketch.Row) -> some View {
      ScoreStepper(
        score: row.minutes ?? 0, caption: .unit("min"), range: 1...60, typing: true,
        accessibilityLabel: "Minutes for \(row.title)",
        accessibilityValue: "\(row.minutes ?? 0) minutes"
      ) { _ in }
    }

    // ── Minutes used, offers and Start ──────────────────────────────────

    private var footer: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        fitLine
        MinutesUsedBar(used: used, length: length)
        offers
        BrandBarButton {
        } label: {
          Image(systemName: "play.fill")
          Text("Start session")
        }
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.cardCompact)
      .padding(.bottom, IntradaSpacing.section)
      .background(IntradaColor.paperTop)
      .overlay(alignment: .top) { Rectangle().fill(IntradaColor.hairline).frame(height: 1) }
    }

    private var fitLine: some View {
      MinutesUsedFitLine(used: used, length: length, untimed: untimed)
    }

    private var offers: some View {
      let titles =
        isOver ? ["Trim to fit", "Drop last item"] : ["Fill with suggestions", "Stretch to fit"]
      return ViewThatFits(in: .horizontal) {
        HStack(spacing: IntradaSpacing.section) { offerButtons(titles) }
        VStack(alignment: .leading, spacing: 0) { offerButtons(titles) }
      }
    }

    @ViewBuilder private func offerButtons(_ titles: [String]) -> some View {
      ForEach(titles, id: \.self) { title in
        Button(title) {}
          .font(IntradaFont.label)
          .foregroundStyle(IntradaColor.accent)
          .frame(minHeight: 44)
          .contentShape(Rectangle())
          .fixedSize()
      }
    }
  }

  struct MinutesUsedFitLine: View {
    let used: Int
    let length: Int
    var untimed = 0

    private var isOver: Bool { used > length }

    private var usedText: String {
      let base = "\(used) of \(length) min"
      return untimed > 0 ? "\(base), plus \(untimed) untimed" : base
    }

    private var differenceText: String {
      isOver ? "\(used - length) min over" : "\(length - used) min left"
    }

    var body: some View {
      ViewThatFits(in: .horizontal) {
        HStack(alignment: .firstTextBaseline) {
          usedLabel
          Spacer(minLength: IntradaSpacing.controlGap)
          differenceLabel
        }
        VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
          usedLabel
          differenceLabel
        }
      }
      .accessibilityElement(children: .combine)
    }

    private var usedLabel: some View {
      Text(usedText)
        .font(IntradaFont.label)
        .monospacedDigit()
        .foregroundStyle(IntradaColor.ink)
    }

    private var differenceLabel: some View {
      HStack(spacing: IntradaSpacing.xSmall) {
        if isOver {
          Image(systemName: "exclamationmark.circle")
            .iconSize(.badge, weight: .semibold)
            .accessibilityHidden(true)
        }
        Text(differenceText).font(isOver ? IntradaFont.label : IntradaFont.secondary)
          .monospacedDigit()
      }
      .foregroundStyle(isOver ? IntradaColor.danger : IntradaColor.inkSecondary)
    }
  }

  // ── The item's edit sheet ────────────────────────────────────────────

  /// `EntrySettingsSheet` section for section, with `ScoreStepper` in place
  /// of the plain stepper.
  struct MinutesUsedEntrySheetSketch: View {
    var showsFit = false

    private let minutes = 15
    private var used: Int {
      MinutesUsedSketch.rows(for: .over).compactMap(\.minutes).reduce(0, +)
    }
    private var length: Int { MinutesUsedSketch.lengthMins }

    var body: some View {
      BottomSheet(title: "Bach Prelude in C") {
        ScrollView {
          VStack(alignment: .leading, spacing: IntradaSpacing.section) {
            FieldCard("Aim") {
              TextField(
                "What are you aiming for on this one?",
                text: .constant("Even quavers in the left hand"), axis: .vertical
              )
              .lineLimit(2...4)
              .font(IntradaFont.body)
              .foregroundStyle(IntradaColor.ink)
            }
            keyRow
            FieldCard("Variation") {
              HStack {
                Text("No variation").font(IntradaFont.body).foregroundStyle(IntradaColor.ink)
                Spacer()
                Image(systemName: "chevron.up.chevron.down")
                  .imageScale(.small)
                  .foregroundStyle(IntradaColor.inkFaintIcon)
              }
            }
            durationSection
            removeButton
          }
          .padding(IntradaSpacing.card)
        }
      }
    }

    private var keyRow: some View {
      HStack(spacing: IntradaSpacing.cardCompact) {
        FieldLabel("Key")
        Spacer(minLength: IntradaSpacing.controlGap)
        Text("C major").font(IntradaFont.body).foregroundStyle(IntradaColor.ink)
        Image(systemName: "chevron.right")
          .iconSize(.caption, weight: .semibold)
          .foregroundStyle(IntradaColor.inkFaintIcon)
      }
      .frame(minHeight: 44)
      .fieldCardSurface()
    }

    private var durationSection: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        SwitchRow(
          title: "Planned duration", isOn: .constant(true),
          identifier: "entrySettings.plannedDuration")
        ScoreStepper(
          score: minutes, caption: .unit("min"), range: 1...60,
          figure: IntradaFont.timer(32), typing: true,
          accessibilityLabel: "Planned duration", accessibilityValue: "\(minutes) minutes"
        ) { _ in }
        .frame(maxWidth: .infinity)
        if showsFit {
          VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
            MinutesUsedFitLine(used: used, length: length)
            MinutesUsedBar(used: used, length: length)
          }
          .padding(.top, IntradaSpacing.cardCompact)
          .overlay(alignment: .top) {
            Rectangle().fill(IntradaColor.hairline).frame(height: 1)
          }
        }
      }
      .fieldCardSurface()
    }

    private var removeButton: some View {
      Button(role: .destructive) {
      } label: {
        Text("Remove from this session")
          .font(IntradaFont.bodyMedium)
          .frame(maxWidth: .infinity)
          .padding(.vertical, IntradaSpacing.cardCompact)
      }
      .buttonStyle(.plain)
      .foregroundStyle(IntradaColor.danger)
      .cardSurface(cornerRadius: IntradaRadius.card)
    }
  }

  /// Stands for a new bar component: minutes used against a length, with the
  /// share past the length drawn in `danger` after a gap.
  struct MinutesUsedBar: View {
    let used: Int
    let length: Int

    var body: some View {
      GeometryReader { proxy in
        let total = CGFloat(max(used, length, 1))
        let inside = CGFloat(min(used, length)) / total * proxy.size.width
        let over = CGFloat(max(used - length, 0)) / total * proxy.size.width
        HStack(spacing: IntradaSpacing.xxSmall) {
          Capsule().fill(IntradaColor.ink).frame(width: inside)
          if over > 0 {
            Capsule().fill(IntradaColor.danger).frame(width: over - IntradaSpacing.xxSmall)
          }
          Spacer(minLength: 0)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(IntradaColor.track, in: Capsule())
      }
      .frame(height: 6)
      .accessibilityHidden(true)
    }
  }
#endif
