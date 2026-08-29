import UIKit

/// Where an item's photo bytes live. The core holds only the id (spec decision
/// 1: a blob in GRDB fattens every row read on a table already loaded whole).
/// Nothing here deletes: removing a photo tombstones the id and leaves the
/// bytes (spec decision 2). Reaping is #1442.
enum PhotoFileStore {
  enum Failure: Error {
    case couldNotEncode
  }

  /// JPEG, not the HEIC the spec first wrote (open question 6, settled
  /// 2026-08-29): the scanner hands back a `UIImage` either way, so neither is
  /// the camera's native format here, and HEIC has no encoder on some simulator
  /// configurations the tests run on.
  private static let quality: CGFloat = 0.8
  /// A page stays readable zoomed at this size, at roughly 400 to 600 KB.
  private static let longestEdge: CGFloat = 2048

  static func url(for photoId: String) throws -> URL {
    try directory().appendingPathComponent("\(photoId).jpg")
  }

  @discardableResult
  static func write(_ image: UIImage, id photoId: String) throws -> URL {
    guard let data = downscaled(image).jpegData(compressionQuality: quality) else {
      throw Failure.couldNotEncode
    }
    let destination = try url(for: photoId)
    try data.write(to: destination, options: .atomic)
    return destination
  }

  static func image(for photoId: String) -> UIImage? {
    guard let source = try? url(for: photoId) else { return nil }
    return UIImage(contentsOfFile: source.path)
  }

  private static func directory() throws -> URL {
    let support = try FileManager.default.url(
      for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
    let photos = support.appendingPathComponent("photos", isDirectory: true)
    try FileManager.default.createDirectory(at: photos, withIntermediateDirectories: true)
    return photos
  }

  /// In pixels, not points: `UIImage.size` is points, so a 2x or 3x image would
  /// otherwise sail past a cap it is four or nine times over.
  private static func downscaled(_ image: UIImage) -> UIImage {
    let longest = max(image.size.width, image.size.height) * image.scale
    guard longest > longestEdge else { return image }
    let ratio = longestEdge / longest
    let size = CGSize(
      width: image.size.width * image.scale * ratio,
      height: image.size.height * image.scale * ratio)
    let format = UIGraphicsImageRendererFormat.default()
    format.scale = 1
    return UIGraphicsImageRenderer(size: size, format: format).image { _ in
      image.draw(in: CGRect(origin: .zero, size: size))
    }
  }
}
