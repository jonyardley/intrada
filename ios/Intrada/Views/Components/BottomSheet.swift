import SwiftUI

/// Shared bottom-sheet chrome. The inline title renders serif via RootView's
/// global nav-bar appearance; Done runs `onDone` then dismisses.
struct BottomSheet<Content: View, LeadingAction: View>: View {
  private let title: String
  private let detents: Set<PresentationDetent>
  private let onDone: () -> Void
  private let leadingAction: LeadingAction
  private let content: Content

  @Environment(\.dismiss) private var dismiss

  init(
    title: String,
    detents: Set<PresentationDetent> = [.medium, .large],
    onDone: @escaping () -> Void = {},
    @ViewBuilder leadingAction: () -> LeadingAction,
    @ViewBuilder content: () -> Content
  ) {
    self.title = title
    self.detents = detents
    self.onDone = onDone
    self.leadingAction = leadingAction()
    self.content = content()
  }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        content
      }
      .navigationTitle(title)
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) { leadingAction }
        ToolbarItem(placement: .confirmationAction) {
          Button("Done") {
            onDone()
            dismiss()
          }
        }
      }
    }
    .presentationDetents(detents)
  }
}

extension BottomSheet where LeadingAction == EmptyView {
  init(
    title: String,
    detents: Set<PresentationDetent> = [.medium, .large],
    onDone: @escaping () -> Void = {},
    @ViewBuilder content: () -> Content
  ) {
    self.init(
      title: title, detents: detents, onDone: onDone,
      leadingAction: { EmptyView() }, content: content)
  }
}
