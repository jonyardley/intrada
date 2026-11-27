#if DEBUG
  import SharedTypes
  import SwiftUI

  struct OverrunSketch {
    struct Notice {
      let onDetail: String
      let offDetail: String
    }

    static let two = Notice(
      onDetail: "4\u{00A0}minutes over, so Bach and Clair de lune drop to 3\u{00A0}minutes each.",
      offDetail:
        "Bach and Clair de lune keep 5\u{00A0}minutes. You'll finish 4\u{00A0}minutes late.")

    static let squeezed = Notice(
      onDetail:
        "4\u{00A0}minutes over, so Bach and Clair de lune drop to 1\u{00A0}minute each. You'll still finish 2\u{00A0}minutes late.",
      offDetail:
        "Bach and Clair de lune keep 5\u{00A0}minutes. You'll finish 4\u{00A0}minutes late.")

    static let three = Notice(
      onDetail: "4\u{00A0}minutes over, so the next 3 items drop to 2\u{00A0}minutes each.",
      offDetail: "The next 3 items keep their full times. You'll finish 4\u{00A0}minutes late.")

    static let switchTitle = "Finish on time"
    static let overLabel = "4 min over"
  }

  // ── Item complete, drawn from its public parts with fixture values ──

  struct OverrunSheetSketch: View {
    var notice: OverrunSketch.Notice = OverrunSketch.two
    var shortenOn = true

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

              HairlineDivider().padding(.top, IntradaSpacing.controlGap)
              StepperRow {
                StepperRowLabel(
                  OverrunSketch.switchTitle,
                  detail: shortenOn ? notice.onDetail : notice.offDetail
                )
                .fixedSize(horizontal: false, vertical: true)
              } stepper: {
                Toggle(OverrunSketch.switchTitle, isOn: .constant(shortenOn))
                  .labelsHidden()
                  .fixedSize()
                  .tint(IntradaColor.accent)
                  .accessibilityIdentifier("reflection.shorten")
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

  // ── The player's top, with the real band and the over-time line ──

  struct OverrunPlayerSketch: View {
    var body: some View {
      PlayerFrameSketch(
        remaining: 17 * 60 + 40, position: "Focus · 1 of 3", filled: 1, kind: .exercise,
        title: "Major scales", next: "Then Bach, now 3 min"
      ) {
        Text(OverrunSketch.overLabel)
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
          .frame(maxWidth: .infinity)
          .padding(.top, IntradaSpacing.controlGap)
      } ring: {
        RingStandInSketch(figure: "+4:00", caption: "of 5:00")
      }
    }
  }

  struct PlayerFrameSketch<Strip: View, Ring: View>: View {
    let remaining: Int
    let position: String
    let filled: Int
    let kind: ItemKind
    let title: String
    let next: String
    @ViewBuilder let strip: Strip
    @ViewBuilder let ring: Ring

    var body: some View {
      ZStack {
        PaperBackground()
        VStack(spacing: 0) {
          SessionOrientationBand(
            session: .remaining(remaining), positionLabel: position,
            types: [.exercise, .piece, .piece], filled: filled
          ) {
            Image(systemName: "ellipsis")
              .iconSize(.control)
              .foregroundStyle(IntradaColor.inkSecondary)
              .frame(width: 44, height: 44)
          }
          strip
          Spacer(minLength: IntradaSpacing.card)
          VStack(spacing: IntradaSpacing.controlGap) {
            TypeBadge(kind: kind)
            Text(title)
              .font(IntradaFont.pageTitle)
              .foregroundStyle(IntradaColor.ink)
          }
          ring.padding(.top, IntradaSpacing.section)
          Spacer(minLength: IntradaSpacing.card)
          Text(next)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .padding(.bottom, IntradaSpacing.section)
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.top, IntradaSpacing.card)
      }
    }
  }

  struct RingStandInSketch: View {
    let figure: String
    let caption: String

    var body: some View {
      ZStack {
        Circle().stroke(IntradaColor.ink, lineWidth: 12)
        VStack(spacing: IntradaSpacing.xxSmall) {
          Text(figure).font(IntradaFont.figureLarge).foregroundStyle(IntradaColor.ink)
          Text(caption).font(IntradaFont.secondary).foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(width: 210, height: 210)
    }
  }
#endif
