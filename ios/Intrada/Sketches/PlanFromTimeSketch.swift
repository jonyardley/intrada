#if DEBUG
  import SharedTypes
  import SwiftUI

  struct PlanFromTimeSketch: View {
    enum LengthLook { case rings, ringsWithHint, systemStepper }

    struct Row: Identifiable {
      let id = UUID()
      let title: String
      let reason: String
      let minutes: Int
      let kind: ItemKind
    }

    var look: LengthLook = .rings
    let minutes: Int
    var usualMinutes = 30
    let warmUp: Row?
    let rows: [Row]

    var body: some View {
      ScreenScaffold(title: "Practice", subtitle: "Thursday 26 November") {
        VStack(alignment: .leading, spacing: IntradaSpacing.section) {
          lengthSection
          VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
            SectionTitle("Suggested plan")
            warmUpSection
            sessionSection
            AddRowButton(title: "Add piece or exercise") {}
          }
          MarkerButton("Start session") {}
          SectionHeader(title: "This week", trailing: "2 of 5 days")
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
        .frame(maxHeight: .infinity, alignment: .top)
      }
    }

    // MARK: - Length

    private var lengthSection: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        SectionTitle("How long have you got?")
        VStack(spacing: IntradaSpacing.controlGap) {
          lengthControl
          if look == .ringsWithHint {
            Text("5 minutes a tap")
              .font(IntradaFont.small)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
          Text("Usual length \(usualMinutes) minutes, set in Profile")
            .font(IntradaFont.small)
            .foregroundStyle(IntradaColor.inkSecondary)
            .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
        .padding(IntradaSpacing.card)
        .cardSurface()
      }
    }

    @ViewBuilder private var lengthControl: some View {
      switch look {
      case .rings, .ringsWithHint:
        HStack(spacing: 0) {
          TempoStepButton(systemImage: "minus", label: "Shorter") {}
          VStack(spacing: 0) {
            Text("\(minutes)")
              .font(IntradaFont.figureLarge)
              .foregroundStyle(IntradaColor.ink)
            Text("minutes")
              .font(IntradaFont.small)
              .foregroundStyle(IntradaColor.inkSecondary)
              .fixedSize()
          }
          .padding(.horizontal, IntradaSpacing.controlGap)
          .frame(minWidth: StepperValue.width, minHeight: 44)
          TempoStepButton(systemImage: "plus", label: "Longer") {}
        }
        .fixedSize()
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("How long have you got")
        .accessibilityValue("\(minutes) minutes")
      case .systemStepper:
        HStack(spacing: IntradaSpacing.cardCompact) {
          Text("\(minutes) minutes")
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
          Spacer(minLength: 0)
          Stepper("Session length", value: .constant(minutes), in: 5...120, step: 5)
            .labelsHidden()
            .fixedSize()
        }
      }
    }

    // MARK: - Plan

    @ViewBuilder private var warmUpSection: some View {
      SectionHeader(title: "Warm-up", trailing: warmUp.map { "\($0.minutes) min" })
      if let warmUp {
        planRow(warmUp).cardSurface()
      } else {
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
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.accent)
            .padding(.top, IntradaSpacing.xSmall)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(IntradaSpacing.card)
        .cardSurface()
      }
    }

    @ViewBuilder private var sessionSection: some View {
      SectionHeader(title: "Your session", trailing: "\(rows.map(\.minutes).reduce(0, +)) min")
      VStack(spacing: 0) {
        ForEach(Array(rows.enumerated()), id: \.element.id) { index, row in
          if index > 0 { HairlineDivider().padding(.leading, IntradaSpacing.card) }
          planRow(row)
        }
      }
      .cardSurface()
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
        CapsuleActionButton(title: "\(row.minutes) min") {}
      }
      .padding(IntradaSpacing.cardCompact)
    }
  }

  // MARK: - Fixtures

  extension PlanFromTimeSketch {
    static let scales = Row(
      title: "Major scales", reason: "Exercise, warm-up", minutes: 3, kind: .exercise)

    static func emptyWarmUp(_ look: LengthLook = .rings) -> Self {
      Self(
        look: look, minutes: 30, warmUp: nil,
        rows: [
          Row(title: "Bach Prelude in C", reason: "Piece, priority", minutes: 12, kind: .piece),
          Row(title: "Major scales", reason: "Exercise, going cold", minutes: 8, kind: .exercise),
          Row(
            title: "Clair de lune", reason: "Piece, not played for 16 days", minutes: 10,
            kind: .piece),
        ])
    }

    static func typical(_ look: LengthLook = .rings) -> Self {
      Self(
        look: look, minutes: 20, warmUp: scales,
        rows: [
          Row(title: "Bach Prelude in C", reason: "Piece, priority", minutes: 10, kind: .piece),
          Row(
            title: "ii V i enclosures", reason: "Exercise, going cold", minutes: 7, kind: .exercise),
        ])
    }

    static func longest(_ look: LengthLook = .rings) -> Self {
      Self(
        look: look, minutes: 60,
        warmUp: Row(
          title: "Chromatic scales in contrary motion, two octaves", reason: "Exercise, warm-up",
          minutes: 5, kind: .exercise),
        rows: [
          Row(
            title: "Prelude and Fugue in C sharp minor, BWV 849", reason: "Piece, priority",
            minutes: 15, kind: .piece),
          Row(
            title: "Rachmaninoff Prelude in G minor, Op. 23 No. 5",
            reason: "Piece, not played for 16 days", minutes: 12, kind: .piece),
          Row(
            title: "Arpeggios in tenths, hands together", reason: "Exercise, going cold",
            minutes: 10, kind: .exercise),
          Row(
            title: "Chopin Ballade No. 1 in G minor, Op. 23", reason: "Piece, priority",
            minutes: 10, kind: .piece),
          Row(
            title: "Hanon exercises 1 to 5", reason: "Exercise, going cold", minutes: 8,
            kind: .exercise),
        ])
    }
  }

  #Preview("Empty warm-up") { ScrollView { PlanFromTimeSketch.emptyWarmUp() } }
  #Preview("Typical, caption") { ScrollView { PlanFromTimeSketch.typical(.ringsWithHint) } }
  #Preview("Typical, stepper") { ScrollView { PlanFromTimeSketch.typical(.systemStepper) } }
#endif
