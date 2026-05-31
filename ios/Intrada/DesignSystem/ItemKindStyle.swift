import SharedTypes
import SwiftUI

/// The type-language pairing — colour + glyph + label per `ItemKind` — defined
/// once here so every type-coded surface (card bars, badges, chips) stays in
/// sync. Piece = indigo + note; Exercise = gold + repeat.
extension ItemKind {
  var accent: Color {
    switch self {
    case .piece: IntradaColor.accent
    case .exercise: IntradaColor.exerciseAccent
    }
  }

  var bar: LinearGradient {
    switch self {
    case .piece: .brandBar
    case .exercise: .exerciseBar
    }
  }

  var iconName: String {
    switch self {
    case .piece: "music.note"
    case .exercise: "repeat"
    }
  }

  var label: String {
    switch self {
    case .piece: "Piece"
    case .exercise: "Exercise"
    }
  }
}
