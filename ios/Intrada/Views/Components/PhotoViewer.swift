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

  private let maxZoom: CGFloat = 6

  var body: some View {
    ZStack {
      IntradaColor.viewerBackdrop.ignoresSafeArea()

      Image(uiImage: image)
        .resizable()
        .scaledToFit()
        .scaleEffect(zoom)
        .offset(pan)
        .gesture(magnify)
        .simultaneousGesture(drag)
        .onTapGesture(count: 2) { toggleZoom() }
        .accessibilityLabel("Photo of the page")
    }
    // Inside the safe area, and on its own scrim: the page behind it is the
    // user's photo, so nothing can be assumed about the contrast under it.
    .overlay(alignment: .topLeading) {
      Button("Done") { dismiss() }
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent)
        .padding(.horizontal, IntradaSpacing.row)
        .padding(.vertical, IntradaSpacing.cardCompact)
        .background(IntradaColor.viewerBackdrop.opacity(0.55), in: Capsule())
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
        if zoom == 1 { pan = .zero }
      }
  }

  private var drag: some Gesture {
    DragGesture()
      .onChanged { value in
        guard zoom > 1 else { return }
        pan = CGSize(
          width: dragStart.width + value.translation.width,
          height: dragStart.height + value.translation.height)
      }
      .onEnded { _ in dragStart = pan }
  }

  private func toggleZoom() {
    zoom = zoom > 1 ? 1 : 2.5
    pinchStart = zoom
    if zoom == 1 {
      pan = .zero
      dragStart = .zero
    }
  }

  private func clamped(_ value: CGFloat) -> CGFloat { min(max(value, 1), maxZoom) }
}

#if DEBUG
  #Preview {
    PhotoViewer(image: UIImage(systemName: "doc.text.image") ?? UIImage())
  }
#endif
