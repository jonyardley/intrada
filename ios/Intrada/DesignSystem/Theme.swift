import SwiftUI

/// The locked "paper / score" light theme (Pencil: *Mobile / Library — Light*).
/// Every colour, gradient, and type style the native shell draws traces back to
/// a token here — views never hard-code a hex value. Source of truth mirrors the
/// Pencil variables; bundling the exact Source Serif 4 / Inter faces is a
/// follow-up (we stand in with the system serif + system sans for now).
enum IntradaColor {
  static let paperTop = Color(hex: 0xF4F1E8)
  static let paperBottom = Color(hex: 0xEBE7D9)

  static let cardFill = Color(hex: 0xFCFAF3)
  static let hairline = Color(hex: 0xE5DECD)
  static let divider = Color(hex: 0xE0D9C8)

  static let ink = Color(hex: 0x2B2A26)
  static let inkSecondary = Color(hex: 0x6E6557)
  static let inkFaint = Color(hex: 0x9A927F)

  static let accent = Color(hex: 0x4C3FA6)
  static let onAccent = Color(hex: 0xF2EFE8)
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

/// Semantic type styles. Headings use the system serif (New York) as a native
/// stand-in for Source Serif 4; body/meta use the system sans for Inter.
enum IntradaFont {
  static func pageTitle(_ size: CGFloat = 32) -> Font {
    .system(size: size, weight: .semibold, design: .serif)
  }
  static func cardTitle(_ size: CGFloat = 17) -> Font {
    .system(size: size, weight: .semibold, design: .serif)
  }
  static let body = Font.system(size: 14)
  static let meta = Font.system(size: 12)
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
