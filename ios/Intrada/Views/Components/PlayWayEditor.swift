import SharedTypes
import SwiftUI

struct PlayWayChoices {
  let sections: [SectionView]
  let keys: [KeyChoiceView]
  let variations: [PickerVariationView]
  let planned: String?
}

struct PlayWayEditor: View {
  let row: FinishRowView
  let choices: PlayWayChoices
  let onChange: (DraftWay) -> Void

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      if !choices.sections.isEmpty {
        group("Section") {
          ChoiceGrid(
            options: [nil] + choices.sections.map { Optional($0.id) },
            isOn: { $0 == row.sectionId }, label: sectionLabel,
            identifier: "reflection.waySection"
          ) { id in
            var next = way
            next.sectionId = id
            onChange(next)
          }
        }
      }
      if !choices.keys.isEmpty {
        group("Key") {
          ChoiceGrid(
            options: choices.keys.map(\.key), isOn: { $0 == row.key }, label: keyLabel,
            identifier: "reflection.wayKey"
          ) { key in
            var next = way
            next.key = key
            onChange(next)
          }
        }
      }
      if !choices.variations.isEmpty {
        group("Variations") {
          ChoiceGrid(
            options: choices.variations.map(\.id), isOn: { row.variationIds.contains($0) },
            label: variationLabel, identifier: "reflection.wayVariation"
          ) { id in
            var next = way
            if let at = next.variationIds.firstIndex(of: id) {
              next.variationIds.remove(at: at)
            } else {
              next.variationIds.append(id)
            }
            onChange(next)
          }
        }
      }
      if let planned = choices.planned {
        Text("Planned: \(planned)")
          .font(IntradaFont.small)
          .foregroundStyle(IntradaColor.inkSecondary)
          .fixedSize(horizontal: false, vertical: true)
      }
    }
    .padding(IntradaSpacing.cardCompact)
    .frame(maxWidth: .infinity, alignment: .leading)
    .cardSurface(cornerRadius: IntradaRadius.control)
  }

  private func group(_ title: String, @ViewBuilder _ grid: () -> some View) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      FieldLabel(title)
        .accessibilityAddTraits(.isHeader)
      grid()
    }
  }

  private var way: DraftWay {
    DraftWay(
      playId: row.playId, sectionId: row.sectionId, key: row.key, variationIds: row.variationIds)
  }

  private func sectionLabel(_ id: String?) -> String {
    id.flatMap { id in choices.sections.first { $0.id == id }?.label } ?? "Whole piece"
  }

  private func keyLabel(_ key: Key?) -> String {
    choices.keys.first { $0.key == key }?.label ?? ""
  }

  private func variationLabel(_ id: String) -> String {
    choices.variations.first { $0.id == id }?.label ?? ""
  }
}
