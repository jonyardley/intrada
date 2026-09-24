import SharedTypes
import SwiftUI

/// A tempo and whether the user set it, as one value: `bpm` is not writable
/// except through `set`, so the sheet cannot report a number without also
/// reporting where it came from (#1420).
@MainActor
struct TrackedTempo {
  private(set) var bpm: Int
  private(set) var userSet = false
  /// The beat value `bpm` counts in, so a quaver tempo is not clamped against
  /// the crotchet band (#1499).
  private let unit: UInt8

  init(startingBpm: Int, unit: UInt8 = 4) {
    self.unit = unit
    bpm = TempoScale.clamp(startingBpm, unit: unit)
  }

  mutating func set(_ next: Int) {
    bpm = TempoScale.clamp(next, unit: unit)
    userSet = true
  }
}

/// One row of the sheet: a stretch of the item spent on one variation, already
/// resolved for display so the sheet stays free of core types (#1739).
struct ReflectionPlay: Identifiable, Equatable {
  let id: String
  /// The variation's label, or `nil` for an unattributed play (a piece, or an
  /// exercise with no variations).
  let variationLabel: String?
  let durationDisplay: String
  let repCount: UInt8?
  let repTarget: UInt8?
  /// Whether the core predicts this play survives the terminal drop (#1758).
  let isMarkable: Bool
  /// The stamp in its own unit, and the metre it counted in; both `nil` unstamped (#1761).
  let tempoDisplay: UInt16?
  let clickPattern: ClickState?

  var title: String { variationLabel ?? "No variation" }

  var meta: String {
    var parts = [durationDisplay]
    if let repTarget { parts.append("\(repCount ?? 0) of \(repTarget)") }
    return parts.joined(separator: " · ")
  }

  /// Reads each play's own stamped seconds: `PrepareReflection` gives the still-open play its real duration first.
  static func rows(_ plays: [VariationPlayView]) -> [ReflectionPlay] {
    plays.map { play in
      ReflectionPlay(
        id: play.id, variationLabel: play.variationLabel,
        durationDisplay: SessionClock.clockDisplay(Int(play.seconds)),
        repCount: play.repCount, repTarget: play.repTarget, isMarkable: play.isMarkable,
        tempoDisplay: play.tempoDisplay, clickPattern: play.clickPattern)
    }
  }
}

/// One row's own tempo write, since each row can be stamped in a different metre or not at all (#1761).
struct ReflectionRowTempo {
  let playId: String
  let tempo: UInt16
  /// Moved the stepper, rather than accepting the pre-fill.
  let userSet: Bool
  let click: ClickState?
}

/// What the sheet collected. `tempos` holds one entry per play, sent whether or
/// not it was touched; the core ignores the ones nobody moved (#1420, #1761).
struct ReflectionResult {
  /// Play id to mark, holding only the rows the musician actually marked.
  let marks: [String: UInt8]
  let note: String
  let tempos: [ReflectionRowTempo]
}

struct ReflectionSheet: View {
  let itemTitle: String
  let elapsedDisplay: String?
  /// The item's own declared tempo marking (the practice target), if any.
  let tempoTarget: UInt16?
  /// The beat value an unstamped row's stepper counts in, and reads `♪` for (#1499).
  let tempoUnit: UInt8
  /// The metre an unstamped row counts in, sent as that row's `click` (#1761).
  let currentClick: ClickState?
  /// What was played, in order. One row is the sheet that shipped before
  /// plays existed; several give each variation its own mark (#1739
  /// decision 10).
  let plays: [ReflectionPlay]
  /// Shown here because the player's banner sits under the sheet (#2009).
  let refusal: String?
  let onSave: (ReflectionResult) -> Void
  let onSkip: () -> Void

  @State private var marks: [String: Int] = [:]
  @State private var note: String = ""
  /// One per play, seeded from its stamp or the current click (#1761 rule 6).
  @State private var tempos: [String: TrackedTempo]

  init(
    itemTitle: String, elapsedDisplay: String?, tempoTarget: UInt16?,
    startingTempoBpm: Int = TempoScale.defaultBpm, tempoUnit: UInt8 = 4,
    currentClick: ClickState? = nil,
    plays: [ReflectionPlay],
    refusal: String? = nil,
    onSave: @escaping (ReflectionResult) -> Void,
    onSkip: @escaping () -> Void
  ) {
    self.itemTitle = itemTitle
    self.elapsedDisplay = elapsedDisplay
    self.tempoTarget = tempoTarget
    self.tempoUnit = tempoUnit
    self.currentClick = currentClick
    self.plays = plays
    self.refusal = refusal
    self.onSave = onSave
    self.onSkip = onSkip
    _tempos = State(
      initialValue: Dictionary(
        plays.filter(\.isMarkable).map { play in
          (
            play.id,
            TrackedTempo(
              startingBpm: play.tempoDisplay.map(Int.init) ?? startingTempoBpm,
              unit: play.clickPattern?.metre.unit ?? tempoUnit)
          )
        }, uniquingKeysWith: { first, _ in first }))
  }

  static func heading(elapsedDisplay: String?) -> String {
    elapsedDisplay.map { "Item complete · \($0)" } ?? "Item complete"
  }

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 0) {
        VStack(spacing: 8) {
          Text(Self.heading(elapsedDisplay: elapsedDisplay))
            .font(IntradaFont.badge).textCase(.uppercase).kerning(1.5)
            .foregroundStyle(IntradaColor.exerciseBadgeFg)
          Text("How did it go?")
            .font(IntradaFont.pageTitle(24)).foregroundStyle(IntradaColor.ink)
            .multilineTextAlignment(.center)
          Text(itemTitle)
            .font(IntradaFont.subtitle).foregroundStyle(IntradaColor.inkSecondary)
            .lineLimit(1).truncationMode(.tail)
            .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
        .padding(.top, IntradaSpacing.card)

        if plays.count > 1 {
          eyebrow("What you played").padding(.top, IntradaSpacing.section)
          playRows.padding(.top, IntradaSpacing.controlGap)
        } else if let only = plays.first {
          // The core gives every practised entry at least one play, and its sole play always predicts markable (#1758).
          eyebrow("Mark").padding(.top, IntradaSpacing.section)
          ScoreSelector(
            score: mark(for: only.id), accessibilityLabel: "Mark for \(itemTitle)"
          ) { next in
            setMark(next, for: only.id)
          }
          .padding(.top, IntradaSpacing.controlGap)

          Eyebrow(singlePlayTempoEyebrow, tint: IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, IntradaSpacing.card)
          TempoStepper(value: tempoBinding(for: only.id), unit: stepperUnit(for: only))
            .padding(.top, IntradaSpacing.controlGap)
        }

        eyebrow("Reflection · optional").padding(.top, IntradaSpacing.card)
        TextField("What went well? What to fix next time?", text: $note, axis: .vertical)
          .lineLimit(3...5)
          .font(IntradaFont.field)
          .foregroundStyle(IntradaColor.ink)
          .padding(IntradaSpacing.cardCompact)
          .cardSurface(cornerRadius: IntradaRadius.control)
          .padding(.top, IntradaSpacing.controlGap)

        if let refusal {
          FormErrorBanner(message: refusal)
            .padding(.top, IntradaSpacing.card)
            .transition(.move(edge: .top).combined(with: .opacity))
        }

        BrandBarButton {
          onSave(
            ReflectionResult(
              marks: marks.compactMapValues { $0 == 0 ? nil : UInt8($0) },
              note: note.trimmingCharacters(in: .whitespacesAndNewlines),
              tempos: plays.compactMap { play in
                tempos[play.id].map { tracked in
                  ReflectionRowTempo(
                    playId: play.id, tempo: UInt16(tracked.bpm), userSet: tracked.userSet,
                    click: play.clickPattern ?? currentClick)
                }
              }))
        } label: {
          Text("Save & continue")
          Image(systemName: "arrow.right")
        }
        .padding(.top, IntradaSpacing.card)

        Button("Skip rating") { onSkip() }
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.inkSecondary)
          .frame(maxWidth: .infinity)
          .padding(.top, IntradaSpacing.cardCompact)
      }
      .padding(.horizontal, IntradaSpacing.section)
      .padding(.bottom, IntradaSpacing.section)
    }
  }

  private var playRows: some View {
    VStack(alignment: .leading, spacing: 0) {
      ForEach(Array(plays.enumerated()), id: \.element.id) { index, play in
        if index > 0 { HairlineDivider() }
        VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
          HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.cardCompact) {
            Text(play.title)
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.ink)
              .frame(maxWidth: .infinity, alignment: .leading)
            Text(play.meta)
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
          if play.isMarkable {
            ScoreSelector(
              score: mark(for: play.id), accessibilityLabel: "Mark for \(play.title)"
            ) { next in
              setMark(next, for: play.id)
            }
            TempoStepper(
              value: tempoBinding(for: play.id), unit: stepperUnit(for: play),
              accessibilityLabel: "Tempo for \(play.title)")
          }
        }
        .padding(.vertical, IntradaSpacing.cardCompact)
      }
    }
  }

  private var singlePlayTempoEyebrow: String {
    tempoTarget.map { "Tempo reached · target \(TempoUnit.readout(Int($0), unit: tempoUnit))" }
      ?? "Tempo reached"
  }

  private func mark(for playId: String) -> Int { marks[playId] ?? 0 }

  private func setMark(_ next: UInt8?, for playId: String) {
    marks[playId] = next.map(Int.init) ?? 0
  }

  private func stepperUnit(for play: ReflectionPlay) -> UInt8 {
    play.clickPattern?.metre.unit ?? tempoUnit
  }

  // TempoStepper only writes on an explicit tap or accessibility adjustment,
  // never on appear, so a write here is the user considering the number (#1420).
  private func tempoBinding(for playId: String) -> Binding<Int> {
    Binding(get: { tempos[playId]?.bpm ?? 0 }, set: { tempos[playId]?.set($0) })
  }

  private func eyebrow(_ text: String) -> some View {
    Eyebrow(text).frame(maxWidth: .infinity, alignment: .leading)
  }
}

#if DEBUG
  #Preview("Reflection · one play") {
    Color.black.opacity(0.2).ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        ReflectionSheet(
          itemTitle: "Clair de Lune", elapsedDisplay: "7:00", tempoTarget: 66,
          plays: [ReflectionPlay.preview("p1", nil, "7:00", nil, nil)],
          onSave: { _ in }, onSkip: {}
        )
        .presentationDetents([.medium, .large])
      }
  }

  #Preview("Reflection · three variations") {
    Color.black.opacity(0.2).ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        ReflectionSheet(
          itemTitle: "Major Scales", elapsedDisplay: "12:40", tempoTarget: nil,
          plays: [
            ReflectionPlay.preview("p1", "C major", "4:10", 8, 10),
            ReflectionPlay.preview("p2", "G major", "3:20", 10, 10),
            ReflectionPlay.preview("p3", "D major", "5:10", 4, 10),
          ],
          onSave: { _ in }, onSkip: {}
        )
        .presentationDetents([.large])
      }
  }
#endif
