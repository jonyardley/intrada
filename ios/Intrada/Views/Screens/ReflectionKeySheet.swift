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
  @State private var contentHeight: CGFloat = 0

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
      title: title, detents: [fitted, .large], dismissesOnDone: false, onDone: done,
      leadingAction: { EmptyView() },
      content: {
        ScrollView {
          content
            .padding(.horizontal, IntradaSpacing.section)
            .padding(.vertical, IntradaSpacing.card)
            .onGeometryChange(for: CGFloat.self) {
              $0.size.height
            } action: {
              contentHeight = $0
            }
        }
      })
  }

  /// Opens as tall as its few fields, then drags up for the keyboard or a long note.
  private var fitted: PresentationDetent {
    .height(contentHeight > 0 ? contentHeight + Self.barHeight : Self.openingHeight)
  }

  /// The inline title bar and grabber above the content.
  private static let barHeight: CGFloat = 64
  private static let openingHeight: CGFloat = 320

  var content: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.section) {
      if let refusal {
        FormErrorBanner(message: "Couldn't keep the note for \(title). \(refusal)")
      }
      if let wayRow {
        PlayWayEditor(row: wayRow, choices: wayChoices, onChange: onWay)
      }
      if let tempoRow, tempo != nil {
        StepperRow {
          StepperRowLabel("Tempo reached")
        } stepper: {
          TempoDrag(
            value: tempoBinding, unit: tempoRow.click.metre.unit,
            step: limits.clickStep, band: tempoRow.band.range,
            accessibilityLabel: "Tempo for \(title)")
        }
      }
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        SectionTitle("Note · optional")
        NoteBox {
          TextField("Add a note", text: $note, axis: .vertical)
            .lineLimit(2...4)
            .font(IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
            .accessibilityLabel("Note for \(title)")
            .accessibilityIdentifier("reflection.keyNote")
            .onChange(of: note) { _, typed in
              let kept = Self.capped(typed, to: Int(limits.playNoteMaxChars))
              if kept != typed { note = kept }
            }
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

/// A label or key name leading and a stepper trailing on one right edge, the
/// row grammar both reflection sheets share (#3004).
struct StepperRow<Leading: View, Stepper: View>: View {
  @ViewBuilder let leading: Leading
  @ViewBuilder let stepper: Stepper

  var body: some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      leading.frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
      stepper
    }
  }
}

/// What a stepper row sets, with an optional line under it, the same on both
/// reflection sheets (#3004).
struct StepperRowLabel: View {
  let title: String
  var detail: String?

  init(_ title: String, detail: String? = nil) {
    self.title = title
    self.detail = detail
  }

  var body: some View {
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
}

/// The white box a reflection note is typed into, under its section title.
struct NoteBox<Content: View>: View {
  @ViewBuilder let content: Content

  var body: some View {
    content
      .padding(IntradaSpacing.cardCompact)
      .frame(maxWidth: .infinity, alignment: .leading)
      .cardSurface(cornerRadius: IntradaRadius.card)
  }
}
