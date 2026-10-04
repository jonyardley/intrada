import SwiftUI

/// Drag-to-reorder for a list of rows, driven from a handle on each row.
// A gesture on the handle rather than the system drag and drop, which never
// delivered a drop to these rows on the simulator (#1783): the row moves as the
// finger passes half of its neighbour, so the list reorders under the finger.
@Observable
@MainActor
final class DragReorder<ID: Hashable> {
  private(set) var dragged: ID?
  private(set) var travel: CGFloat = 0
  fileprivate var rowPitch: CGFloat = 44
  private var rowsPassed = 0

  func gesture(
    for id: ID, ids: @escaping () -> [ID], reduceMotion: Bool,
    move: @escaping (_ id: ID, _ step: Int) -> Void
  ) -> some Gesture {
    DragGesture(minimumDistance: 4, coordinateSpace: .global)
      .onChanged { value in
        if self.dragged != id {
          self.dragged = id
          self.rowsPassed = 0
        }
        var travel = value.translation.height - CGFloat(self.rowsPassed) * self.rowPitch
        while abs(travel) > self.rowPitch / 2, let index = ids().firstIndex(of: id) {
          let step = travel > 0 ? 1 : -1
          guard ids().indices.contains(index + step) else { break }
          withAnimation(reduceMotion ? nil : IntradaMotion.standard) {
            move(id, step)
          }
          self.rowsPassed += step
          travel -= CGFloat(step) * self.rowPitch
          Haptic.selection.play()
        }
        self.travel = travel
      }
      .onEnded { _ in
        withAnimation(reduceMotion ? nil : IntradaMotion.standard) {
          self.dragged = nil
          self.travel = 0
        }
        self.rowsPassed = 0
      }
  }
}

extension Array where Element: Identifiable {
  mutating func move(_ id: Element.ID, by offset: Int) {
    guard let from = firstIndex(where: { $0.id == id }), indices.contains(from + offset) else {
      return
    }
    swapAt(from, from + offset)
  }
}

extension View {
  /// Lifts the row under the finger and lets its neighbours animate aside.
  func reorderableRow<ID: Hashable>(_ id: ID, in drag: DragReorder<ID>) -> some View {
    let isDragged = drag.dragged == id
    return
      self
      .background(isDragged ? IntradaColor.cardFill : Color.clear)
      .onGeometryChange(for: CGFloat.self) {
        $0.size.height
      } action: {
        drag.rowPitch = $0
      }
      .offset(y: isDragged ? drag.travel : 0)
      .zIndex(isDragged ? 1 : 0)
      .transaction { if isDragged { $0.animation = nil } }
  }
}
