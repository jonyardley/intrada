import SharedTypes
import SwiftUI

/// The player's variation picker (#1739 decision 6). Tapping a row switches the
/// open play; switching to the one already playing is the core's no-op, so a
/// stray tap cannot clear the repetition dots.
struct VariationPickerSheet: View {
  let itemTitle: String
  let currentVariations: [PickerVariationView]
  let currentVariationId: String?
  /// Returns false when the core refused the switch, which keeps the sheet up
  /// rather than closing over a chip that still names the old variation.
  let onPick: (String) -> Bool

  @Environment(\.dismiss) private var dismiss

  var body: some View {
    BottomSheet(title: "Switch variation", detents: [.medium, .large]) {
      ScrollView {
        VStack(alignment: .leading, spacing: 0) {
          ForEach(Array(currentVariations.enumerated()), id: \.element.id) { index, variation in
            if index > 0 {
              HairlineDivider().padding(.horizontal, IntradaSpacing.card)
            }
            row(variation)
          }
          Text("Switching starts a fresh count of repetitions.")
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .padding(IntradaSpacing.card)
        }
        .padding(.top, IntradaSpacing.controlGap)
      }
    }
  }

  private func row(_ variation: PickerVariationView) -> some View {
    PickerRow(
      label: variation.label, caption: variation.caption,
      isCurrent: variation.id == currentVariationId,
      hint: "Switches \(itemTitle) to this variation", identifier: "variationPicker.row"
    ) {
      if onPick(variation.id) { dismiss() }
    }
  }
}

#if DEBUG
  #Preview("Variation picker") {
    IntradaColor.sheetScrim.ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        VariationPickerSheet(
          itemTitle: "Major Scales",
          currentVariations: ActiveSessionView.previewActiveVariations.currentVariations,
          currentVariationId: ActiveSessionView.previewActiveVariations.currentVariationIds.first,
          onPick: { _ in true })
      }
  }
#endif
