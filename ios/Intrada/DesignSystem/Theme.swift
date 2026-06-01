import SwiftUI

/// The locked "paper / score" light theme (Pencil: *Mobile / Library — Light*).
/// Every colour, gradient, and type style the native shell draws traces back to
/// a token here — views never hard-code a hex value or raw `.system` font.
enum IntradaColor {
  static let paperTop = Color(hex: 0xF4F1E8)
  static let paperBottom = Color(hex: 0xEBE7D9)

  static let cardFill = Color(hex: 0xFCFAF3)
  static let surfaceSunken = Color(hex: 0xF0E7D6)
  static let hairline = Color(hex: 0xE5DECD)
  static let divider = Color(hex: 0xE0D9C8)

  static let ink = Color(hex: 0x2B2A26)
  static let inkSecondary = Color(hex: 0x6E6557)
  static let inkFaint = Color(hex: 0x9A927F)

  static let accent = Color(hex: 0x4C3FA6)
  static let onAccent = Color(hex: 0xF2EFE8)
  static let danger = Color(hex: 0xB3261E)
  static let shadow = Color.black.opacity(0.06)
  static let brandGradientStart = Color(hex: 0x6346E5)
  static let brandGradientEnd = Color(hex: 0x4C3FA6)

  static let tabBarFill = Color(hex: 0xEFEBDF)
  static let tabBarInactiveIcon = Color(hex: 0xB6AEC4)

  static let pieceBadgeBg = Color(hex: 0xE7E3F4)
  static let pieceBadgeFg = Color(hex: 0x4C3FA6)
  static let exerciseAccent = Color(hex: 0x9E7B33)
  static let exerciseBadgeBg = Color(hex: 0xF0E5CC)
  static let exerciseBadgeFg = Color(hex: 0x8A6A2E)
}

extension LinearGradient {
  static let paper = LinearGradient(
    colors: [IntradaColor.paperTop, IntradaColor.paperBottom],
    startPoint: .top, endPoint: .bottom)

  static let brandBar = LinearGradient(
    colors: [IntradaColor.brandGradientStart, IntradaColor.brandGradientEnd],
    startPoint: .top, endPoint: .bottom)

  static let exerciseBar = LinearGradient(
    colors: [IntradaColor.exerciseAccent, IntradaColor.exerciseBadgeFg],
    startPoint: .top, endPoint: .bottom)
}

/// Semantic type styles: Source Serif 4 headings, Inter body/UI (bundled via
/// `IntradaFonts`). `relativeTo:` tracks Dynamic Type; weights use named-instance
/// PostScript names, not `.weight()`, which is synthetic over a variable axis.
enum IntradaFont {
  static func pageTitle(_ size: CGFloat = 32) -> Font {
    .custom(Serif.semibold, size: size, relativeTo: .largeTitle)
  }
  static func cardTitle(_ size: CGFloat = 17) -> Font {
    .custom(Serif.semibold, size: size, relativeTo: .title3)
  }

  static let body = Font.custom(Inter.regular, size: 14, relativeTo: .subheadline)
  static let bodyMedium = Font.custom(Inter.medium, size: 15, relativeTo: .subheadline)
  static let subtitle = Font.custom(Inter.regular, size: 13, relativeTo: .footnote)
  static let meta = Font.custom(Inter.regular, size: 12, relativeTo: .caption)
  static let micro = Font.custom(Inter.regular, size: 10, relativeTo: .caption2)
  static let metaMedium = Font.custom(Inter.medium, size: 12, relativeTo: .caption)
  static let badge = Font.custom(Inter.semibold, size: 12, relativeTo: .caption)
  static let tab = Font.custom(Inter.medium, size: 13, relativeTo: .footnote)
  static let segment = Font.custom(Inter.medium, size: 14, relativeTo: .subheadline)
  static let field = Font.custom(Inter.regular, size: 16, relativeTo: .callout)

  private enum Inter {
    static let regular = "InterVariable"
    static let medium = "InterVariable-Medium"
    static let semibold = "InterVariable-SemiBold"
  }

  private enum Serif {
    static let semibold = "SourceSerif4Variable-Semibold"
  }
}

extension Color {
  /// Build a `Color` from a packed `0xRRGGBB` literal so tokens read like the
  /// Pencil hex values they mirror.
  init(hex: UInt32) {
    let r = Double((hex >> 16) & 0xFF) / 255
    let g = Double((hex >> 8) & 0xFF) / 255
    let b = Double(hex & 0xFF) / 255
    self.init(.sRGB, red: r, green: g, blue: b, opacity: 1)
  }
}
