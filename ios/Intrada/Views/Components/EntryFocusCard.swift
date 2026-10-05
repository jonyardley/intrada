import SharedTypes
import SwiftUI

/// The builder's focus for one entry (#2303): what the intention checks, where
/// and against which number. Every tap sends; the core validates the target.
struct EntryFocusCard: View {
  let current: FocusView?
  let choices: [FocusChoiceView]
  let sections: [SectionView]
  let send: (IntentionFocus?) -> Void

  private var focus: IntentionFocus? { current?.focus }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      HStack {
        FieldLabel("Focus")
        Spacer()
        if focus != nil {
          Button("Clear") { send(nil) }
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(minHeight: 44)
            .accessibilityIdentifier("entrySettings.focusClear")
        }
      }
      FlowLayout(spacing: IntradaSpacing.controlGap) {
        ForEach(choices, id: \.kind) { choice in
          ChoiceBox(choice.label, isOn: focus?.kind == choice.kind) { pick(choice.kind) }
            .accessibilityIdentifier("entrySettings.focusKind")
        }
      }
      if let focus {
        if !sections.isEmpty {
          FieldLabel("Where")
          FlowLayout(spacing: IntradaSpacing.controlGap) {
            ChoiceBox("Whole piece", isOn: focus.sectionId == nil) { place(nil) }
            ForEach(sections, id: \.id) { section in
              ChoiceBox(section.label, isOn: focus.sectionId == section.id) { place(section.id) }
            }
          }
        }
        if let range = choices.first(where: { $0.kind == focus.kind })?.target {
          targetStepper(focus, range: range)
        }
      }
    }
    .fieldCardSurface()
  }

  private func targetStepper(_ focus: IntentionFocus, range: FocusTargetView) -> some View {
    let value = Binding(
      get: { Int(focus.target ?? range.min) },
      set: { next in
        var updated = focus
        updated.target = UInt16(next)
        send(updated)
      })
    return Stepper(
      value: value, in: Int(range.min)...Int(range.max), step: Int(range.step)
    ) {
      VStack(alignment: .leading, spacing: 2) {
        FieldLabel("Target")
        Text(current?.targetCaption ?? "")
          .font(IntradaFont.figure).foregroundStyle(IntradaColor.ink)
      }
    }
    .accessibilityIdentifier("entrySettings.focusTarget")
  }

  private func pick(_ kind: FocusKind) {
    guard focus?.kind != kind else { return }
    send(IntentionFocus(kind: kind, sectionId: focus?.sectionId, target: nil))
  }

  private func place(_ sectionId: String?) {
    guard var updated = focus, updated.sectionId != sectionId else { return }
    updated.sectionId = sectionId
    send(updated)
  }
}

/// One option in a small grid of choices: the musician's highlighter when on.
private struct ChoiceBox: View {
  let title: String
  let isOn: Bool
  let action: () -> Void
  @Environment(\.marker) private var marker

  init(_ title: String, isOn: Bool, action: @escaping () -> Void) {
    self.title = title
    self.isOn = isOn
    self.action = action
  }

  var body: some View {
    Button(action: action) {
      Text(title)
        .font(IntradaFont.smallMedium)
        .foregroundStyle(isOn ? IntradaColor.onMarker : IntradaColor.ink)
        .padding(.horizontal, IntradaSpacing.cardCompact)
        .frame(minHeight: 44)
        .background(isOn ? marker : IntradaColor.cardFill)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.control))
        .overlay {
          RoundedRectangle(cornerRadius: IntradaRadius.control)
            .strokeBorder(isOn ? IntradaColor.ink : IntradaColor.hairline, lineWidth: 1)
        }
    }
    .buttonStyle(.plain)
    .accessibilityAddTraits(isOn ? .isSelected : [])
  }
}
