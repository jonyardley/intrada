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
  static func rows(_ plays: [PlayView]) -> [ReflectionPlay] {
    plays.map { play in
      ReflectionPlay(
        id: play.id, variationLabel: play.label,
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
  var felt: Felt?
  var obstacles: [Obstacle] = []
  /// Points the musician kept, as spans into the note as drafted.
  var notePoints: [NoteSpan] = []
  var intentionMet: IntentionMet?
}

struct ReflectionSheet: View {
  @Environment(\.scenePhase) private var scenePhase

  let itemTitle: String
  let elapsedDisplay: String?
  /// The item's own declared tempo marking (the practice target), if any.
  let tempoTarget: UInt16?
  /// The bar an unstamped row counts in, reads `♪` for and sends as its `click`, so the
  /// range shown and the scale saved cannot disagree (#1499, #1761, #2304).
  let currentClick: ClickState?
  private var tempoUnit: UInt8 { currentClick?.metre.unit ?? 4 }
  /// What was played, in order. One row is the sheet that shipped before
  /// plays existed; several give each variation its own mark (#1739
  /// decision 10).
  let plays: [ReflectionPlay]
  let limits: LimitsView
  /// The core's finish answers to offer, and the aim they are asked against.
  let finish: FinishSheetView?
  let aim: String?
  /// Shown here because the player's banner sits under the sheet (#2009).
  let refusal: String?
  let onSave: (ReflectionResult) -> Void
  let onSkip: () -> Void
  /// The answers so far, for the crash-recovery copy (#2137): a mark or tempo at
  /// once, the note after a pause in typing or on leaving the app.
  let onDraft: (ReflectionResult) -> Void

  @State private var marks: [String: Int]
  @State private var note: String
  /// The trimmed note last handed to `onDraft`, so a pause with no new text writes nothing.
  @State private var draftedNote: String
  /// One per play, seeded from its stamp or the current click (#1761 rule 6).
  @State private var tempos: [String: TrackedTempo]
  @State private var felt: Felt?
  @State private var obstacles: [Obstacle]
  @State private var notePoints: [NoteSpan]
  @State private var intentionMet: IntentionMet?
  @State private var detailOpen: Bool

  init(
    itemTitle: String, elapsedDisplay: String?, tempoTarget: UInt16?,
    startingTempoBpm: Int? = nil,
    currentClick: ClickState? = nil,
    plays: [ReflectionPlay],
    limits: LimitsView,
    finish: FinishSheetView? = nil,
    aim: String? = nil,
    refusal: String? = nil,
    seed: ReflectionResult? = nil,
    onSave: @escaping (ReflectionResult) -> Void,
    onSkip: @escaping () -> Void,
    onDraft: @escaping (ReflectionResult) -> Void = { _ in }
  ) {
    self.itemTitle = itemTitle
    self.elapsedDisplay = elapsedDisplay
    self.tempoTarget = tempoTarget
    self.currentClick = currentClick
    self.plays = plays
    self.limits = limits
    self.finish = finish
    self.aim = aim
    self.refusal = refusal
    self.onSave = onSave
    self.onSkip = onSkip
    self.onDraft = onDraft
    _marks = State(initialValue: (seed?.marks ?? [:]).mapValues(Int.init))
    _note = State(initialValue: seed?.note ?? "")
    _draftedNote = State(initialValue: seed?.note ?? "")
    var tempos = Dictionary(
      plays.filter(\.isMarkable).map { play in
        (
          play.id,
          TrackedTempo(
            startingBpm: play.tempoDisplay.map(Int.init) ?? startingTempoBpm
              ?? Int(limits.clickTempoDefault),
            band: limits.clickBand(
              unit: play.clickPattern?.metre.unit ?? currentClick?.metre.unit ?? 4))
        )
      }, uniquingKeysWith: { first, _ in first })
    for row in seed?.tempos ?? [] where row.userSet {
      tempos[row.playId]?.set(Int(row.tempo))
    }
    _tempos = State(initialValue: tempos)
    _felt = State(initialValue: seed?.felt)
    _obstacles = State(initialValue: seed?.obstacles ?? [])
    _notePoints = State(initialValue: seed?.notePoints ?? [])
    _intentionMet = State(initialValue: seed?.intentionMet)
    _detailOpen = State(initialValue: seed?.felt != nil || !(seed?.obstacles.isEmpty ?? true))
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

          SectionTitle(singlePlayTempoHeading)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, IntradaSpacing.card)
          TempoStepper(
            value: tempoBinding(for: only.id), unit: stepperUnit(for: only),
            step: limits.clickStep, band: limits.clickBand(unit: stepperUnit(for: only))
          )
          .accessibilityIdentifier("reflection.tempo")
          .padding(.top, IntradaSpacing.controlGap)
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
            NoteOffers(
              offers: finish.noteOffers, isKept: { notePoints.contains($0) },
              onTap: toggleNotePoint
            )
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
          onSave(result)
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

  private var result: ReflectionResult {
    ReflectionResult(
      marks: marks.compactMapValues { $0 == 0 ? nil : UInt8($0) },
      note: note.trimmingCharacters(in: .whitespacesAndNewlines),
      tempos: plays.compactMap { play in
        tempos[play.id].map { tracked in
          ReflectionRowTempo(
            playId: play.id, tempo: UInt16(tracked.bpm), userSet: tracked.userSet,
            click: play.clickPattern ?? currentClick)
        }
      }, felt: felt, obstacles: obstacles, notePoints: keptPoints, intentionMet: intentionMet)
  }

  /// Only points the drafted note still offers: an edit can drop one the musician kept.
  private var keptPoints: [NoteSpan] {
    let offered = Set((finish?.noteOffers ?? []).map(\.span))
    return notePoints.filter(offered.contains)
  }

  private func toggleNotePoint(_ span: NoteSpan) {
    if let at = notePoints.firstIndex(of: span) {
      notePoints.remove(at: at)
    } else {
      notePoints.append(span)
    }
    draft()
  }

  /// A binding that saves the draft as soon as the answer changes (#2137).
  private func drafting<Value>(_ binding: Binding<Value>) -> Binding<Value> {
    Binding(
      get: { binding.wrappedValue },
      set: {
        binding.wrappedValue = $0
        draft()
      })
  }

  private func draft() {
    let current = result
    draftedNote = current.note
    onDraft(current)
  }

  private func draftNoteIfChanged() {
    if note.trimmingCharacters(in: .whitespacesAndNewlines) != draftedNote { draft() }
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
              .font(IntradaFont.secondary)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
          if play.isMarkable {
            ScoreSelector(
              score: mark(for: play.id), range: limits.scoreRange,
              accessibilityLabel: "Mark for \(play.title)"
            ) { next in
              setMark(next, for: play.id)
            }
            .accessibilityIdentifier("reflection.mark")
            TempoStepper(
              value: tempoBinding(for: play.id), unit: stepperUnit(for: play),
              step: limits.clickStep, band: limits.clickBand(unit: stepperUnit(for: play)),
              accessibilityLabel: "Tempo for \(play.title)"
            )
            .accessibilityIdentifier("reflection.tempo")
          }
        }
        .padding(.vertical, IntradaSpacing.cardCompact)
      }
    }
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

  private func stepperUnit(for play: ReflectionPlay) -> UInt8 {
    play.clickPattern?.metre.unit ?? tempoUnit
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
