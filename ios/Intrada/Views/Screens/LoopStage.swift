import SharedTypes

/// What the drill loop's cover is showing. The order is the rule: a question
/// the core has set outranks the block behind it, the landing is the last thing
/// a session leaves (#1323), and `idle` — the only stage the cover may close on
/// — is what every exit path asks, because the block a session ends by closing
/// can still owe both questions and a landing.
enum LoopStage: Equatable {
  case feel(FeelPrompt)
  case drill(DrillView)
  case reflection
  case landing(LandingView)
  case idle

  static func of(drill: DrillView?, feel: FeelPrompt?, reflection: Bool, landing: LandingView?)
    -> LoopStage
  {
    if let feel { return .feel(feel) }
    // The next block opens on its silent entry card, which bills nothing while
    // a question is up.
    if let drill { return .drill(drill) }
    if reflection { return .reflection }
    // Last, so the questions come before the acknowledgement rather than after
    // it: "how did that feel" reads oddly once the app has said well done.
    if let landing { return .landing(landing) }
    return .idle
  }

  var isIdle: Bool { self == .idle }
}
