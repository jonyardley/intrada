import SwiftUI
import UIKit

/// Both ways in present from UIKit, so the form reaches over the player's
/// cover and other sheets, and one guard stops a second form stacking.
@MainActor
enum FeedbackPresenter {
  static func presentFromProfile() {
    let window = UIApplication.shared.connectedScenes
      .compactMap { $0 as? UIWindowScene }
      .flatMap(\.windows)
      .first(where: \.isKeyWindow)
    guard let window else { return }
    present(from: window, withScreenshot: false)
  }

  static func present(from window: UIWindow, withScreenshot: Bool) {
    guard var top = window.rootViewController else { return }
    while let presented = top.presentedViewController { top = presented }
    // Shake-to-undo raises its alert from the same shake.
    guard !(top is FeedbackHostingController), !(top is UIAlertController) else { return }
    let screenshot = withScreenshot ? window.feedbackScreenshot() : nil
    top.present(FeedbackHostingController(screenshot: screenshot), animated: true)
  }
}

private final class FeedbackHostingController: UIHostingController<FeedbackSheet> {
  init(screenshot: Data?) {
    super.init(rootView: FeedbackSheet(screenshot: screenshot))
  }

  @available(*, unavailable)
  required init?(coder: NSCoder) { nil }
}

extension UIWindow {
  fileprivate func feedbackScreenshot() -> Data? {
    UIGraphicsImageRenderer(bounds: bounds).image { _ in
      drawHierarchy(in: bounds, afterScreenUpdates: false)
    }.pngData()
  }

  // Overridden here because SwiftUI owns the window, so it cannot be subclassed.
  override open func motionEnded(_ motion: UIEvent.EventSubtype, with event: UIEvent?) {
    super.motionEnded(motion, with: event)
    guard motion == .motionShake else { return }
    // A turn later, so an undo alert raised by the same shake is already showing.
    Task { @MainActor in FeedbackPresenter.present(from: self, withScreenshot: true) }
  }
}
