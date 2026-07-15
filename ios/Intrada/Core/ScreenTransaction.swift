import Sentry
import SwiftUI

extension View {
  // `.onAppear` fires only for the visible tab, so this names the transaction after the
  // on-screen view — unlike SentryTracedView, whose root binds wrongly across a TabView's eager body eval (#912).
  func screenTransaction(_ name: String) -> some View {
    modifier(ScreenTransactionModifier(name: name))
  }
}

private struct ScreenTransactionModifier: ViewModifier {
  let name: String

  func body(content: Content) -> some View {
    content.onAppear {
      let transaction = SentrySDK.startTransaction(
        name: name, operation: "ui.load", bindToScope: false)
      DispatchQueue.main.async { transaction.finish() }
    }
  }
}
