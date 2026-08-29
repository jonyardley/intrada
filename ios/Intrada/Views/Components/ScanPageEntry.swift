import PhotosUI
import SwiftUI

/// The way into recognition on the add screen (#1436): photograph the page, and
/// the form arrives holding what could be read off it. Sits above the fields,
/// not among them — a scanned page is what fills the form, not a value in it.
struct ScanPageEntry: View {
  let photoId: String?
  let reading: Bool
  let onCaptured: (String) -> Void

  @State private var scanning = false
  @State private var choosingFromLibrary = false
  @State private var picked: PhotosPickerItem?
  @State private var failure: String?

  /// Snapshot tests hand over a fixed image rather than reaching into the
  /// simulator's Application Support directory.
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
    .fullScreenCover(isPresented: $scanning) {
      DocumentScanner {
        scanning = false
        capture.finished(scanning: $0)
      }
      .ignoresSafeArea()
    }
    .photosPicker(isPresented: $choosingFromLibrary, selection: $picked, matching: .images)
    .onChange(of: picked) { _, item in
      guard let item else { return }
      picked = nil
      Task { await capture.loaded(item) }
    }
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
      Image(uiImage: image)
        .resizable()
        .scaledToFill()
        .frame(width: 40, height: 52)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.badge))
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.badge)
            .stroke(IntradaColor.hairline, lineWidth: 1))
      Text(reading ? "Reading the page" : "Scanned page")
        .font(IntradaFont.subtitle)
        .foregroundStyle(IntradaColor.inkSecondary)
      Spacer(minLength: 0)
      if reading {
        ProgressView().controlSize(.small)
      } else {
        sourceMenu {
          Text("Rescan")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.accent)
        }
        .accessibilityLabel("Scan a different page")
      }
    }
    .padding(.horizontal, IntradaSpacing.cardCompact)
    .padding(.vertical, 10)
  }

  private func sourceMenu<Trigger: View>(@ViewBuilder label: () -> Trigger) -> some View {
    Menu {
      if PhotoCaptureSources.canScan {
        Button {
          failure = nil
          scanning = true
        } label: {
          Label("Scan the page", systemImage: "doc.viewfinder")
        }
      }
      Button {
        failure = nil
        choosingFromLibrary = true
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
