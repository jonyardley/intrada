import Testing
import UIKit

@testable import Intrada

/// The encode runs off the main actor (#1450), so the write path's ordering is
/// something a test has to hold down rather than something the call stack gives.
@MainActor
struct PhotoCaptureSourcesTests {
  private func discard(_ photoId: String) {
    guard let url = try? PhotoFileStore.url(for: photoId) else { return }
    try? FileManager.default.removeItem(at: url)
  }

  private func page(_ size: CGSize) -> UIImage {
    UIGraphicsImageRenderer(size: size).image { context in
      UIColor.white.setFill()
      context.fill(CGRect(origin: .zero, size: size))
    }
  }

  @Test func theBytesAreOnDiskBeforeAnythingIsToldTheirName() async {
    var named: String?
    var onDiskWhenNamed = false
    var failure: String?
    let sources = PhotoCaptureSources(
      onCaptured: { photoId in
        named = photoId
        guard let url = try? PhotoFileStore.url(for: photoId) else { return }
        onDiskWhenNamed = FileManager.default.fileExists(atPath: url.path)
      },
      onFailure: { failure = $0 })

    await sources.finished(scanning: .captured(page(CGSize(width: 400, height: 300))))

    defer { if let named { discard(named) } }
    #expect(failure == nil)
    #expect(named != nil)
    #expect(onDiskWhenNamed)
  }

  @Test func cancellingNamesNothing() async {
    var named: String?
    var failure: String?
    let sources = PhotoCaptureSources(onCaptured: { named = $0 }, onFailure: { failure = $0 })

    await sources.finished(scanning: .cancelled)

    #expect(named == nil)
    #expect(failure == nil)
  }
}
