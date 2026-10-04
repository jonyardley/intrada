import SharedTypes
import SwiftUI

/// Chooses the keys an item is practised in on the circle of fifths (#2247),
/// saved together on Done.
struct KeysSheet: View {
  let item: LibraryItemView

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var keys: [Key]
  @State private var formError: String?

  init(item: LibraryItemView, keys: [Key]? = nil) {
    self.item = item
    _keys = State(initialValue: keys ?? item.keys.map(\.key))
  }

  var body: some View {
    BottomSheet(
      title: "Keys", detents: [.large], dismissesOnDone: false, onDone: save,
      leadingAction: {
        Button("Cancel") { dismiss() }
          .accessibilityIdentifier("keysSheet.cancel")
      },
      content: { content }
    )
  }

  private var content: some View {
    ScrollView {
      VStack(spacing: IntradaSpacing.card) {
        if let formError {
          FormErrorBanner(message: formError)
        }
        KeyWheel(
          chosen: { KeySet.chosenSpelling(in: keys, ring: $0, mode: $1) },
          onTap: { ring, mode in
            keys = KeySet.tap(keys, ring: ring, mode: mode)
            Haptic.selection.play()
          },
          hub: {
            VStack(spacing: 0) {
              Text("\(keys.count)")
                .font(IntradaFont.title)
                .foregroundStyle(IntradaColor.ink)
              Text(keys.count == 1 ? "key chosen" : "keys chosen")
                .font(IntradaFont.secondary)
                .foregroundStyle(IntradaColor.inkSecondary)
            }
            .accessibilityElement(children: .combine)
          }
        )
        .padding(.vertical, IntradaSpacing.cardCompact)
        HStack(spacing: IntradaSpacing.controlGap) {
          chip("All major", identifier: "keysSheet.allMajor") {
            keys = KeySet.addingAll(.major, to: keys)
          }
          chip("All minor", identifier: "keysSheet.allMinor") {
            keys = KeySet.addingAll(.minor, to: keys)
          }
          chip("Clear", identifier: "keysSheet.clear") { keys = [] }
        }
        Text(
          "Tap a key to add it. Tap a chosen key with \u{21C5} to switch its spelling; tap again to remove it."
        )
        .font(IntradaFont.small)
        .foregroundStyle(IntradaColor.inkSecondary)
        .multilineTextAlignment(.center)
      }
      .padding(IntradaSpacing.card)
    }
  }

  private func chip(_ title: String, identifier: String, action: @escaping () -> Void)
    -> some View
  {
    Button {
      action()
      Haptic.impact.play()
    } label: {
      TagChip(title, style: .outlined)
    }
    .buttonStyle(.plain)
    .accessibilityIdentifier(identifier)
  }

  // Dismiss only once the core accepts; a refusal stays here (#1595).
  private func save() {
    guard keys != item.keys.map(\.key) else {
      dismiss()
      return
    }
    formError = nil
    let accepted = store.sendAccepted(.item(.updateKeys(id: item.id, keys: keys)))
    if let error = store.viewModel?.error ?? (accepted ? nil : "Couldn't save. Try again.") {
      formError = error
      store.send(.clearError)
      Haptic.error.play()
      UIAccessibility.post(notification: .announcement, argument: "Error: \(error)")
    } else {
      Haptic.success.play()
      dismiss()
    }
  }
}
