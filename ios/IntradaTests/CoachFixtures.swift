import SharedTypes

/// The one place a test coach record is spelt out in full. The generated types
/// have no memberwise default, so every field gets one here instead: a new Rust
/// field costs one edit rather than one per call site (#1256's `origin` cost
/// four, each found by a separate build).
enum CoachFixture {
  static func attempt(
    at: String = "2026-08-04T10:00:09Z",
    verdict: Verdict = .clean,
    source: EvidenceSource = .tapVerdict,
    cold: Bool = false,
    selfPredicted: Verdict? = nil,
    level: ParameterLevel = Self.level
  ) -> AttemptSummary {
    AttemptSummary(
      at: at, verdict: verdict, source: source, cold: cold, selfPredicted: selfPredicted,
      level: level)
  }

  /// Computed, not stored: the generated types are not `Sendable`.
  static var level: ParameterLevel { ParameterLevel(tempoBpm: 92, clickLevel: .twoAndFour) }

  static func blockRecord(
    id: String = "b1",
    node: String = "rootless-a-b",
    drill: String = "shell-voicings",
    gate: String = "rootless-under-melody",
    level: ParameterLevel = Self.level,
    circle: Circle = .hands,
    mode: Mode = .keys,
    startedAt: String = "2026-08-04T10:00:00Z",
    endedAt: String = "2026-08-04T10:00:30Z",
    attempts: [AttemptSummary] = [],
    attemptsToPass: UInt16? = 3,
    gateOpenedAtAttempt: UInt16? = 3,
    repsAfterGate: UInt16 = 0,
    activeMs: UInt64 = 30_000,
    escalationFired: [Rung] = [],
    exit: Exit = .gatePassed,
    origin: BlockOrigin = .authored
  ) -> BlockRecord {
    BlockRecord(
      id: id, node: node, drill: drill, gate: gate, level: level, circle: circle, mode: mode,
      startedAt: startedAt, endedAt: endedAt, attempts: attempts,
      attemptsToPass: attemptsToPass, gateOpenedAtAttempt: gateOpenedAtAttempt,
      repsAfterGate: repsAfterGate, activeMs: activeMs, escalationFired: escalationFired,
      exit: exit, origin: origin)
  }
}
