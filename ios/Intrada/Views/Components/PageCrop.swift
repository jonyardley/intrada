import CoreImage
import UIKit
import Vision

/// Crops a photographed page out of whatever it was lying on and flattens the
/// perspective, as the scanner does for its own route. Vision reads text out of
/// the surround, and that competes with the real title (#1436).
enum PageCrop {
  /// Returns the original whenever no page is found: never lose the photo.
  /// One context, not one per photo: it spins up a Metal command queue.
  private static let context = CIContext()

  static func toPage(_ image: UIImage) -> UIImage {
    // `cgImage` is the raw buffer with the EXIF rotation split off into
    // `imageOrientation`, and both Vision and CoreImage below assume `.up`.
    // A portrait photo would otherwise be cropped, stored and read sideways.
    let upright = image.imageOrientation == .up ? image : redrawnUpright(image)
    guard let cgImage = upright.cgImage else { return image }

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
      let rendered = context.createCGImage(corrected, from: corrected.extent)
    else {
      return image
    }

    return UIImage(cgImage: rendered)
  }

  /// Internal so it is testable without Vision, which `toPage` is not.
  static func redrawnUpright(_ image: UIImage) -> UIImage {
    let format = UIGraphicsImageRendererFormat.default()
    format.scale = image.scale
    return UIGraphicsImageRenderer(size: image.size, format: format).image { _ in
      image.draw(in: CGRect(origin: .zero, size: image.size))
    }
  }
}
