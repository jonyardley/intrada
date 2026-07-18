import SharedTypes
import SwiftUI

/// Minimal step-ladder creation sheet: ordered labels in, `setVariants` out.
/// Only ever opened from an exercise with no ladder yet, so there are no
/// existing steps to rename/reorder/archive here — that's C4, once a ladder
/// already exists.
struct AddStepsSheet: View {
  let itemId: String

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var labels: [String] = ["", ""]

  var body: some View {
    BottomSheet(
      title: "Steps", confirmationLabel: "Save",
      confirmationDisabled: trimmedLabels.isEmpty,
      onDone: save
    ) {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
          Eyebrow("Steps, in order")
          VStack(spacing: IntradaSpacing.controlGap) {
            ForEach(Array(labels.indices), id: \.self) { index in
              stepRow(index)
            }
          }
          AddRowButton(title: "Add a step", style: .plain) {
            labels.append("")
          }
        }
        .padding(IntradaSpacing.card)
      }
    }
  }

  private func stepRow(_ index: Int) -> some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      TextField("e.g. C", text: Binding(get: { labels[index] }, set: { labels[index] = $0 }))
        .font(IntradaFont.field)
        .foregroundStyle(IntradaColor.ink)
        .padding(IntradaSpacing.cardCompact)
        .cardSurface(cornerRadius: IntradaRadius.control)
      if labels.count > 1 {
        Button {
          labels.remove(at: index)
        } label: {
          Image(systemName: "minus.circle")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.danger)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Remove step \(index + 1)")
      }
    }
  }

  private var trimmedLabels: [String] {
    Self.trimmedLabels(labels)
  }

  /// Trims and drops blank rows — pulled out as a static func so it's directly
  /// testable, same as `ReflectionSheet.resolvedAchievedTempo`.
  static func trimmedLabels(_ labels: [String]) -> [String] {
    labels.map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }.filter { !$0.isEmpty }
  }

  private func save() {
    let trimmed = trimmedLabels
    guard !trimmed.isEmpty else { return }
    store.send(.item(.setVariants(id: itemId, labels: trimmed)))
  }
}

#if DEBUG
  #Preview("Add steps") {
    Color.black.opacity(0.2).ignoresSafeArea()
      .sheet(isPresented: .constant(true)) {
        AddStepsSheet(itemId: "preview-item").environment(Store.preview)
      }
  }
#endif
