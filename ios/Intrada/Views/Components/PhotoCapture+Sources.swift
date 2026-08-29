import PhotosUI
import SwiftUI
import VisionKit

/// The capture half of a photo surface: the scanner, the library picker, and
/// writing the bytes before anything names them. Shared by `PhotoCard` (the
/// photo on a piece) and `ScanPageEntry` (the photo the add form is read from),
/// which otherwise duplicate every step of it.
///
/// The order inside `write` is the whole point: an item can never end up naming
/// a file that was never written.
@MainActor
struct PhotoCaptureSources {
  /// Called with a freshly minted photo id once its bytes are on disk.
  let onCaptured: (String) -> Void
  /// Called with a message written for the user, never a raw error.
  let onFailure: (String) -> Void

  static var canScan: Bool { VNDocumentCameraViewController.isSupported }

  func finished(scanning capture: PhotoCapture) {
    switch capture {
    case .captured(let image): write(image)
    case .cancelled: break
    case .failed(let error):
      report(error, "document scan")
      onFailure("Couldn't scan the page. Try again.")
    }
  }

  func loaded(_ item: PhotosPickerItem) async {
    do {
      guard let data = try await item.loadTransferable(type: Data.self),
        let image = UIImage(data: data)
      else {
        onFailure("Couldn't read that photo. Try another.")
        return
      }
      write(image)
    } catch {
      report(error, "photo library load")
      onFailure("Couldn't read that photo. Try another.")
    }
  }

  private func write(_ image: UIImage) {
    let photoId = Ulid.generate()
    do {
      try PhotoFileStore.write(image, id: photoId)
    } catch {
      report(error, "photo write")
      onFailure("Couldn't save the photo. Try again.")
      return
    }
    onCaptured(photoId)
  }
}
