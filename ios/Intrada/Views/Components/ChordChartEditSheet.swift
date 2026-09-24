import SharedTypes
import SwiftUI

/// Editor for a piece's chord chart. Sends the raw text to the core, which
/// parses it; on a parse error the core surfaces the offending token and this
/// sheet stays open showing it inline (never a silent dismiss, #846). The chart
/// derives in the piece's key, so the key isn't edited here.
struct ChordChartEditSheet: View {
  enum Destination {
    case piece(id: String)
    case caller((String) -> Void)
  }

  let destination: Destination
  let pieceKey: String?
  let pieceModality: Modality?

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var text: String
  @State private var parseError: String?

  init(pieceId: String, pieceKey: String?, pieceModality: Modality?, existingChart: ChordChart?) {
    destination = .piece(id: pieceId)
    self.pieceKey = pieceKey
    self.pieceModality = pieceModality
    _text = State(initialValue: existingChart.map(Self.reconstructText) ?? "")
  }

  init(
    text: String, pieceKey: String?, pieceModality: Modality?,
    onSave: @escaping (String) -> Void
  ) {
    destination = .caller(onSave)
    self.pieceKey = pieceKey
    self.pieceModality = pieceModality
    _text = State(initialValue: text)
  }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        ScrollView {
          VStack(alignment: .leading, spacing: IntradaSpacing.card) {
            keyCaption
            editor
            if let parseError {
              errorCard(parseError)
            }
            formatHint
          }
          .padding(IntradaSpacing.card)
        }
      }
      .navigationTitle("Chord chart")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Cancel") { dismiss() }
        }
        ToolbarItem(placement: .confirmationAction) {
          Button("Save") { save() }
            .disabled(saveDisabled)
            .accessibilityIdentifier("chordChart.save")
        }
      }
    }
    .presentationDetents([.large])
  }

  private var keyCaption: some View {
    HStack(spacing: 6) {
      Image(systemName: "key")
        .font(IntradaFont.meta)
        .accessibilityHidden(true)
      Text("Derives in \(keyDisplay)")
        .font(IntradaFont.meta)
    }
    .foregroundStyle(IntradaColor.inkSecondary)
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Derives in \(keyDisplay)")
  }

  private var keyDisplay: String {
    let modality = pieceModality ?? .major
    let key = pieceKey.flatMap { $0.isEmpty ? nil : $0 } ?? "C"
    return KeyHelper.display(key: key, modality: modality) ?? "C \(KeyHelper.modeWord(modality))"
  }

  private var editor: some View {
    ZStack(alignment: .topLeading) {
      if text.isEmpty {
        Text("| Cm7 | F7 | Bbmaj7 |")
          .font(IntradaFont.chartEditor)
          .foregroundStyle(IntradaColor.inkSecondary)
          .padding(.horizontal, 5)
          .padding(.vertical, 8)
          .allowsHitTesting(false)
      }
      TextEditor(text: $text)
        .font(IntradaFont.chartEditor)
        .foregroundStyle(IntradaColor.ink)
        .scrollContentBackground(.hidden)
        .frame(minHeight: 160)
        .autocorrectionDisabled()
        .textInputAutocapitalization(.characters)
        .accessibilityLabel("Chord chart text")
        .accessibilityIdentifier("chordChart.text")
    }
    .padding(IntradaSpacing.cardCompact)
    .background(IntradaColor.cardFill, in: RoundedRectangle(cornerRadius: IntradaRadius.control))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.control)
        .stroke(parseError == nil ? IntradaColor.divider : IntradaColor.danger, lineWidth: 1)
    )
  }

  private func errorCard(_ message: String) -> some View {
    HStack(alignment: .top, spacing: IntradaSpacing.controlGap) {
      Image(systemName: "exclamationmark.circle.fill")
        .foregroundStyle(IntradaColor.danger)
        .accessibilityHidden(true)
      Text(message)
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.ink)
        .fixedSize(horizontal: false, vertical: true)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(IntradaSpacing.cardCompact)
    .background(
      IntradaColor.dangerWash, in: RoundedRectangle(cornerRadius: IntradaRadius.control)
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Couldn't parse the chart. \(message)")
  }

  private var formatHint: some View {
    VStack(alignment: .leading, spacing: 6) {
      Eyebrow("Format")
      Text("[Section] labels · one bar between | pipes")
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.inkSecondary)
      Text("Cm7  F7  Bbmaj7  Aø7  D7alt")
        .font(IntradaFont.chart)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(IntradaSpacing.card)
    .cardSurface()
  }

  // A parse rejection or a bridge failure keeps the sheet open with the
  // message inline.
  private func save() {
    switch destination {
    case .piece(let id):
      if store.send(.item(.setChordChart(pieceId: id, rawChart: text)), onSuccess: .success) {
        dismiss()
      } else {
        parseError = store.viewModel?.error ?? "Couldn't save the chart. Try again."
      }
    case .caller(let onSave):
      onSave(text)
      dismiss()
    }
  }

  private var saveDisabled: Bool {
    switch destination {
    case .piece: text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    case .caller: false
    }
  }

  /// Rebuild an editable text chart from the stored structure (chord symbols'
  /// verbatim `raw`), so re-opening the editor shows what was entered.
  static func reconstructText(_ chart: ChordChart) -> String {
    var lines: [String] = []
    for section in chart.sections {
      if let label = section.label, !label.isEmpty {
        lines.append("[\(label)]")
      }
      let bars = section.bars.map { bar in
        bar.chords.map { $0.symbol.raw }.joined(separator: " ")
      }
      if !bars.isEmpty {
        lines.append("| " + bars.joined(separator: " | ") + " |")
      }
    }
    return lines.joined(separator: "\n")
  }
}

#if DEBUG
  #Preview("Empty") {
    Color.clear.sheet(isPresented: .constant(true)) {
      ChordChartEditSheet(
        pieceId: "p1", pieceKey: "G", pieceModality: .minor, existingChart: nil)
    }
    .environment(Store.preview)
  }
#endif
