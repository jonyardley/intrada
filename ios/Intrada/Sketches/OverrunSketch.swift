#if DEBUG
  import SharedTypes
  import SwiftUI

  struct OverrunSketch {
    enum Look: CaseIterable { case sunken, card, plain }

    struct Notice {
      let headline: String
      let detail: String
      let canKeep: Bool
    }

    static let two = Notice(
      headline: "You spent 4 minutes longer on scales.",
      detail: "To finish on time, Bach and Clair de lune lose 2 minutes each.",
      canKeep: true)

    static let one = Notice(
      headline: "You spent 4 minutes longer on scales.",
      detail: "To finish on time, Bach loses 4 minutes.",
      canKeep: true)

    static let many = Notice(
      headline: "You spent 4 minutes longer on Hanon No. 1 in C major, hands together.",
      detail:
        "To finish on time, Bach Prelude in C, Clair de lune and Chopin Nocturne in E flat lose 4 minutes between them.",
      canKeep: true)

    static let squeezed = Notice(
      headline: "You spent 12 minutes longer on scales.",
      detail: "To finish on time, Bach and Clair de lune drop to 1 minute each.",
      canKeep: true)

    static let kept = Notice(
      headline: "You spent 4 minutes longer on scales.",
      detail: "Bach and Clair de lune keep their full times. You will finish about 4 minutes late.",
      canKeep: false)

    static let overLabel = "4 min over"
    static let overDetail = "The rest will be shortened"
  }

  // ── Item complete, drawn from its public parts with fixture values ──

  struct OverrunSheetSketch: View {
    var look: OverrunSketch.Look = .sunken
    var notice: OverrunSketch.Notice = OverrunSketch.two

    var body: some View {
      ZStack(alignment: .bottom) {
        PaperBackground()
        ScrollView {
          VStack(alignment: .leading, spacing: 0) {
            VStack(spacing: IntradaSpacing.controlGap) {
              SectionTitle("Item complete · 9:00")
              Text("How did it go?")
                .font(IntradaFont.title).foregroundStyle(IntradaColor.ink)
              Text("Major scales")
                .font(IntradaFont.secondary).foregroundStyle(IntradaColor.inkSecondary)
            }
            .frame(maxWidth: .infinity)
            .padding(.top, IntradaSpacing.card)

            HairlineDivider().padding(.top, IntradaSpacing.section)

            if look != .plain {
              OverrunNoticeSketch(look: look, notice: notice)
                .padding(.top, IntradaSpacing.section)
            }

            VStack(alignment: .leading, spacing: 0) {
              StepperRow {
                StepperRowLabel("Confidence")
              } stepper: {
                ScoreStepper(
                  score: 6, origin: .lastTime, range: 1...10, accessibilityLabel: "Score"
                ) { _ in }
              }
              .padding(.bottom, IntradaSpacing.controlGap)
              HairlineDivider()
              StepperRow {
                StepperRowLabel("Tempo reached")
              } stepper: {
                TempoDrag(value: .constant(104), step: 2, band: 40...208)
              }
              .padding(.top, IntradaSpacing.controlGap)
            }
            .padding(.top, IntradaSpacing.section)

            SectionTitle("Reflection · optional")
              .frame(maxWidth: .infinity, alignment: .leading)
              .padding(.top, IntradaSpacing.section)
            NoteBox {
              Text("What went well? What to fix next time?")
                .font(IntradaFont.body)
                .foregroundStyle(IntradaColor.inkFainter)
                .frame(maxWidth: .infinity, minHeight: 66, alignment: .topLeading)
            }
            .padding(.top, IntradaSpacing.controlGap)

            if look == .plain {
              OverrunNoticeSketch(look: look, notice: notice)
                .padding(.top, IntradaSpacing.card)
            }

            BrandBarButton {
            } label: {
              Text("Save & continue")
              Image(systemName: "arrow.right")
            }
            .padding(.top, IntradaSpacing.card)

            Button("Skip") {}
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.inkSecondary)
              .frame(maxWidth: .infinity)
              .padding(.top, IntradaSpacing.cardCompact)
          }
          .padding(.horizontal, IntradaSpacing.section)
          .padding(.bottom, IntradaSpacing.section)
        }
      }
    }
  }

  struct OverrunNoticeSketch: View {
    let look: OverrunSketch.Look
    let notice: OverrunSketch.Notice

    var body: some View {
      switch look {
      case .sunken:
        HStack(alignment: .top, spacing: IntradaSpacing.controlGap) {
          clockIcon.padding(.top, IntradaSpacing.xxSmall)
          VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
            words
            keepButton(underlined: true)
          }
        }
        .padding(.horizontal, IntradaSpacing.cardCompact)
        .padding(.top, IntradaSpacing.cardCompact)
        .padding(.bottom, notice.canKeep ? 0 : IntradaSpacing.cardCompact)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
          IntradaColor.surfaceSunken, in: RoundedRectangle(cornerRadius: IntradaRadius.card))
      case .card:
        HStack(alignment: .top, spacing: IntradaSpacing.controlGap) {
          clockIcon.padding(.top, IntradaSpacing.xxSmall)
          VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
            words
            keepButton(underlined: false)
          }
        }
        .padding(.horizontal, IntradaSpacing.cardCompact)
        .padding(.top, IntradaSpacing.cardCompact)
        .padding(.bottom, notice.canKeep ? 0 : IntradaSpacing.cardCompact)
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardSurface(cornerRadius: IntradaRadius.card)
      case .plain:
        VStack(spacing: IntradaSpacing.xxSmall) {
          Text(notice.headline)
            .font(IntradaFont.label)
            .foregroundStyle(IntradaColor.ink)
          Text(notice.detail)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
          keepButton(underlined: false)
        }
        .multilineTextAlignment(.center)
        .frame(maxWidth: .infinity)
      }
    }

    private var clockIcon: some View {
      Image(systemName: "clock")
        .iconSize(.inline)
        .foregroundStyle(IntradaColor.inkSecondary)
        .accessibilityHidden(true)
    }

    private var words: some View {
      VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
        Text(notice.headline)
          .font(IntradaFont.label)
          .foregroundStyle(IntradaColor.ink)
        Text(notice.detail)
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .fixedSize(horizontal: false, vertical: true)
    }

    @ViewBuilder private func keepButton(underlined: Bool) -> some View {
      if notice.canKeep {
        Button {
        } label: {
          Text("Keep their full times").underline(underlined)
        }
        .font(IntradaFont.button)
        .foregroundStyle(IntradaColor.ink)
        .frame(minHeight: 44)
      }
    }
  }

  // ── The player's top, with the real band and the over-time strip ──

  struct OverrunPlayerSketch: View {
    var look: OverrunSketch.Look = .sunken

    var body: some View {
      ZStack {
        PaperBackground()
        VStack(spacing: 0) {
          SessionOrientationBand(
            session: .remaining(17 * 60 + 40), positionLabel: "Focus · 1 of 3",
            types: [.exercise, .piece, .piece], filled: 1
          ) {
            Image(systemName: "ellipsis")
              .iconSize(.control)
              .foregroundStyle(IntradaColor.inkSecondary)
              .frame(width: 44, height: 44)
          }
          OverrunStripSketch(look: look)
            .padding(.top, IntradaSpacing.controlGap)
          Spacer(minLength: IntradaSpacing.card)
          VStack(spacing: IntradaSpacing.controlGap) {
            TypeBadge(kind: .exercise)
            Text("Major scales")
              .font(IntradaFont.pageTitle)
              .foregroundStyle(IntradaColor.ink)
          }
          ringStandIn.padding(.top, IntradaSpacing.section)
          Spacer(minLength: IntradaSpacing.card)
          Text("Then Bach, now 3 min")
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .padding(.bottom, IntradaSpacing.section)
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
      }
    }

    private var ringStandIn: some View {
      ZStack {
        Circle().stroke(IntradaColor.ink, lineWidth: 12)
        VStack(spacing: IntradaSpacing.xxSmall) {
          Text("+4:00").font(IntradaFont.figureLarge).foregroundStyle(IntradaColor.ink)
          Text("of 5:00").font(IntradaFont.secondary).foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(width: 210, height: 210)
    }
  }

  struct OverrunStripSketch: View {
    let look: OverrunSketch.Look

    var body: some View {
      switch look {
      case .sunken:
        HStack(spacing: IntradaSpacing.controlGap) {
          clockIcon
          Text(OverrunSketch.overLabel)
            .font(IntradaFont.smallMedium)
            .foregroundStyle(IntradaColor.ink)
          Spacer(minLength: IntradaSpacing.controlGap)
          Text(OverrunSketch.overDetail)
            .font(IntradaFont.small)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        .padding(.horizontal, IntradaSpacing.cardCompact)
        .padding(.vertical, IntradaSpacing.controlGap)
        .background(
          IntradaColor.surfaceSunken, in: RoundedRectangle(cornerRadius: IntradaRadius.card))
      case .card:
        HStack(spacing: IntradaSpacing.controlGap) {
          clockIcon
          Text("\(OverrunSketch.overLabel). \(OverrunSketch.overDetail).")
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.horizontal, IntradaSpacing.cardCompact)
        .padding(.vertical, IntradaSpacing.cardCompact)
        .cardSurface(cornerRadius: IntradaRadius.card)
      case .plain:
        Text(OverrunSketch.overLabel)
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
          .frame(maxWidth: .infinity)
      }
    }

    private var clockIcon: some View {
      Image(systemName: "clock")
        .iconSize(.inline)
        .foregroundStyle(IntradaColor.inkSecondary)
        .accessibilityHidden(true)
    }
  }
#endif
