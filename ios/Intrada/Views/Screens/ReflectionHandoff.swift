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

  /// False keeps the sheet up with its answers and the refusal shown in it:
  /// a refused note or move leaves the entry current. Once it has moved on
  /// the sheet closes, or a retry would send NextItem twice (#1945).
  static func run(_ plan: Plan, send: (Event) -> Bool) -> Bool {
    if let note = plan.note, !send(note) { return false }
    if !send(plan.nextItem) { return false }
    for event in plan.after { _ = send(event) }
    return true
  }

  /// What the core keeps for the open sheet (#2137): only the tempos set by
  /// hand, since the rest reseed from their stamps.
  static func draft(_ result: ReflectionResult, plays: [ReflectionPlay]) -> ReflectionAnswers {
    ReflectionAnswers(
      marks: plays.compactMap { play in
        result.marks[play.id].map { DraftMark(playId: play.id, score: $0) }
      },
      note: result.note,
      tempos: result.tempos.filter(\.userSet).map {
        DraftTempo(playId: $0.playId, tempo: $0.tempo, click: $0.click)
      }, felt: nil, gotInTheWay: [], notePoints: [], intentionMet: nil)
  }

  /// The saved draft as the sheet's starting answers, after a resume (#2137).
  static func seed(_ answers: ReflectionAnswers) -> ReflectionResult {
    ReflectionResult(
      marks: Dictionary(
        answers.marks.map { ($0.playId, $0.score) }, uniquingKeysWith: { first, _ in first }),
      note: answers.note,
      tempos: answers.tempos.map {
        ReflectionRowTempo(playId: $0.playId, tempo: $0.tempo, userSet: true, click: $0.click)
      })
  }

  /// An untouched click sat on the item's own bar, so a quaver bar's tempo is quavers (#2304).
  /// Only the sheet's rows carry it: `NextItem` keeps the untouched reading.
  static func sheetClick(_ reading: TempoReading, active: ActiveSessionView) -> ClickState {
    reading.click
      ?? ClickState(metre: active.clickSeedMetre, sounding: active.currentClickSounding)
  }

  /// A halted app has no core error to read, and a retry can never succeed (#2009).
  @MainActor static func refusalMessage(halted: Bool, error: String?) -> String {
    if halted { return Store.haltedMessage }
    return error ?? "Couldn't save. Try again."
  }
}
