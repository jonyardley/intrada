import Testing
import UIKit

@testable import Intrada

/// The bytes behind a photo id. The store never deletes (spec decision 2), so
/// there is nothing to test on that side. Assertions are in pixels: a stored
/// photo is re-read at scale 1, so its `size` is its pixel count.
@MainActor
struct PhotoFileStoreTests {
  private func page(_ size: CGSize, scale: CGFloat = 1) -> UIImage {
    let format = UIGraphicsImageRendererFormat.default()
    format.scale = scale
    return UIGraphicsImageRenderer(size: size, format: format).image { context in
      UIColor.white.setFill()
      context.fill(CGRect(origin: .zero, size: size))
    }
  }

  @Test func writesAPhotoThatReadsBack() throws {
    let photoId = Ulid.generate()

    try PhotoFileStore.write(page(CGSize(width: 300, height: 400)), id: photoId)

    #expect(PhotoFileStore.image(for: photoId) != nil)
  }

  @Test func readingAnIdNothingWasWrittenForIsNil() {
    #expect(PhotoFileStore.image(for: Ulid.generate()) == nil)
  }

  @Test func bringsACameraSizedPageDownToTheLongEdgeCap() throws {
    let photoId = Ulid.generate()

    try PhotoFileStore.write(page(CGSize(width: 4032, height: 3024)), id: photoId)

    let stored = try #require(PhotoFileStore.image(for: photoId))
    #expect(max(stored.size.width, stored.size.height) == 2048)
    #expect(abs(stored.size.width / stored.size.height - 4032.0 / 3024.0) < 0.01)
  }

  /// The cap is a pixel count. A 1024pt page at 3x is 3072px across and has to
  /// come down, even though its `size` is comfortably under the limit.
  @Test func measuresTheCapInPixelsNotPoints() throws {
    let photoId = Ulid.generate()

    try PhotoFileStore.write(page(CGSize(width: 1024, height: 768), scale: 3), id: photoId)

    let stored = try #require(PhotoFileStore.image(for: photoId))
    #expect(max(stored.size.width, stored.size.height) == 2048)
  }

  @Test func leavesAPageThatIsAlreadySmallEnoughAlone() throws {
    let photoId = Ulid.generate()

    try PhotoFileStore.write(page(CGSize(width: 1024, height: 768)), id: photoId)

    let stored = try #require(PhotoFileStore.image(for: photoId))
    #expect(stored.size == CGSize(width: 1024, height: 768))
  }
}
