import SwiftUI

/// Shared bottom-sheet chrome; Done runs `onDone` then dismisses.
struct BottomSheet<Content: View, LeadingAction: View>: View {
  private let title: String
  private let detents: Set<PresentationDetent>
  private let confirmationLabel: String
  private let confirmationDisabled: Bool
  private let dismissesOnDone: Bool
  private let titleWrapsAtLargeText: Bool
  private let onDone: () -> Void
  private let leadingAction: LeadingAction
  private let content: Content

  @Environment(\.dismiss) private var dismiss
  @Environment(\.dynamicTypeSize) private var typeSize

  init(
    title: String,
    detents: Set<PresentationDetent> = [.medium, .large],
    confirmationLabel: String = "Done",
    confirmationDisabled: Bool = false,
    dismissesOnDone: Bool = true,
    titleWrapsAtLargeText: Bool = false,
    onDone: @escaping () -> Void = {},
    @ViewBuilder leadingAction: () -> LeadingAction,
    @ViewBuilder content: () -> Content
  ) {
    self.title = title
    self.detents = detents
    self.confirmationLabel = confirmationLabel
    self.confirmationDisabled = confirmationDisabled
    self.dismissesOnDone = dismissesOnDone
    self.titleWrapsAtLargeText = titleWrapsAtLargeText
    self.onDone = onDone
    self.leadingAction = leadingAction()
    self.content = content()
  }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        VStack(spacing: 0) {
          if titleInContent {
            Text(title)
              .font(IntradaFont.cardTitle())
              .foregroundStyle(IntradaColor.ink)
              .frame(maxWidth: .infinity, alignment: .leading)
              .padding(.horizontal, IntradaSpacing.card)
              .padding(.top, IntradaSpacing.controlGap)
              .accessibilityAddTraits(.isHeader)
          }
          content
        }
      }
      .navigationTitle(titleInContent ? "" : title)
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) { leadingAction }
        ToolbarItem(placement: .confirmationAction) {
          Button(confirmationLabel) {
            onDone()
            if dismissesOnDone { dismiss() }
          }
          .disabled(confirmationDisabled)
          .accessibilityIdentifier("sheet.done")
        }
      }
    }
    .presentationDetents(detents)
  }

  // The inline bar truncates a long title at accessibility sizes (#2125).
  private var titleInContent: Bool {
    titleWrapsAtLargeText && typeSize.isAccessibilitySize
  }
}

extension BottomSheet where LeadingAction == EmptyView {
  init(
    title: String,
    detents: Set<PresentationDetent> = [.medium, .large],
    confirmationLabel: String = "Done",
    confirmationDisabled: Bool = false,
    onDone: @escaping () -> Void = {},
    @ViewBuilder content: () -> Content
  ) {
    self.init(
      title: title, detents: detents, confirmationLabel: confirmationLabel,
      confirmationDisabled: confirmationDisabled, onDone: onDone,
      leadingAction: { EmptyView() }, content: content)
  }
}
