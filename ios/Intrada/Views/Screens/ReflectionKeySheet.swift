import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// One key's tempo and note on the item-complete sheet, opened from its name
/// (#3004). Tempo and note apply on Done; a change of key or variation goes to
/// the core at once, as its choices read back from the core's row (#2249).
struct ReflectionKeySheet: View {
  let title: String
  let tempoRow: ReflectionTempoView?
  let limits: LimitsView
  let wayRow: FinishRowView?
  let wayChoices: PlayWayChoices
  let onWay: (DraftWay) -> Void
  let onDone: (TrackedTempo?, String) -> Void

  @State private var tempo: TrackedTempo?
  @State private var note: String

  init(
    title: String, tempoRow: ReflectionTempoView?, limits: LimitsView,
    tempo: TrackedTempo?, note: String,
    wayRow: FinishRowView?, wayChoices: PlayWayChoices,
    onWay: @escaping (DraftWay) -> Void,
    onDone: @escaping (TrackedTempo?, String) -> Void
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
  }

  var body: some View {
    BottomSheet(
      title: title, onDone: { onDone(tempo, note) },
      content: {
        ScrollView {
          content.padding(IntradaSpacing.card)
        }
      })
  }

  var content: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.section) {
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
      }
    }
  }

  private var tempoBinding: Binding<Int> {
    Binding(get: { tempo?.bpm ?? 0 }, set: { tempo?.set($0) })
  }
}
