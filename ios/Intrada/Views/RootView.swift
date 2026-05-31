import SharedTypes
import SwiftUI

/// The minimal foundation screen: sends `StartApp` on appear and renders the
/// core's `ViewModel`. Proves the bridge → store → SwiftUI render path. Real
/// library UI (and auth) come with the screen-by-screen rewrite (Phase C).
struct RootView: View {
  @Environment(Store.self) private var store

  private let apiBaseURL = "https://intrada-api.fly.dev"

  var body: some View {
    NavigationStack {
      Group {
        if let viewModel = store.viewModel {
          summary(viewModel)
        } else {
          ProgressView("Loading…")
        }
      }
      .navigationTitle("Intrada")
    }
    .task {
      store.send(.startApp(apiBaseUrl: apiBaseURL))
    }
  }

  private func summary(_ viewModel: ViewModel) -> some View {
    VStack(spacing: 16) {
      Image(systemName: "music.note.list")
        .font(.system(size: 48))
        .foregroundStyle(.tint)
      Text("\(viewModel.items.count) pieces")
        .font(.title2.weight(.semibold))
      if let error = viewModel.error {
        Text(error)
          .font(.footnote)
          .foregroundStyle(.secondary)
          .multilineTextAlignment(.center)
      }
    }
    .padding()
    .dynamicTypeSize(...DynamicTypeSize.accessibility3)
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Library — \(viewModel.items.count) pieces")
  }
}
