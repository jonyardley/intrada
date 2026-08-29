import CoreImage
import UIKit
import Vision

/// Crops a photographed page out of whatever it was lying on and flattens the
/// perspective, as the scanner does for its own route. Vision reads text out of
/// the surround, and that competes with the real title (#1436).
enum PageCrop {
  /// Returns the original whenever no page is found: never lose the photo.
  static func toPage(_ image: UIImage) -> UIImage {
    guard let cgImage = image.cgImage else { return image }

    let request = VNDetectDocumentSegmentationRequest()
    guard (try? VNImageRequestHandler(cgImage: cgImage, options: [:]).perform([request])) != nil,
      let page = request.results?.first
    else {
      return image
    }

    let source = CIImage(cgImage: cgImage)
    let extent = source.extent
    func point(_ normalised: CGPoint) -> CIVector {
      CIVector(x: normalised.x * extent.width, y: normalised.y * extent.height)
    }

    guard
      let corrected = CIFilter(
        name: "CIPerspectiveCorrection",
        parameters: [
          kCIInputImageKey: source,
          "inputTopLeft": point(page.topLeft),
          "inputTopRight": point(page.topRight),
          "inputBottomLeft": point(page.bottomLeft),
          "inputBottomRight": point(page.bottomRight),
        ])?.outputImage,
      let rendered = CIContext().createCGImage(corrected, from: corrected.extent)
    else {
      return image
    }

    return UIImage(cgImage: rendered)
  }
}
