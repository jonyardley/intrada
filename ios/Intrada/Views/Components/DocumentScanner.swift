import SwiftUI
import VisionKit

enum PhotoCapture {
  case captured(UIImage)
  case cancelled
  case failed(Error)
}

/// `VNDocumentCameraViewController` in SwiftUI clothing: it finds the page
/// edges and flattens the perspective, which a plain camera does not. Only the
/// first page is kept — one photo per item is iteration 1 of #1355. Callers
/// gate on `isSupported`, which is false on the simulator.
struct DocumentScanner: UIViewControllerRepresentable {
  let onFinish: (PhotoCapture) -> Void

  func makeUIViewController(context: Context) -> VNDocumentCameraViewController {
    let controller = VNDocumentCameraViewController()
    controller.delegate = context.coordinator
    return controller
  }

  func updateUIViewController(_: VNDocumentCameraViewController, context _: Context) {}

  func makeCoordinator() -> Coordinator { Coordinator(onFinish: onFinish) }

  final class Coordinator: NSObject, VNDocumentCameraViewControllerDelegate {
    private let onFinish: (PhotoCapture) -> Void

    init(onFinish: @escaping (PhotoCapture) -> Void) {
      self.onFinish = onFinish
    }

    func documentCameraViewController(
      _: VNDocumentCameraViewController, didFinishWith scan: VNDocumentCameraScan
    ) {
      onFinish(scan.pageCount > 0 ? .captured(scan.imageOfPage(at: 0)) : .cancelled)
    }

    func documentCameraViewControllerDidCancel(_: VNDocumentCameraViewController) {
      onFinish(.cancelled)
    }

    func documentCameraViewController(
      _: VNDocumentCameraViewController, didFailWithError error: Error
    ) {
      onFinish(.failed(error))
    }
  }
}
