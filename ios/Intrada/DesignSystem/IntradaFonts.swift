import CoreText
import Foundation
import os

/// Registers the bundled Source Serif 4 + Inter faces with the process font
/// manager (idempotent). The app calls this at launch; snapshot tests call it
/// too, since they render screens without launching the app to do it for them.
enum IntradaFonts {
  private final class BundleToken {}

  static func register() { _ = didRegister }

  private static let didRegister: Bool = {
    let bundle = Bundle(for: BundleToken.self)
    for face in ["InterVariable", "SourceSerif4Variable-Roman"] {
      guard let url = bundle.url(forResource: face, withExtension: "ttf") else {
        assertionFailure("Missing bundled font \(face).ttf")
        continue
      }
      var error: Unmanaged<CFError>?
      if !CTFontManagerRegisterFontsForURL(url as CFURL, .process, &error),
        let cfError = error?.takeRetainedValue()
      {
        Logger(subsystem: "com.intrada.native", category: "fonts")
          .error(
            "Font registration failed for \(face, privacy: .public): \(cfError.localizedDescription, privacy: .public)"
          )
      }
    }
    return true
  }()
}
