import SwiftUI

extension View {
  /// A trailing swipe that reveals a delete button, for rows outside a `List`
  /// where `.swipeActions` does nothing. A long swipe deletes at once. The
  /// gesture is an accelerator only (T4): the same action is offered to
  /// assistive tech here and as a visible button elsewhere.
  func swipeToDelete(label: String, action: @escaping () -> Void) -> some View {
    modifier(SwipeToDelete(label: label, action: action))
  }
}

private struct SwipeToDelete: ViewModifier {
  let label: String
  let action: () -> Void

  @State private var offset: CGFloat = 0
  @State private var settled: CGFloat = 0

  private let revealWidth: CGFloat = 88
  private let fullSwipe: CGFloat = 180

  func body(content: Content) -> some View {
    content
      .offset(x: offset)
      .background(alignment: .trailing) {
        if offset < 0 { revealed }
      }
      .simultaneousGesture(drag)
      .accessibilityAction(named: label, perform)
  }

  private var revealed: some View {
    Button(action: perform) {
      Image(systemName: "trash")
        .iconSize(.inline, weight: .medium)
        .foregroundStyle(IntradaColor.onAccent)
        .frame(width: max(revealWidth, -offset))
        .frame(maxHeight: .infinity)
        .background(IntradaColor.danger)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    }
    .buttonStyle(.plain)
    .accessibilityLabel(label)
  }

  private var drag: some Gesture {
    DragGesture(minimumDistance: 20)
      .onChanged { value in
        guard abs(value.translation.width) > abs(value.translation.height) else { return }
        offset = min(0, settled + value.translation.width)
      }
      .onEnded { _ in
        if offset <= -fullSwipe {
          perform()
        } else {
          settle(at: offset < -revealWidth / 2 ? -revealWidth : 0)
        }
      }
  }

  private func settle(at target: CGFloat) {
    withAnimation(IntradaMotion.snappy) { offset = target }
    settled = target
  }

  private func perform() {
    settle(at: 0)
    action()
  }
}
