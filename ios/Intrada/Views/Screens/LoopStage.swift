import SharedTypes

/// What the drill loop's cover is showing. The order is the rule: a question
/// the core has set outranks the block behind it, and `idle` — the only stage
/// the cover may close on — is what every exit path asks, because the block a
/// session ends by closing can still owe both questions.
enum LoopStage: Equatable {
  case feel(FeelPrompt)
  case drill(DrillView)
  case reflection
  case idle

  static func of(drill: DrillView?, feel: FeelPrompt?, reflection: Bool) -> LoopStage {
    if let feel { return .feel(feel) }
    // The next block opens on its silent entry card, which bills nothing while
    // a question is up.
    if let drill { return .drill(drill) }
    if reflection { return .reflection }
    return .idle
  }

  var isIdle: Bool { self == .idle }
}
