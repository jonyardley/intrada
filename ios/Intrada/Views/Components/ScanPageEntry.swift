import PhotosUI
import SharedTypes
import SwiftUI

/// The way into recognition on the add screen (#1436): photograph the page, and
/// the form arrives holding what could be read off it. Sits above the fields,
/// not among them — a scanned page is what fills the form, not a value in it.
struct ScanPageEntry: View {
  let photoId: String?
  let status: PhotoRecognitionStatus
  /// A photograph of something that is not a page reads cleanly and finds
  /// nothing, which would otherwise look exactly like a good scan.
  let readNothing: Bool
  let onCaptured: (String) -> Void

  @State private var captureState = PhotoCaptureState()
  @State private var failure: String?
  @State private var viewing = false

  /// Injected so a snapshot test needs no Application Support directory.
  var loadImage: @MainActor (String) -> UIImage? = PhotoFileStore.image(for:)

  var body: some View {
    VStack(spacing: 0) {
      if let failure {
        FormErrorBanner(message: failure)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.top, IntradaSpacing.cardCompact)
          .transition(.opacity)
      }
      content
    }
    .cardSurface()
    .photoCapture($captureState, using: capture)
  }

  @ViewBuilder private var content: some View {
    if let photoId, let image = loadImage(photoId) {
      scanned(image)
    } else {
      prompt
    }
  }

  private var prompt: some View {
    sourceMenu {
      HStack(spacing: 10) {
        Image(systemName: "doc.viewfinder")
          .font(.system(size: 20, weight: .regular))
          .foregroundStyle(IntradaColor.accent)
        VStack(alignment: .leading, spacing: 2) {
          Text("Scan a page")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.accent)
          Text("Title, composer and tempo, read off the page")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
            .multilineTextAlignment(.leading)
        }
        Spacer(minLength: 0)
      }
      .padding(IntradaSpacing.card)
    }
    .accessibilityLabel("Scan a page")
    .accessibilityHint("Reads the title, composer and tempo off a photo of the music")
  }

  private func scanned(_ image: UIImage) -> some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      // The scanner shows nothing back after the shutter, so without this there
      // is no way to see what was actually captured, or why a field read oddly.
      Button {
        viewing = true
      } label: {
        Image(uiImage: image)
          .resizable()
          .scaledToFill()
          .frame(width: 40, height: 52)
          .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.badge))
          .overlay(
            RoundedRectangle(cornerRadius: IntradaRadius.badge)
              .stroke(IntradaColor.hairline, lineWidth: 1))
      }
      .buttonStyle(.plain)
      .accessibilityLabel("The page you scanned")
      .accessibilityHint("Opens it full screen")
      .fullScreenCover(isPresented: $viewing) { PhotoViewer(image: image) }
      Text(outcome)
        .font(IntradaFont.subtitle)
        .foregroundStyle(IntradaColor.inkSecondary)
      Spacer(minLength: 0)
      switch status {
      case .reading:
        ProgressView().controlSize(.small)
      case .unsupported:
        // Nothing to offer: another page reads no better on a device that
        // cannot read one at all.
        EmptyView()
      default:
        sourceMenu {
          Text("Change")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.accent)
        }
        .accessibilityLabel("Use a different page")
      }
    }
    .padding(.horizontal, IntradaSpacing.cardCompact)
    .padding(.vertical, 10)
  }

  /// Every outcome the core models says something: a read that failed must not
  /// look identical to one that worked, after the user waited for it.
  private var outcome: String {
    switch status {
    case .reading: "Reading the page"
    case .failed: "Couldn't read that page. Type the fields instead."
    case .unsupported: "This phone can't read a page. Type the fields instead."
    case .ready where readNothing: "Nothing to read on that page."
    default: "Scanned page"
    }
  }

  private func sourceMenu<Trigger: View>(@ViewBuilder label: () -> Trigger) -> some View {
    Menu {
      if PhotoCaptureSources.canScan {
        Button {
          failure = nil
          captureState.scan()
        } label: {
          Label("Scan the page", systemImage: "doc.viewfinder")
        }
      }
      Button {
        failure = nil
        captureState.chooseFromLibrary()
      } label: {
        Label("Choose a photo", systemImage: "photo.on.rectangle")
      }
    } label: {
      label()
    }
  }

  private var capture: PhotoCaptureSources {
    PhotoCaptureSources(
      onCaptured: { photoId in
        failure = nil
        onCaptured(photoId)
      },
      onFailure: { message in
        withAnimation { failure = message }
        UINotificationFeedbackGenerator().notificationOccurred(.error)
        UIAccessibility.post(notification: .announcement, argument: "Error: \(message)")
      })
  }
}
