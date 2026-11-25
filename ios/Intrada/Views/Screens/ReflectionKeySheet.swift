import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// One key's tempo and note, opened from its name; Done closes it only once the
/// core keeps them (#3004). A change of key or variation goes at once (#2249).
struct ReflectionKeySheet: View {
  let title: String
  let tempoRow: ReflectionTempoView?
  let limits: LimitsView
  let wayRow: FinishRowView?
  let wayChoices: PlayWayChoices
  let onWay: (DraftWay) -> Void
  /// The core's refusal, or `nil` once it has kept the tempo and note.
  let onDone: (TrackedTempo?, String) -> String?

  @Environment(\.dismiss) private var dismiss
  @State private var tempo: TrackedTempo?
  @State private var note: String
  @State private var refusal: String?

  init(
    title: String, tempoRow: ReflectionTempoView?, limits: LimitsView,
    tempo: TrackedTempo?, note: String,
    wayRow: FinishRowView?, wayChoices: PlayWayChoices,
    refusal: String? = nil,
    onWay: @escaping (DraftWay) -> Void,
    onDone: @escaping (TrackedTempo?, String) -> String?
  ) {
    self.title = title
    self.tempoRow = tempoRow
    self.limits = limits
    self.wayRow = wayRow
    self.wayChoices = wayChoices
    self.onWay = onWay
    self.onDone = onDone
    _tempo = State(initialValue: tempo)
    _note = State(initialValue: note)
    _refusal = State(initialValue: refusal)
  }

  var body: some View {
    BottomSheet(
      title: title, dismissesOnDone: false, onDone: done,
      leadingAction: { EmptyView() },
      content: {
        ScrollView {
          content.padding(IntradaSpacing.card)
        }
      })
  }

  var content: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.section) {
      if let refusal {
        FormErrorBanner(message: "Couldn't keep the note for \(title). \(refusal)")
      }
      if let wayRow {
        PlayWayEditor(row: wayRow, choices: wayChoices, onChange: onWay)
        HairlineDivider()
      }
      if let tempoRow, tempo != nil {
        HStack(spacing: IntradaSpacing.cardCompact) {
          FieldLabel("Tempo")
          Spacer(minLength: IntradaSpacing.controlGap)
          TempoStepper(
            value: tempoBinding, unit: tempoRow.click.metre.unit,
            step: limits.clickStep, band: tempoRow.band.range,
            accessibilityLabel: "Tempo for \(title)"
          )
          .accessibilityIdentifier("reflection.tempo")
        }
        HairlineDivider()
      }
      FieldCard("Note for \(title)") {
        TextField("Add a note", text: $note, axis: .vertical)
          .lineLimit(2...4)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.ink)
          .accessibilityIdentifier("reflection.keyNote")
          .onChange(of: note) { _, typed in
            let kept = Self.capped(typed, to: Int(limits.playNoteMaxChars))
            if kept != typed { note = kept }
          }
      }
    }
  }

  private func done() {
    refusal = onDone(tempo, note)
    if refusal == nil { dismiss() }
  }

  /// Whole characters off the end until it fits, counted in Unicode scalars as
  /// the core counts, so an emoji or accent cannot slip past the cap (#3004).
  nonisolated static func capped(_ text: String, to max: Int) -> String {
    guard text.unicodeScalars.count > max else { return text }
    var scalars = 0
    var end = text.startIndex
    for index in text.indices {
      let next = scalars + text[index].unicodeScalars.count
      if next > max { break }
      scalars = next
      end = text.index(after: index)
    }
    return String(text[..<end])
  }

  private var tempoBinding: Binding<Int> {
    Binding(get: { tempo?.bpm ?? 0 }, set: { tempo?.set($0) })
  }
}
