import SharedTypes

/// Shell-side presentation formatting for a library item. The core exposes
/// structured `tempoMarking` / `tempoBpm`; how iOS renders them ("Allegro · ♩ =
/// 132") is the shell's call, shared here so the card and detail agree.
extension LibraryItemView {
  var keyDisplay: String? {
    KeyHelper.display(key: key, modality: modality)
  }

  /// Visual tempo: "Allegro · ♩ = 132". ♩ is U+2669 (no SF Symbol equivalent).
  var tempoDisplay: String? {
    let parts = [tempoMarking, tempoBpm.map { "♩ = \($0)" }]
      .compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  /// Spoken tempo for VoiceOver — spells the BPM out instead of the ♩ glyph.
  var tempoSpoken: String? {
    let parts = [
      tempoMarking.flatMap { $0.isEmpty ? nil : $0 }, tempoBpm.map { "\($0) beats per minute" },
    ]
    .compactMap { $0 }
    return parts.isEmpty ? nil : parts.joined(separator: ", ")
  }
}
