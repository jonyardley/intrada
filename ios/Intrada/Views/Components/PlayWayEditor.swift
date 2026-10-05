import SharedTypes
import SwiftUI

struct PlayWayChoices {
  let sections: [SectionView]
  let keys: [KeyChoiceView]
  let variations: [PickerVariationView]
  let planned: String?

  /// Section, key, variations, as the core joins a play's label; the written key goes unnamed.
  func label(_ way: DraftWay) -> String? {
    if way.sectionId == nil, way.key == nil, way.variationIds.isEmpty { return "Whole piece" }
    let section = way.sectionId.flatMap { id in sections.first { $0.id == id }?.label }
    let key = way.key.flatMap { key in keys.first { $0.key == key }?.label }
    let variations = way.variationIds.compactMap { id in
      self.variations.first { $0.id == id }?.label
    }
    let parts = [section, key].compactMap { $0 } + variations
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  /// The row's drafted ways after one change: a way back to what the play
  /// recorded is no change, so it leaves the draft.
  static func drafting(
    _ way: DraftWay, recorded: DraftWay, into ways: [DraftWay]
  ) -> [DraftWay] {
    let others = ways.filter { $0.playId != way.playId }
    let same =
      way.sectionId == recorded.sectionId && way.key == recorded.key
      && way.variationIds.sorted() == recorded.variationIds.sorted()
    return same ? others : others + [way]
  }
}

struct PlayWayEditor: View {
  let way: DraftWay
  let choices: PlayWayChoices
  let onChange: (DraftWay) -> Void

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      if !choices.sections.isEmpty {
        group("Section") {
          ChoiceGrid(
            options: [nil] + choices.sections.map { Optional($0.id) },
            isOn: { $0 == way.sectionId }, label: sectionLabel,
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
            options: choices.keys.map(\.key), isOn: { $0 == way.key }, label: keyLabel,
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
            options: choices.variations.map(\.id), isOn: { way.variationIds.contains($0) },
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
      grid()
    }
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
