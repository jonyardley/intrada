import SwiftUI

struct SectionEmptyState<Action: View>: View {
  let message: String
  @ViewBuilder let action: Action

  init(_ message: String, @ViewBuilder action: () -> Action) {
    self.message = message
    self.action = action()
  }

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Text(message)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkSecondary)
        .fixedSize(horizontal: false, vertical: true)
      action
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.bottom, IntradaSpacing.card)
  }
}

extension SectionEmptyState where Action == EmptyView {
  init(_ message: String) {
    self.init(message) { EmptyView() }
  }
}
