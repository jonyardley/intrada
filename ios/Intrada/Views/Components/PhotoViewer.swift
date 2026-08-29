import SwiftUI

/// The photo full screen, because a card-sized thumbnail of a page of music is
/// not readable.
struct PhotoViewer: View {
  let image: UIImage

  @Environment(\.dismiss) private var dismiss
  @State private var zoom: CGFloat = 1
  @State private var pinchStart: CGFloat = 1
  @State private var pan: CGSize = .zero
  @State private var dragStart: CGSize = .zero
  @State private var displayed: CGSize = .zero

  private let maxZoom: CGFloat = 6

  var body: some View {
    ZStack {
      IntradaColor.viewerBackdrop.ignoresSafeArea()

      Image(uiImage: image)
        .resizable()
        .scaledToFit()
        .background(
          GeometryReader { proxy in
            Color.clear.onAppear { displayed = proxy.size }
          }
        )
        .scaleEffect(zoom)
        .offset(pan)
        .gesture(magnify)
        .simultaneousGesture(drag)
        .onTapGesture(count: 2) { toggleZoom() }
        .accessibilityLabel("Photo of the page")
    }
    // On its own scrim because the page behind it is the user's photo. At 0.8
    // the worst case (a white page) is 8:1; 0.55 was 3.4:1, under the AA floor.
    .overlay(alignment: .topLeading) {
      Button("Done") { dismiss() }
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent)
        .padding(.horizontal, IntradaSpacing.row)
        .padding(.vertical, IntradaSpacing.cardCompact)
        .background(IntradaColor.viewerBackdrop.opacity(0.8), in: Capsule())
        .contentShape(Capsule())
        .padding(IntradaSpacing.card)
    }
    .animation(IntradaMotion.standard, value: zoom)
  }

  private var magnify: some Gesture {
    MagnifyGesture()
      .onChanged { zoom = clamped(pinchStart * $0.magnification) }
      .onEnded { _ in
        pinchStart = zoom
        settle()
      }
  }

  private var drag: some Gesture {
    DragGesture()
      .onChanged { value in
        guard zoom > 1 else { return }
        pan = bounded(
          CGSize(
            width: dragStart.width + value.translation.width,
            height: dragStart.height + value.translation.height))
      }
      .onEnded { _ in dragStart = pan }
  }

  private func toggleZoom() {
    zoom = zoom > 1 ? 1 : 2.5
    pinchStart = zoom
    settle()
  }

  /// Both zoom paths land here so neither can leave `dragStart` holding an
  /// offset `pan` no longer has — the next drag would otherwise jump by it.
  private func settle() {
    pan = bounded(pan)
    dragStart = pan
  }

  /// Clamped to the overhang the zoom creates, so the page cannot be flicked
  /// off screen and left recoverable only by a gesture nothing advertises.
  private func bounded(_ offset: CGSize) -> CGSize {
    let slack = CGSize(
      width: max(0, displayed.width * (zoom - 1) / 2),
      height: max(0, displayed.height * (zoom - 1) / 2))
    return CGSize(
      width: min(max(offset.width, -slack.width), slack.width),
      height: min(max(offset.height, -slack.height), slack.height))
  }

  private func clamped(_ value: CGFloat) -> CGFloat { min(max(value, 1), maxZoom) }
}

#if DEBUG
  #Preview {
    PhotoViewer(image: UIImage(systemName: "doc.text.image") ?? UIImage())
  }
#endif
