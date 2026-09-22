import SharedTypes

/// The item-complete sheet's sends, in the order the core needs them (#1945).
enum ReflectionHandoff {
  struct Plan {
    /// Sent first with no status guard, so a refused note stops the hand-off before the entry moves on.
    let note: Event?
    /// Completes the entry; the scores and tempos after it need that.
    let nextItem: Event
    let after: [Event]
  }

  static func plan(
    entryId: String, now: String, nextItemStartedAt: String, reading: TempoReading,
    plays: [ReflectionPlay], result: ReflectionResult
  ) -> Plan {
    let note: Event? =
      result.note.isEmpty
      ? nil : .session(.updateEntryNotes(entryId: entryId, notes: result.note))
    let scores: [Event] = plays.compactMap { play in
      result.marks[play.id].map {
        .session(.updateEntryScore(entryId: entryId, playId: play.id, score: $0))
      }
    }
    // The click's tempo already landed on each play as it closed; this is the
    // manual path, and the core ignores a row nobody moved (#1761).
    let tempos: [Event] = result.tempos.map { row in
      .session(
        .updateEntryTempo(
          entryId: entryId, playId: row.playId, tempo: row.tempo, userSet: row.userSet,
          click: row.click))
    }
    return Plan(
      note: note,
      nextItem: .session(
        .nextItem(now: now, nextItemStartedAt: nextItemStartedAt, reading: reading)),
      after: scores + tempos)
  }

  /// Errors surface on PlayerHost's banner. False keeps the sheet up with its
  /// answers: a refused note or move leaves the entry current. Once it has
  /// moved on the sheet closes, or a retry would send NextItem twice (#1945).
  static func run(_ plan: Plan, send: (Event) -> Bool) -> Bool {
    if let note = plan.note, !send(note) { return false }
    if !send(plan.nextItem) { return false }
    for event in plan.after { _ = send(event) }
    return true
  }
}
