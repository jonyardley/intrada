import PhotosUI
import SharedTypes
import SwiftUI

/// The photo an item carries as an aide-memoire (#1355): the page you practise
/// from, kept with the piece.
struct PhotoCard: View {
  let itemId: String
  let photoId: String?
  /// Injected so a snapshot test can hand over a fixed image instead of
  /// reaching into the simulator's Application Support directory.
  let loadImage: @MainActor (String) -> UIImage?

  @Environment(Store.self) private var store
  @State private var image: UIImage?
  @State private var captureState = PhotoCaptureState()
  @State private var confirmingRemove = false
  @State private var viewing = false
  @State private var failure: String?

  /// The first read is synchronous, not a `task`: the card renders with its
  /// photo already in place, so there is no frame where a piece that has one
  /// looks like a piece that does not.
  init(
    itemId: String, photoId: String?,
    loadImage: @escaping @MainActor (String) -> UIImage? = PhotoFileStore.image(for:)
  ) {
    self.itemId = itemId
    self.photoId = photoId
    self.loadImage = loadImage
    _image = State(initialValue: photoId.flatMap(loadImage))
  }

  var body: some View {
    VStack(spacing: 0) {
      header
      if let failure {
        FormErrorBanner(message: failure)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.bottom, IntradaSpacing.cardCompact)
          .transition(.opacity)
      }
      if let image {
        thumbnail(image)
      } else {
        emptyState
      }
    }
    .cardSurface()
    .onChange(of: photoId) { _, next in image = next.flatMap(loadImage) }
    .photoCapture($captureState, using: capture)
    .alert("Remove this photo?", isPresented: $confirmingRemove) {
      Button("Remove", role: .destructive, action: remove)
      Button("Cancel", role: .cancel) {}
    } message: {
      Text("You'll need to scan or choose it again.")
    }
  }

  // ── Layout ──

  private var header: some View {
    HStack {
      Text("Photo")
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.ink)
      Spacer()
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
        if photoId != nil {
          Button(role: .destructive) {
            confirmingRemove = true
          } label: {
            Label("Remove", systemImage: "trash")
          }
        }
      } label: {
        Text(photoId == nil ? "Add" : "Change")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
      }
      .accessibilityLabel(photoId == nil ? "Add a photo" : "Change the photo")
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.card)
    .padding(.bottom, image == nil ? IntradaSpacing.card : IntradaSpacing.cardCompact)
  }

  /// A stored id whose file has gone says so rather than posing as "no photo
  /// yet", which would hide Remove behind the very id it needs to clear.
  private var emptyState: some View {
    Text(photoId == nil ? "Keep the page you practise from." : "That photo is missing.")
      .font(IntradaFont.body)
      .foregroundStyle(IntradaColor.inkSecondary)
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.bottom, IntradaSpacing.card)
  }

  /// The viewer's cover hangs off the thumbnail, not the card: the image is
  /// already unwrapped here, so it cannot present with nothing to show, and it
  /// is not a second `fullScreenCover` competing with the scanner's on one view.
  private func thumbnail(_ image: UIImage) -> some View {
    Button {
      viewing = true
    } label: {
      Image(uiImage: image)
        .resizable()
        .scaledToFit()
        .frame(maxWidth: .infinity)
        .frame(height: 244)
        .background(IntradaColor.surfaceSunken)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.badge))
    }
    .buttonStyle(.plain)
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.bottom, IntradaSpacing.cardCompact)
    .accessibilityLabel("Photo of the page")
    .accessibilityHint("Opens it full screen")
    .fullScreenCover(isPresented: $viewing) { PhotoViewer(image: image) }
  }

  // ── Capture ──

  private var capture: PhotoCaptureSources {
    PhotoCaptureSources(
      onCaptured: { confirm(.item(.setPhoto(id: itemId, photoId: $0))) },
      onFailure: surface)
  }

  private func remove() {
    confirm(.item(.clearPhoto(id: itemId)))
  }

  /// Optimistic UI reconciles with the core's confirmed outcome, never before:
  /// a rejected event keeps its error on screen instead of feeling like it
  /// landed.
  private func confirm(_ event: Event) {
    failure = nil
    if store.send(event, onSuccess: .impact) {
      return
    }
    surface(store.viewModel?.error ?? "Couldn't save the photo. Try again.")
    store.send(.clearError)
  }

  private func surface(_ message: String) {
    withAnimation { failure = message }
    UINotificationFeedbackGenerator().notificationOccurred(.error)
    UIAccessibility.post(notification: .announcement, argument: "Error: \(message)")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.card) {
        PhotoCard(itemId: "p1", photoId: nil)
        PhotoCard(itemId: "p1", photoId: "01ARZ3NDEKTSV4RRFFQ69G5FAV") { _ in
          UIImage(systemName: "doc.text.image")
        }
      }
      .padding(IntradaSpacing.card)
    }
    .environment(Store.preview)
  }
#endif
