import SwiftUI

enum AppBanner: Equatable, Identifiable {
  case halted
  case storageUnavailable
  case error(String)
  case notice(String)

  static let storageMessage = "Storage unavailable · changes this session won't be saved."

  static func stack(halted: Bool, degraded: Bool, error: String?, notice: String?)
    -> [AppBanner]
  {
    var banners: [AppBanner] = []
    if halted { banners.append(.halted) }
    if degraded { banners.append(.storageUnavailable) }
    if let error { banners.append(.error(error)) }
    if let notice { banners.append(.notice(notice)) }
    return banners
  }

  var id: String {
    switch self {
    case .halted: "halted"
    case .storageUnavailable: "storage"
    case .error: "error"
    case .notice: "notice"
    }
  }
}

/// App-level banners below the status bar, shared by the tab shell and the
/// player cover so neither loses one the other shows (#2036).
struct AppBannerStack: View {
  @Environment(Store.self) private var store

  var body: some View {
    VStack(spacing: 0) {
      ForEach(banners) { banner in
        row(banner)
      }
    }
  }

  private var banners: [AppBanner] {
    AppBanner.stack(
      halted: store.halted, degraded: store.degraded,
      error: store.viewModel?.error, notice: store.viewModel?.notice)
  }

  @ViewBuilder
  private func row(_ banner: AppBanner) -> some View {
    switch banner {
    case .halted:
      GlobalBanner(message: Store.haltedMessage)
    case .storageUnavailable:
      GlobalBanner(message: AppBanner.storageMessage)
    case .error(let message):
      GlobalBanner(message: message) { store.send(.clearError) }
    case .notice(let message):
      GlobalBanner(message: message, tone: .notice) { store.send(.clearNotice) }
    }
  }
}
