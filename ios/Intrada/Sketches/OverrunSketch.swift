#if DEBUG
  import SharedTypes
  import SwiftUI

  struct OverrunSketch {
    enum Option { case switchRow, lineUnderSave, playerOnly }

    struct Notice {
      let switchDetail: String
      let switchOffDetail: String
      let underSave: String
      let keepLabel: String
    }

    static let two = Notice(
      switchDetail: "Bach and Clair de lune now 3\u{00A0}minutes each",
      switchOffDetail: "Ends 4\u{00A0}minutes late",
      underSave: "Bach and Clair de lune now 3\u{00A0}minutes each",
      keepLabel: "Keep 5\u{00A0}min")

    static let squeezed = Notice(
      switchDetail: "Bach and Clair de lune now 1\u{00A0}minute each",
      switchOffDetail: "Ends 12\u{00A0}minutes late",
      underSave: "Bach and Clair de lune now 1\u{00A0}minute each",
      keepLabel: "Keep 5\u{00A0}min")

    static let switchTitle = "Finish on time"
    static let overLabel = "4 min over"
  }

  // ── Item complete, drawn from its public parts with fixture values ──

  struct OverrunSheetSketch: View {
    var option: OverrunSketch.Option = .switchRow
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

              if option == .switchRow {
                HairlineDivider().padding(.top, IntradaSpacing.controlGap)
                VStack(alignment: .leading, spacing: 0) {
                  SwitchRow(
                    title: OverrunSketch.switchTitle, isOn: .constant(shortenOn),
                    identifier: "reflection.shorten")
                  Text(shortenOn ? notice.switchDetail : notice.switchOffDetail)
                    .font(IntradaFont.small)
                    .foregroundStyle(IntradaColor.inkSecondary)
                    .fixedSize(horizontal: false, vertical: true)
                }
                .padding(.top, IntradaSpacing.controlGap)
              }
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

            if option == .lineUnderSave {
              UnderSaveLineSketch(notice: notice)
                .padding(.top, IntradaSpacing.controlGap)
            }

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

  struct UnderSaveLineSketch: View {
    let notice: OverrunSketch.Notice

    var body: some View {
      ViewThatFits(in: .horizontal) {
        HStack(spacing: IntradaSpacing.controlGap) {
          words
          keep
        }
        VStack(spacing: 0) {
          words.multilineTextAlignment(.center)
          keep
        }
      }
      .frame(maxWidth: .infinity)
    }

    private var words: some View {
      Text(notice.underSave)
        .font(IntradaFont.secondary)
        .foregroundStyle(IntradaColor.inkSecondary)
        .fixedSize(horizontal: false, vertical: true)
    }

    private var keep: some View {
      Button(notice.keepLabel) {}
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(minHeight: 44)
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
      } underRing: {
        EmptyView()
      }
    }
  }

  // ── Option C: the next item's player carries the shorter time ──

  struct OverrunNextPlayerSketch: View {
    var body: some View {
      PlayerFrameSketch(
        remaining: 6 * 60, position: "Focus · 2 of 3", filled: 1, kind: .piece,
        title: "Bach", next: "Then Clair de lune, now 3 min"
      ) {
        EmptyView()
      } ring: {
        RingStandInSketch(figure: "3:00", caption: "now 3 min, was 5")
      } underRing: {
        Button("Keep 5 min") {}
          .font(IntradaFont.button)
          .foregroundStyle(IntradaColor.inkSecondary)
          .frame(minHeight: 44)
          .padding(.top, IntradaSpacing.controlGap)
      }
    }
  }

  struct PlayerFrameSketch<Strip: View, Ring: View, UnderRing: View>: View {
    let remaining: Int
    let position: String
    let filled: Int
    let kind: ItemKind
    let title: String
    let next: String
    @ViewBuilder let strip: Strip
    @ViewBuilder let ring: Ring
    @ViewBuilder let underRing: UnderRing

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
          underRing
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
