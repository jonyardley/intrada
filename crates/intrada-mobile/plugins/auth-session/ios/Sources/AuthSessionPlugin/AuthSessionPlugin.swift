import AuthenticationServices
import SwiftRs
import Tauri
import UIKit
import WebKit

struct OpenArgs: Decodable {
  let url: String
  let callback_scheme: String
}

class AuthSessionPlugin: Plugin {

  private var activeSession: ASWebAuthenticationSession?
  private var activeContext: AuthPresentationContext?

  @objc public func open(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(OpenArgs.self)

    guard let authUrl = URL(string: args.url) else {
      invoke.reject("auth-session: invalid URL")
      return
    }

    let scheme = args.callback_scheme

    DispatchQueue.main.async { [weak self] in
      guard let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
        let window = scene.windows.first(where: { $0.isKeyWindow })
      else {
        invoke.reject("auth-session: no key window")
        return
      }

      let contextProvider = AuthPresentationContext(anchor: window)
      self?.activeContext = contextProvider

      let session = ASWebAuthenticationSession(
        url: authUrl,
        callbackURLScheme: scheme
      ) { [weak self] callbackURL, error in
        self?.activeSession = nil
        self?.activeContext = nil

        if let error = error {
          let nsError = error as NSError
          if nsError.domain == ASWebAuthenticationSessionErrorDomain
            && nsError.code == ASWebAuthenticationSessionError.canceledLogin.rawValue
          {
            invoke.reject("auth-session: user cancelled")
          } else {
            invoke.reject("auth-session: \(error.localizedDescription)")
          }
          return
        }

        guard let callbackURL = callbackURL else {
          invoke.reject("auth-session: no callback URL")
          return
        }

        invoke.resolve(["callback_url": callbackURL.absoluteString])
      }

      session.presentationContextProvider = contextProvider
      // Ephemeral = true gives a clean cookie jar each time, so Google
      // always shows the account picker instead of auto-selecting the
      // previously used account from Safari's shared cookies.
      session.prefersEphemeralWebBrowserSession = true

      self?.activeSession = session

      if !session.start() {
        self?.activeSession = nil
        self?.activeContext = nil
        invoke.reject("auth-session: failed to start")
      }
    }
  }
}

private class AuthPresentationContext: NSObject, ASWebAuthenticationPresentationContextProviding {
  private let anchor: ASPresentationAnchor

  init(anchor: ASPresentationAnchor) {
    self.anchor = anchor
    super.init()
  }

  func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
    return anchor
  }
}

@_cdecl("init_plugin_auth_session")
func initPlugin() -> Plugin {
  return AuthSessionPlugin()
}
