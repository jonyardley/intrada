import SharedTypes
import SwiftUI

/// The type-language pairing — colour + glyph + label per `ItemKind` — defined
/// once here so every type-coded surface (card bars, badges, chips) stays in
/// sync. Piece = indigo + note; Exercise = gold + dumbbell.
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
    case .exercise: "dumbbell.fill"
    }
  }

  var label: String {
    switch self {
    case .piece: "Piece"
    case .exercise: "Exercise"
    }
  }
}

/// The same type-language, extended to the compose sheet's third kind
/// (#1256, A2). `unresolved` wears no kind at all — a row that still owes a
/// question must not be tinted as though it were already something.
extension ComposeKind {
  var accent: Color {
    switch self {
    case .piece: IntradaColor.accent
    case .exercise: IntradaColor.exerciseAccent
    case .journal: IntradaColor.journalAccent
    case .unresolved: IntradaColor.inkFainter
    }
  }

  var iconName: String {
    switch self {
    case .piece: "music.note"
    case .exercise: "dumbbell.fill"
    case .journal: "note.text"
    case .unresolved: "questionmark.circle"
    }
  }

  var label: String {
    switch self {
    case .piece: "Piece"
    case .exercise: "Exercise"
    case .journal: "Journal"
    case .unresolved: "Needs a moment"
    }
  }

  var badgeBackground: Color {
    switch self {
    case .piece: IntradaColor.pieceBadgeBg
    case .exercise: IntradaColor.exerciseBadgeBg
    case .journal, .unresolved: IntradaColor.journalBadgeBg
    }
  }

  var badgeForeground: Color {
    switch self {
    case .piece: IntradaColor.pieceBadgeFg
    case .exercise: IntradaColor.exerciseBadgeFg
    case .journal, .unresolved: IntradaColor.journalBadgeFg
    }
  }
}
