import PhotosUI
import SwiftUI
import VisionKit

/// Which capture route is on screen. Driven by `.photoCapture(...)`, which
/// owns the dismissal `VNDocumentCameraViewController` does not do itself —
/// forgetting that once left the scanner with no way out (#1436).
struct PhotoCaptureState {
  fileprivate(set) var scanning = false
  fileprivate(set) var choosingFromLibrary = false
  fileprivate var picked: PhotosPickerItem?

  mutating func scan() { scanning = true }
  mutating func chooseFromLibrary() { choosingFromLibrary = true }
}

extension View {
  func photoCapture(_ state: Binding<PhotoCaptureState>, using sources: PhotoCaptureSources)
    -> some View
  {
    fullScreenCover(isPresented: state.scanning) {
      DocumentScanner { capture in
        state.wrappedValue.scanning = false
        sources.finished(scanning: capture)
      }
      .ignoresSafeArea()
    }
    .photosPicker(
      isPresented: state.choosingFromLibrary, selection: state.picked, matching: .images
    )
    .onChange(of: state.wrappedValue.picked) { _, item in
      guard let item else { return }
      state.wrappedValue.picked = nil
      Task { await sources.loaded(item) }
    }
  }
}

/// The capture half of a photo surface: the scanner, the library picker, and
/// writing the bytes before anything names them. Shared by `PhotoCard` (the
/// photo on a piece) and `ScanPageEntry` (the photo the add form is read from),
/// which otherwise duplicate every step of it.
@MainActor
struct PhotoCaptureSources {
  let onCaptured: (String) -> Void
  /// Takes a message written for the user, never a raw error.
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

  /// Bytes first, then whoever names them. The order is the whole point: an
  /// item can never end up naming a file that was never written.
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
