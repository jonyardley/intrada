import AVFoundation
import SwiftUI

/// The camera we own, in place of `VNDocumentCameraViewController`: that one
/// fired its own shutter while you were reaching for Save, and kept the first
/// of however many shots it took without saying so (#1460). One press, then the
/// flattened page to approve before it is stored.
struct PageCameraScreen: View {
  let onFinish: (PhotoCapture) -> Void

  private let device: any PageCameraDevice
  @State private var stage: Stage = .preparing
  @State private var failure: String?
  @State private var capturing = false

  init(device: (any PageCameraDevice)? = nil, onFinish: @escaping (PhotoCapture) -> Void) {
    self.device = device ?? AVPageCameraDevice()
    self.onFinish = onFinish
  }

  private enum Stage {
    case preparing
    case live
    /// The cropped page, not the raw frame: what you approve is what is stored.
    case captured(UIImage)
    case blocked(PageCameraAccess)
  }

  var body: some View {
    ZStack {
      IntradaColor.viewerBackdrop.ignoresSafeArea()
      stageContent
      chrome
    }
    .task { await begin() }
    .onDisappear { device.stop() }
  }

  @ViewBuilder private var stageContent: some View {
    switch stage {
    case .preparing:
      EmptyView()
    case .live:
      if let session = device.session {
        CameraPreview(session: session).ignoresSafeArea()
      }
    case .captured(let page):
      CapturedPageConfirm(page: page, onKeep: keep, onRetake: { stage = .live })
    case .blocked(let access):
      PageCameraBlocked(access: access, onOpenSettings: openSettings)
    }
  }

  private var chrome: some View {
    VStack(spacing: 0) {
      HStack {
        Button("Cancel", action: cancel)
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.onAccent)
        Spacer()
      }
      .padding(IntradaSpacing.card)

      Spacer()

      if let failure {
        PageCameraFailure(message: failure)
          .padding(.bottom, IntradaSpacing.card)
      }

      if case .live = stage {
        PageCameraShutter(disabled: capturing, onPress: takePhoto)
          .padding(.bottom, IntradaSpacing.section)
      }
    }
  }

  // ── Actions ──

  private func begin() async {
    let access = await device.authorise()
    guard access == .allowed else {
      stage = .blocked(access)
      return
    }
    do {
      try await device.start()
      stage = .live
    } catch {
      report(error, "page camera start")
      stage = .blocked(.unavailable)
    }
  }

  private func takePhoto() {
    guard !capturing else { return }
    capturing = true
    failure = nil
    UIImpactFeedbackGenerator(style: .light).impactOccurred()

    Task {
      defer { capturing = false }
      do {
        let shot = try await device.capture()
        // The system scanner cropped its own output; ours has to, and the work
        // blocks long enough to drop frames.
        let page = await Task.detached(priority: .userInitiated) { PageCrop.toPage(shot) }.value
        stage = .captured(page)
      } catch {
        report(error, "page camera capture")
        failure = "Couldn't take the photo. Try again."
      }
    }
  }

  private func keep() {
    guard case .captured(let page) = stage else { return }
    device.stop()
    onFinish(.captured(page))
  }

  private func cancel() {
    device.stop()
    onFinish(.cancelled)
  }

  private func openSettings() {
    guard let url = URL(string: UIApplication.openSettingsURLString) else { return }
    UIApplication.shared.open(url)
  }
}

// ── The states, each renderable without a camera ──

struct PageCameraShutter: View {
  let disabled: Bool
  let onPress: () -> Void

  var body: some View {
    Button(action: onPress) {
      ZStack {
        Circle()
          .strokeBorder(IntradaColor.onAccent.opacity(0.6), lineWidth: 3)
          .frame(width: 76, height: 76)
        Circle()
          .fill(IntradaColor.onAccent)
          .frame(width: 62, height: 62)
      }
    }
    .disabled(disabled)
    .accessibilityLabel("Take the photo")
  }
}

/// The flattened page as it will be stored, with the two ways out. Approving
/// what was kept is the whole fix: the scanner discarded shots silently.
struct CapturedPageConfirm: View {
  let page: UIImage
  let onKeep: () -> Void
  let onRetake: () -> Void

  var body: some View {
    VStack(spacing: IntradaSpacing.section) {
      Image(uiImage: page)
        .resizable()
        .scaledToFit()
        .accessibilityLabel("The page you photographed")

      HStack(spacing: IntradaSpacing.section) {
        Button("Retake", action: onRetake)
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.onAccent)

        Button("Use this photo", action: onKeep)
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.onAccent)
          .padding(.horizontal, IntradaSpacing.section)
          .padding(.vertical, IntradaSpacing.cardCompact)
          .background(LinearGradient.brandBar, in: Capsule())
      }
    }
    .padding(.vertical, IntradaSpacing.section)
  }
}

struct PageCameraBlocked: View {
  let access: PageCameraAccess
  let onOpenSettings: () -> Void

  var body: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      Text(access == .denied ? "Camera access is off" : "No camera on this device")
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.onAccent)

      if access == .denied {
        Text("Turn it on in Settings to photograph a page.")
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.onAccent.opacity(0.75))
          .multilineTextAlignment(.center)

        Button("Open Settings", action: onOpenSettings)
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.onAccent)
          .padding(.horizontal, IntradaSpacing.section)
          .padding(.vertical, IntradaSpacing.cardCompact)
          .background(LinearGradient.brandBar, in: Capsule())
      }
    }
    .padding(IntradaSpacing.section)
  }
}

struct PageCameraFailure: View {
  let message: String

  var body: some View {
    Text(message)
      .font(IntradaFont.body)
      .foregroundStyle(IntradaColor.onAccent)
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.vertical, IntradaSpacing.controlGap)
      .background(IntradaColor.danger, in: RoundedRectangle(cornerRadius: IntradaRadius.badge))
  }
}

/// `AVCaptureVideoPreviewLayer` is a layer, not a view, so it needs a `UIView`
/// backed by one rather than a sublayer that has to be resized by hand.
private struct CameraPreview: UIViewRepresentable {
  let session: AVCaptureSession

  func makeUIView(context _: Context) -> PreviewView {
    let view = PreviewView()
    view.previewLayer?.session = session
    view.previewLayer?.videoGravity = .resizeAspectFill
    return view
  }

  func updateUIView(_ view: PreviewView, context _: Context) {
    view.previewLayer?.session = session
  }
}

private final class PreviewView: UIView {
  override class var layerClass: AnyClass { AVCaptureVideoPreviewLayer.self }
  var previewLayer: AVCaptureVideoPreviewLayer? { layer as? AVCaptureVideoPreviewLayer }
}
