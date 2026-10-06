import SharedTypes
import SwiftUI

/// A tempo and whether the user set it, as one value: `bpm` is not writable
/// except through `set`, so the sheet cannot report a number without also
/// reporting where it came from (#1420).
@MainActor
struct TrackedTempo {
  private(set) var bpm: Int
  private(set) var userSet = false
  /// The core's band in the unit `bpm` counts in, so a quaver tempo is not
  /// clamped against the crotchet band (#1499, #2225).
  private let band: ClosedRange<Int>

  init(startingBpm: Int, band: ClosedRange<Int>) {
    self.band = band
    bpm = min(band.upperBound, max(band.lowerBound, startingBpm))
  }

  mutating func set(_ next: Int) {
    bpm = min(band.upperBound, max(band.lowerBound, next))
    userSet = true
  }

  /// A tempo set by hand before a resume reopens as set by hand (#2137).
  init(row: ReflectionTempoView) {
    self.init(startingBpm: Int(row.tempo), band: row.band.range)
    if row.setByHand { set(Int(row.tempo)) }
  }

  /// Only the rows the musician moved, each with its own bar (#1420, #2304).
  static func handSet(_ rows: [ReflectionTempoView], tracked: [String: TrackedTempo])
    -> [DraftTempo]
  {
    rows.compactMap { row in
      guard let tempo = tracked[row.playId], tempo.userSet else { return nil }
      return DraftTempo(playId: row.playId, tempo: UInt16(tempo.bpm), click: row.click)
    }
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

  var title: String { variationLabel ?? "No variation" }

  var meta: String {
    var parts = [durationDisplay]
    if let repTarget { parts.append("\(repCount ?? 0) of \(repTarget)") }
    return parts.joined(separator: " · ")
  }

  /// Reads each play's own stamped seconds: `PrepareReflection` gives the still-open play its real duration first.
  static func rows(_ plays: [PlayView]) -> [ReflectionPlay] {
    plays.map { play in
      ReflectionPlay(
        id: play.id, variationLabel: play.label,
        durationDisplay: SessionClock.clockDisplay(Int(play.seconds)),
        repCount: play.repCount, repTarget: play.repTarget, isMarkable: play.isMarkable)
    }
  }
}

struct ReflectionSheet: View {
  @Environment(\.scenePhase) private var scenePhase

  let itemTitle: String
  let elapsedDisplay: String?
  /// The item's own declared tempo marking (the practice target), if any.
  let tempoTarget: UInt16?
  /// The core's row per markable play: its opening number, bar and band (#2230).
  let tempoRows: [ReflectionTempoView]
  /// What was played, in order. One row is the sheet that shipped before
  /// plays existed; several give each variation its own mark (#1739
  /// decision 10).
  let plays: [ReflectionPlay]
  let limits: LimitsView
  /// The core's finish answers to offer, and the aim they are asked against.
  let finish: FinishSheetView?
  /// The item's live variations, for changing what a row says was played (#2249).
  let variations: [PickerVariationView]
  let plannedLabel: String?
  let aim: String?
  /// Shown here because the player's banner sits under the sheet (#2009).
  let refusal: String?
  let onSave: (ReflectionAnswers) -> Void
  let onSkip: () -> Void
  /// The answers so far, for the crash-recovery copy (#2137): a mark or tempo at
  /// once, the note after a pause in typing or on leaving the app.
  let onDraft: (ReflectionAnswers) -> Void

  @State private var marks: [String: Int]
  @State private var note: String
  /// The trimmed note last handed to `onDraft`, so a pause with no new text writes nothing.
  @State private var draftedNote: String
  /// One per row the core gave, so a play with no row has no tempo to send.
  @State private var tempos: [String: TrackedTempo]
  @State private var felt: Felt?
  @State private var obstacles: [Obstacle]
  @State private var notePoints: [NoteSpan]
  @State private var intentionMet: IntentionMet?
  @State private var detailOpen: Bool
  @State private var ways: [DraftWay]
  @State private var changingPlayId: String?

  init(
    itemTitle: String, elapsedDisplay: String?, tempoTarget: UInt16?,
    tempoRows: [ReflectionTempoView] = [],
    plays: [ReflectionPlay],
    limits: LimitsView,
    finish: FinishSheetView? = nil,
    variations: [PickerVariationView] = [],
    plannedLabel: String? = nil,
    aim: String? = nil,
    refusal: String? = nil,
    seed: ReflectionAnswers? = nil,
    onSave: @escaping (ReflectionAnswers) -> Void,
    onSkip: @escaping () -> Void,
    onDraft: @escaping (ReflectionAnswers) -> Void = { _ in },
    changingPlayId: String? = nil
  ) {
    self.itemTitle = itemTitle
    self.elapsedDisplay = elapsedDisplay
    self.tempoTarget = tempoTarget
    self.tempoRows = tempoRows
    self.plays = plays
    self.limits = limits
    self.finish = finish
    self.variations = variations
    self.plannedLabel = plannedLabel
    self.aim = aim
    self.refusal = refusal
    self.onSave = onSave
    self.onSkip = onSkip
    self.onDraft = onDraft
    _marks = State(
      initialValue: Dictionary(
        (seed?.marks ?? []).map { ($0.playId, Int($0.score)) },
        uniquingKeysWith: { first, _ in first }))
    _note = State(initialValue: seed?.note ?? "")
    _draftedNote = State(initialValue: seed?.note ?? "")
    _tempos = State(
      initialValue: Dictionary(
        tempoRows.map { ($0.playId, TrackedTempo(row: $0)) },
        uniquingKeysWith: { first, _ in first }))
    _felt = State(initialValue: seed?.felt)
    _obstacles = State(initialValue: seed?.gotInTheWay ?? [])
    _notePoints = State(initialValue: seed?.notePoints ?? [])
    _intentionMet = State(initialValue: seed?.intentionMet)
    _detailOpen = State(initialValue: seed?.felt != nil || !(seed?.gotInTheWay.isEmpty ?? true))
    _ways = State(initialValue: seed?.ways ?? [])
    _changingPlayId = State(initialValue: changingPlayId)
  }

  static func heading(elapsedDisplay: String?) -> String {
    elapsedDisplay.map { "Item complete · \($0)" } ?? "Item complete"
  }

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 0) {
        VStack(spacing: 8) {
          SectionTitle(Self.heading(elapsedDisplay: elapsedDisplay))
          Text("How did it go?")
            .font(IntradaFont.title).foregroundStyle(IntradaColor.ink)
            .multilineTextAlignment(.center)
          Text(itemTitle)
            .font(IntradaFont.secondary).foregroundStyle(IntradaColor.inkSecondary)
            .lineLimit(1).truncationMode(.tail)
            .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
        .padding(.top, IntradaSpacing.card)

        if let finish, let aim, finish.asksIntention || finish.intentionMetRead != nil {
          AimAnswer(
            aim: aim, asks: finish.asksIntention, read: finish.intentionMetRead,
            answer: drafting($intentionMet)
          )
          .padding(.top, IntradaSpacing.section)
        }

        if plays.count > 1 {
          sectionTitle("What you played").padding(.top, IntradaSpacing.section)
          playRows.padding(.top, IntradaSpacing.controlGap)
        } else if let only = plays.first {
          // The core gives every practised entry at least one play, and its sole play always predicts markable (#1758).
          sectionTitle("Mark").padding(.top, IntradaSpacing.section)
          ScoreSelector(
            score: mark(for: only.id), range: limits.scoreRange,
            accessibilityLabel: "Mark for \(itemTitle)"
          ) { next in
            setMark(next, for: only.id)
          }
          .accessibilityIdentifier("reflection.mark")
          .padding(.top, IntradaSpacing.controlGap)

          if let row = tempoRow(for: only.id) {
            SectionTitle(singlePlayTempoHeading)
              .frame(maxWidth: .infinity, alignment: .leading)
              .padding(.top, IntradaSpacing.card)
            TempoStepper(
              value: tempoBinding(for: only.id), unit: row.click.metre.unit,
              step: limits.clickStep, band: row.band.range
            )
            .accessibilityIdentifier("reflection.tempo")
            .padding(.top, IntradaSpacing.controlGap)
          }
        }

        sectionTitle("Reflection · optional").padding(.top, IntradaSpacing.card)
        HStack(alignment: .top, spacing: IntradaSpacing.controlGap) {
          TextField("What went well? What to fix next time?", text: $note, axis: .vertical)
            .lineLimit(3...5)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
            .accessibilityIdentifier("reflection.note")
          SpeechInputButton(text: $note)
            .accessibilityIdentifier("reflection.speak")
        }
        .padding(IntradaSpacing.cardCompact)
        .cardSurface(cornerRadius: IntradaRadius.control)
        .padding(.top, IntradaSpacing.controlGap)

        if let finish {
          if !finish.noteOffers.isEmpty {
            NoteOffers(offers: finish.noteOffers, onTap: toggleNotePoint)
              .padding(.top, IntradaSpacing.card)
          }
          FinishDetail(
            feltChoices: finish.feltChoices, obstacleChoices: finish.obstacleChoices,
            felt: drafting($felt), obstacles: drafting($obstacles), open: $detailOpen
          )
          .padding(.top, IntradaSpacing.card)
        }

        if let refusal {
          FormErrorBanner(message: refusal)
            .padding(.top, IntradaSpacing.card)
            .transition(.move(edge: .top).combined(with: .opacity))
        }

        BrandBarButton {
          onSave(answers)
        } label: {
          Text("Save & continue")
          Image(systemName: "arrow.right")
        }
        .accessibilityIdentifier("reflection.save")
        .padding(.top, IntradaSpacing.card)

        Button("Skip rating") { onSkip() }
          .accessibilityIdentifier("reflection.skip")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.inkSecondary)
          .frame(maxWidth: .infinity)
          .padding(.top, IntradaSpacing.cardCompact)
      }
      .padding(.horizontal, IntradaSpacing.section)
      .padding(.bottom, IntradaSpacing.section)
    }
    .task(id: note) {
      do { try await Task.sleep(for: .milliseconds(600)) } catch { return }
      draftNoteIfChanged()
    }
    .onChange(of: scenePhase) { _, phase in
      if phase != .active { draftNoteIfChanged() }
    }
  }

  private var answers: ReflectionAnswers {
    ReflectionAnswers(
      marks: plays.compactMap { play in
        marks[play.id].map { DraftMark(playId: play.id, score: UInt8(clamping: $0)) }
      },
      note: note,
      tempos: TrackedTempo.handSet(tempoRows, tracked: tempos),
      felt: felt, gotInTheWay: obstacles, notePoints: notePoints,
      intentionMet: intentionMet, ways: ways)
  }

  private func toggleNotePoint(_ span: NoteSpan) {
    var kept = finish?.noteOffers.filter(\.confirmed).map(\.span) ?? []
    if let at = kept.firstIndex(of: span) {
      kept.remove(at: at)
    } else {
      kept.append(span)
    }
    notePoints = kept
    draft()
  }

  private func drafting<Value>(_ binding: Binding<Value>) -> Binding<Value> {
    Binding(
      get: { binding.wrappedValue },
      set: {
        binding.wrappedValue = $0
        draft()
      })
  }

  private func draft() {
    let current = answers
    draftedNote = current.note
    onDraft(current)
  }

  private func draftNoteIfChanged() {
    if note != draftedNote { draft() }
  }

  private var playRows: some View {
    VStack(alignment: .leading, spacing: 0) {
      ForEach(Array(plays.enumerated()), id: \.element.id) { index, play in
        if index > 0 { HairlineDivider() }
        VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
          playHeader(play)
          if changingPlayId == play.id {
            if let row = finishRow(play) {
              PlayWayEditor(row: row, choices: wayChoices) { next in
                ways = ways.filter { $0.playId != next.playId } + [next]
                draft()
              }
            }
          }
          if play.isMarkable {
            ScoreSelector(
              score: mark(for: play.id), range: limits.scoreRange,
              accessibilityLabel: "Mark for \(rowTitle(play))"
            ) { next in
              setMark(next, for: play.id)
            }
            .accessibilityIdentifier("reflection.mark")
            if let row = tempoRow(for: play.id) {
              TempoStepper(
                value: tempoBinding(for: play.id), unit: row.click.metre.unit,
                step: limits.clickStep, band: row.band.range,
                accessibilityLabel: "Tempo for \(rowTitle(play))"
              )
              .accessibilityIdentifier("reflection.tempo")
            }
          }
        }
        .padding(.vertical, IntradaSpacing.cardCompact)
      }
    }
  }

  @ViewBuilder private func playHeader(_ play: ReflectionPlay) -> some View {
    let title = rowTitle(play)
    if finishRow(play)?.canChange != true {
      HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.cardCompact) {
        Text(title)
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.ink)
          .frame(maxWidth: .infinity, alignment: .leading)
        Text(play.meta)
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    } else {
      let open = changingPlayId == play.id
      HStack(alignment: .firstTextBaseline, spacing: IntradaSpacing.cardCompact) {
        VStack(alignment: .leading, spacing: 2) {
          Text(title)
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.ink)
            .fixedSize(horizontal: false, vertical: true)
          Text(play.meta)
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        Button(open ? "Done" : "Change") {
          withAnimation { changingPlayId = open ? nil : play.id }
        }
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(minHeight: 44)
        .accessibilityLabel(
          open ? "Done changing \(title)" : "Change what you played on \(title)"
        )
        .accessibilityIdentifier("reflection.changeWay")
      }
    }
  }

  private var wayChoices: PlayWayChoices {
    PlayWayChoices(
      sections: finish?.sections ?? [], keys: finish?.keys ?? [], variations: variations,
      planned: plannedLabel)
  }

  private func finishRow(_ play: ReflectionPlay) -> FinishRowView? {
    finish?.rows.first { $0.playId == play.id }
  }

  private func rowTitle(_ play: ReflectionPlay) -> String {
    finishRow(play)?.label ?? play.title
  }

  private var singlePlayTempoHeading: String {
    tempoTarget.flatMap { TempoFormatting.display(marking: nil, bpm: $0) }
      .map { "Tempo reached · target \($0)" } ?? "Tempo reached"
  }

  private func mark(for playId: String) -> Int { marks[playId] ?? 0 }

  private func setMark(_ next: UInt8?, for playId: String) {
    marks[playId] = next.map(Int.init) ?? 0
    draft()
  }

  private func tempoRow(for playId: String) -> ReflectionTempoView? {
    tempoRows.first { $0.playId == playId }
  }

  // TempoStepper only writes on an explicit tap or accessibility adjustment,
  // never on appear, so a write here is the user considering the number (#1420).
  private func tempoBinding(for playId: String) -> Binding<Int> {
    Binding(
      get: { tempos[playId]?.bpm ?? 0 },
      set: {
        tempos[playId]?.set($0)
        draft()
      })
  }

  private func sectionTitle(_ text: String) -> some View {
    SectionTitle(text).frame(maxWidth: .infinity, alignment: .leading)
  }
}

#if DEBUG
  #Preview("Reflection · one play") {
    IntradaColor.sheetScrim.ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        ReflectionSheet(
          itemTitle: "Clair de Lune", elapsedDisplay: "7:00", tempoTarget: 66,
          tempoRows: [.preview("p1")],
          plays: [ReflectionPlay.preview("p1", nil, "7:00", nil, nil)], limits: .preview,
          onSave: { _ in }, onSkip: {}
        )
        .presentationDetents([.medium, .large])
      }
  }

  #Preview("Reflection · three variations") {
    IntradaColor.sheetScrim.ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        ReflectionSheet(
          itemTitle: "Major Scales", elapsedDisplay: "12:40", tempoTarget: nil,
          tempoRows: [.preview("p1"), .preview("p2"), .preview("p3")],
          plays: [
            ReflectionPlay.preview("p1", "C major", "4:10", 8, 10),
            ReflectionPlay.preview("p2", "G major", "3:20", 10, 10),
            ReflectionPlay.preview("p3", "D major", "5:10", 4, 10),
          ],
          limits: .preview,
          onSave: { _ in }, onSkip: {}
        )
        .presentationDetents([.large])
      }
  }
#endif
