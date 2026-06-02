import SwiftUI

/// Bottom sheet for filtering the Library by tags. Lists the whole tag
/// vocabulary; tapping toggles a tag in the active filter (the core applies OR
/// semantics — any selected tag matches). The shell supplies `available` +
/// `selected` and writes back via `onChange`; this view owns no domain logic.
struct TagFilterSheet: View {
  let available: [String]
  let selected: [String]
  let onChange: ([String]) -> Void

  @Environment(\.dismiss) private var dismiss

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        if available.isEmpty {
          PlaceholderContent(
            systemImage: "tag",
            message: "No tags yet. Add tags to items to filter by them.")
        } else {
          ScrollView {
            VStack(spacing: 0) {
              ForEach(available, id: \.self) { tag in
                let isOn = isSelected(tag)
                Button {
                  toggle(tag, isOn: isOn)
                } label: {
                  HStack(spacing: 12) {
                    Text(tag)
                      .font(IntradaFont.body)
                      .foregroundStyle(IntradaColor.ink)
                    Spacer(minLength: 0)
                    if isOn {
                      Image(systemName: "checkmark")
                        .font(IntradaFont.bodyMedium)
                        .foregroundStyle(IntradaColor.accent)
                    }
                  }
                  .padding(.vertical, 14)
                  .padding(.horizontal, 16)
                  .frame(maxWidth: .infinity, alignment: .leading)
                  .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityAddTraits(isOn ? [.isButton, .isSelected] : .isButton)

                if tag != available.last {
                  HairlineDivider().padding(.leading, 16)
                }
              }
            }
            .cardSurface()
            .padding(16)
          }
        }
      }
      .navigationTitle("Filter by tag")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Clear", action: { onChange([]) })
            .disabled(selected.isEmpty)
        }
        ToolbarItem(placement: .confirmationAction) {
          Button("Done", action: { dismiss() })
        }
      }
    }
    .presentationDetents([.medium, .large])
  }

  private func isSelected(_ tag: String) -> Bool {
    selected.contains { $0.localizedCaseInsensitiveCompare(tag) == .orderedSame }
  }

  private func toggle(_ tag: String, isOn: Bool) {
    var next = selected.filter { $0.localizedCaseInsensitiveCompare(tag) != .orderedSame }
    if !isOn { next.append(tag) }
    onChange(next)
    UISelectionFeedbackGenerator().selectionChanged()
  }
}

#if DEBUG
  #Preview {
    TagFilterSheet(
      available: ["classical", "jazz", "recital", "warm-up", "technique"],
      selected: ["jazz", "recital"],
      onChange: { _ in })
  }
#endif
