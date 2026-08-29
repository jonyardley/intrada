import Foundation

/// The image files behind `Item.photoId` (#1355). A protocol so a test can
/// inject a failing fake and check the core is told `.failed` rather than a
/// phantom ack (#816), mirroring `ItemStore`.
protocol PhotoStore {
  func delete(photoId: String) throws
}

/// Photos as files in the app container, not rows and not bytes on the bridge:
/// `loadItems` builds every `Item` eagerly and `Item` crosses the FFI bridge on
/// every render, so a megabyte-scale photo in either place would be copied on
/// every hydration and every frame (specs/piece-from-photo.md).
///
/// The shell owns the layout; the core owns the *decisions* and sends only an
/// opaque ulid, which it validates before storing precisely because it lands
/// here as a path component.
struct FilePhotoStore: PhotoStore {
  private let directory: URL

  init(directory: URL) {
    self.directory = directory
  }

  /// `create: false` so merely constructing a Store never touches disk — the
  /// directory is made by the write path, and a delete against a photo that was
  /// never written is a no-op either way.
  static func inAppContainer() throws -> FilePhotoStore {
    let support = try FileManager.default.url(
      for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: false)
    return FilePhotoStore(directory: support.appendingPathComponent("photos", isDirectory: true))
  }

  func delete(photoId: String) throws {
    let url = directory.appendingPathComponent("\(photoId).jpg", isDirectory: false)
    guard FileManager.default.fileExists(atPath: url.path) else { return }
    try FileManager.default.removeItem(at: url)
  }
}
