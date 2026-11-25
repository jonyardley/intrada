import IntradaCoreFFI
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
  /// Whether the core predicts this play survives the terminal drop (#1758).
  let isMarkable: Bool

  var title: String { variationLabel ?? "No variation" }

  /// Reads each play's own stamped seconds: `PrepareReflection` gives the still-open play its real duration first.
  static func rows(_ plays: [PlayView]) -> [ReflectionPlay] {
    plays.map { play in
      ReflectionPlay(
        id: play.id, variationLabel: play.label,
        durationDisplay: SessionClock.clockDisplay(Int(play.seconds)),
        isMarkable: play.isMarkable)
    }
  }
}

struct ReflectionSheet: View {
  @Environment(\.scenePhase) private var scenePhase

  let itemTitle: String
  let elapsedDisplay: String?
  /// The item's declared tempo in the core's words, in its metre's beat (#1510).
  let tempoTarget: String?
  /// The core's row per markable play: its opening number, bar and band (#2230).
  let tempoRows: [ReflectionTempoView]
  /// What was played, in order. One row is the sheet that shipped before
  /// plays existed; several give each variation its own mark (#1739
  /// decision 10).
  let plays: [ReflectionPlay]
  let limits: LimitsView
  /// The core's rows and the keys each can change to (#2249).
  let finish: FinishSheetView?
  /// The item's live variations, for changing what a row says was played (#2249).
  let variations: [PickerVariationView]
  let plannedLabel: String?
  /// Shown here because the player's banner sits under the sheet (#2009).
  let refusal: String?
  let onSave: (ReflectionAnswers) -> Void
  let onSkip: () -> Void
  /// The answers so far, for the crash-recovery copy (#2137): a mark or tempo at
  /// once, the note after a pause in typing or on leaving the app.
  let onDraft: (ReflectionAnswers) -> Void
  /// A key sheet's Done: the draft with that key's tempo and note, answered
  /// with the core's refusal or `nil` once it keeps them (#3004).
  let onKeyDraft: (ReflectionAnswers) -> String?

  @State private var marks: [String: Int]
  @State private var note: String
  /// The note last handed to `onDraft`, so a pause with no new text writes nothing.
  @State private var draftedNote: String
  /// One per row the core gave, so a play with no row has no tempo to send.
  @State private var tempos: [String: TrackedTempo]
  @State private var ways: [DraftWay]
  /// Each key's own note, by play (#3004).
  @State private var notes: [String: String]
  @State private var editingPlayId: String?

  init(
    itemTitle: String, elapsedDisplay: String?, tempoTarget: String?,
    tempoRows: [ReflectionTempoView] = [],
    plays: [ReflectionPlay],
    limits: LimitsView,
    finish: FinishSheetView? = nil,
    variations: [PickerVariationView] = [],
    plannedLabel: String? = nil,
    refusal: String? = nil,
    seed: ReflectionAnswers? = nil,
    onSave: @escaping (ReflectionAnswers) -> Void,
    onSkip: @escaping () -> Void,
    onDraft: @escaping (ReflectionAnswers) -> Void = { _ in },
    onKeyDraft: @escaping (ReflectionAnswers) -> String? = { _ in nil }
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
    self.refusal = refusal
    self.onSave = onSave
    self.onSkip = onSkip
    self.onDraft = onDraft
    self.onKeyDraft = onKeyDraft
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
    _ways = State(initialValue: seed?.ways ?? [])
    _notes = State(
      initialValue: Dictionary(
        (seed?.notes ?? []).map { ($0.playId, $0.note) },
        uniquingKeysWith: { first, _ in first }))
  }

  static func heading(elapsedDisplay: String?) -> String {
    elapsedDisplay.map { "Item complete · \($0)" } ?? "Item complete"
  }

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 0) {
        VStack(spacing: IntradaSpacing.controlGap) {
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

        // The centred title block reads as the sheet's bar, as on every BottomSheet.
        HairlineDivider().padding(.top, IntradaSpacing.section)

        if plays.count > 1 {
          sectionTitle("Confidence").padding(.top, IntradaSpacing.section)
          keyRows.padding(.top, IntradaSpacing.controlGap)
        } else if let only = plays.first, let row = tempoRow(for: only.id) {
          // The core gives every practised entry at least one play, and its sole play always predicts markable (#1758).
          singlePlayRows(only, row).padding(.top, IntradaSpacing.section)
        }

        sectionTitle("Reflection · optional").padding(.top, IntradaSpacing.section)
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

        Button("Skip") { onSkip() }
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
    .sheet(item: editingPlay) { play in
      keySheet(play)
    }
  }

  private var answers: ReflectionAnswers { answers(notes: notes) }

  private func answers(notes: [String: String]) -> ReflectionAnswers {
    ReflectionAnswers(
      marks: plays.compactMap { play in
        marks[play.id].map { DraftMark(playId: play.id, score: UInt8(clamping: $0)) }
      },
      note: note,
      tempos: TrackedTempo.handSet(tempoRows, tracked: tempos), ways: ways,
      notes: plays.compactMap { play in
        notes[play.id].flatMap { $0.isEmpty ? nil : DraftNote(playId: play.id, note: $0) }
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

  private var keyRows: some View {
    VStack(alignment: .leading, spacing: 0) {
      ForEach(Array(plays.enumerated()), id: \.element.id) { index, play in
        if index > 0 { HairlineDivider() }
        keyRow(play)
          .padding(.top, index > 0 ? IntradaSpacing.controlGap : 0)
          .padding(.bottom, index < plays.count - 1 ? IntradaSpacing.controlGap : 0)
      }
    }
  }

  private func singlePlayRows(_ play: ReflectionPlay, _ row: ReflectionTempoView) -> some View {
    VStack(alignment: .leading, spacing: 0) {
      StepperRow {
        rowLabel("Confidence", detail: nil)
      } stepper: {
        scoreStepper(play, row, label: "Score for \(singlePlayTitle(play))")
      }
      .padding(.bottom, IntradaSpacing.controlGap)
      HairlineDivider()
      StepperRow {
        rowLabel("Tempo reached", detail: tempoTarget.map { "target \($0)" })
      } stepper: {
        TempoStepper(
          value: tempoBinding(for: play.id), unit: row.click.metre.unit,
          step: limits.clickStep, band: row.band.range
        )
        .accessibilityIdentifier("reflection.tempo")
      }
      .padding(.top, IntradaSpacing.controlGap)
    }
  }

  private func rowLabel(_ title: String, detail: String?) -> some View {
    VStack(alignment: .leading, spacing: 0) {
      Text(title)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.ink)
      if let detail {
        Text(detail)
          .font(IntradaFont.small)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
    .accessibilityElement(children: .combine)
  }

  @ViewBuilder private func keyRow(_ play: ReflectionPlay) -> some View {
    if play.isMarkable {
      StepperRow {
        Button {
          editingPlayId = play.id
        } label: {
          keyDetails(play, chevron: true)
            .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(spokenRow(play))
        .accessibilityHint("Edit tempo and note")
        .accessibilityIdentifier("reflection.key")
      } stepper: {
        if let row = tempoRow(for: play.id) {
          scoreStepper(play, row, label: "Score for \(rowTitle(play))")
        }
      }
    } else {
      keyDetails(play, chevron: false)
        .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
        .accessibilityElement(children: .combine)
    }
  }

  private func keyDetails(_ play: ReflectionPlay, chevron: Bool) -> some View {
    VStack(alignment: .leading, spacing: 0) {
      HStack(spacing: IntradaSpacing.segmentGap) {
        Text(rowTitle(play))
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.ink)
          .fixedSize(horizontal: false, vertical: true)
        if chevron {
          Image(systemName: "chevron.right")
            .iconSize(.caption, weight: .semibold)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      Text(
        [play.durationDisplay, tempoWords(play)?.text].compactMap { $0 }.joined(separator: " · ")
      )
      .font(IntradaFont.small)
      .foregroundStyle(IntradaColor.inkSecondary)
      if let note = notes[play.id], !note.isEmpty {
        Text(note)
          .font(IntradaFont.small)
          .foregroundStyle(IntradaColor.inkSecondary)
          .lineLimit(1)
      }
    }
  }

  private func tempoWords(_ play: ReflectionPlay) -> TempoWords? {
    guard let row = tempoRow(for: play.id), let tempo = tempos[play.id] else { return nil }
    return clickTempoWords(bpm: UInt16(clamping: tempo.bpm), unit: row.click.metre.unit)
  }

  private func spokenRow(_ play: ReflectionPlay) -> String {
    var parts = [rowTitle(play), play.durationDisplay, tempoWords(play)?.spoken].compactMap { $0 }
    if let note = notes[play.id], !note.isEmpty { parts.append("note: \(note)") }
    return parts.joined(separator: ", ")
  }

  private func origin(_ play: ReflectionPlay, _ row: ReflectionTempoView) -> ScoreStepper.Origin {
    if marks[play.id] != nil { return .today }
    return row.lastScore == nil ? .suggested : .lastTime
  }

  private var editingPlay: Binding<ReflectionPlay?> {
    Binding(
      get: { plays.first { $0.id == editingPlayId && $0.isMarkable } },
      set: { editingPlayId = $0?.id })
  }

  private func keySheet(_ play: ReflectionPlay) -> some View {
    let wayRow = finishRow(play).flatMap { $0.canChange ? $0 : nil }
    return ReflectionKeySheet(
      title: rowTitle(play), tempoRow: tempoRow(for: play.id), limits: limits,
      tempo: tempos[play.id], note: notes[play.id] ?? "",
      wayRow: wayRow, wayChoices: wayChoices,
      onWay: { next in
        ways = ways.filter { $0.playId != next.playId } + [next]
        draft()
      },
      onDone: { tempo, note in
        if let tempo, tempo.userSet { tempos[play.id] = tempo }
        var proposed = notes
        proposed[play.id] = note
        let candidate = answers(notes: proposed)
        if let refusal = onKeyDraft(candidate) { return refusal }
        notes = proposed
        draftedNote = candidate.note
        return nil
      })
  }

  private var wayChoices: PlayWayChoices {
    PlayWayChoices(keys: finish?.keys ?? [], variations: variations, planned: plannedLabel)
  }

  private func finishRow(_ play: ReflectionPlay) -> FinishRowView? {
    finish?.rows.first { $0.playId == play.id }
  }

  private func rowTitle(_ play: ReflectionPlay) -> String {
    finishRow(play)?.label ?? play.title
  }

  /// The item's declared tempo in its metre's beat, the click's seed bar
  /// being the item's metre or 4/4 (#1510).
  nonisolated static func tempoTarget(for active: ActiveSessionView) -> String? {
    active.currentItemTempoBpm.map {
      clickTempoWords(bpm: $0, unit: active.clickSeedMetre.unit).text
    }
  }

  /// Untouched, it shows the score the core will save (#3004).
  private func scoreStepper(
    _ play: ReflectionPlay, _ row: ReflectionTempoView, label: String
  ) -> some View {
    ScoreStepper(
      score: marks[play.id] ?? Int(row.startScore), origin: origin(play, row),
      range: limits.scoreRange, accessibilityLabel: label
    ) { next in
      marks[play.id] = next
      draft()
    }
    .accessibilityIdentifier("reflection.mark")
  }

  private func singlePlayTitle(_ play: ReflectionPlay) -> String {
    finishRow(play).map { "\(itemTitle), \($0.label)" } ?? itemTitle
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
          itemTitle: "Clair de Lune", elapsedDisplay: "7:00",
          tempoTarget: clickTempoWords(bpm: 66, unit: 4).text,
          tempoRows: [.preview("p1")],
          plays: [ReflectionPlay.preview("p1", nil, "7:00")], limits: .preview,
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
            ReflectionPlay.preview("p1", "C major", "4:10"),
            ReflectionPlay.preview("p2", "G major", "3:20"),
            ReflectionPlay.preview("p3", "D major", "5:10"),
          ],
          limits: .preview,
          onSave: { _ in }, onSkip: {}
        )
        .presentationDetents([.large])
      }
  }
#endif
