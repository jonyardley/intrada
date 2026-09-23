#if DEBUG
  import Foundation
  import SharedTypes

  extension AnalyticsView {
    /// A deterministic analytics fixture for the Progress screen + snapshots.
    /// `scoreChanges` (this week's movers) drive the Recent-mastery rows;
    /// `weeklyMinutes` are the consistency bars (40/75/55/95/82).
    static var previewAnalytics: AnalyticsView {
      AnalyticsView(
        weeklySummary: WeeklySummary(
          totalMinutes: 380, sessionCount: 14, itemsCovered: 11,
          prevTotalMinutes: 300, prevSessionCount: 11, prevItemsCovered: 9,
          timeDirection: .up, sessionsDirection: .up, itemsDirection: .up,
          hasPrevWeekData: true),
        streak: PracticeStreak(currentDays: 4),
        topItems: [
          ItemRanking(
            itemId: "piece-1", itemTitle: "Clair de Lune", itemType: .piece,
            totalMinutes: 180, sessionCount: 9)
        ],
        neglectedItems: [],
        scoreChanges: [
          ScoreChange(
            itemId: "piece-1", itemTitle: "Clair de Lune", previousScore: 3,
            currentScore: 4, delta: 1, isNew: false),
          ScoreChange(
            itemId: "exercise-1", itemTitle: "Hanon No. 1", previousScore: 2,
            currentScore: 3, delta: 1, isNew: false),
          ScoreChange(
            itemId: "piece-2", itemTitle: "Gymnopédie No. 1", previousScore: nil,
            currentScore: 3, delta: 0, isNew: true),
        ],
        variationCoverage: [
          VariationCoverageView(
            itemId: "exercise-2", title: LibraryItemView.previewExerciseWithVariations.title,
            solid: 1, total: 3),
          VariationCoverageView(itemId: "exercise-3", title: "Chromatic run", solid: 4, total: 12),
        ],
        weeklyMinutes: [40, 75, 55, 95, 82],
        overallMastery: 3.4,
        topMover: ScoreChange(
          itemId: "exercise-1", itemTitle: "Hanon No. 1", previousScore: 2,
          currentScore: 3, delta: 1, isNew: false),
        masteryChange: "+1.0 this week")
    }
  }

  extension PracticeWeekView {
    /// The core's week of 25 May on `previewReferenceDate`: sessions Thursday
    /// and Saturday, opening on Saturday, the latest practice before today.
    static var previewWeek: PracticeWeekView {
      mayWeek(sessions: ["2026-05-28": ["session-2"], "2026-05-30": ["session-1"]], openingDay: 5)
    }

    static var previewEmptyWeek: PracticeWeekView { mayWeek(sessions: [:], openingDay: 6) }

    private static func mayWeek(sessions: [String: [String]], openingDay: UInt64)
      -> PracticeWeekView
    {
      let weekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]
      let days = weekdays.enumerated().map { index, weekday in
        let dayNumber = 25 + index
        let date = "2026-05-\(dayNumber)"
        let fullDate = "\(weekday) \(dayNumber) May"
        let heading = index == 6 ? "Today" : index == 5 ? "Yesterday" : fullDate
        return PracticeDayView(
          date: date, weekdayInitial: String(weekday.prefix(1)), dayNumber: UInt32(dayNumber),
          fullDate: fullDate, heading: heading, isToday: index == 6, isFuture: false,
          sessionIds: sessions[date] ?? [])
      }
      return PracticeWeekView(
        days: days, practisedDays: UInt64(sessions.count), openingDay: openingDay)
    }
  }

  extension SetlistEntryView {
    static var previewPiece: SetlistEntryView {
      building(id: "setlist-1", item: "piece-1", title: "Clair de Lune", type: .piece, position: 0)
    }

    static var previewExercise: SetlistEntryView {
      building(
        id: "setlist-2", item: "exercise-1", title: "Hanon No. 1", type: .exercise, position: 1)
    }

    static var previewGroupedScales: SetlistEntryView {
      building(id: "g-a", item: "ex-a", title: "Scales", type: .exercise, position: 0, group: "g1")
    }
    static var previewGroupedArpeggios: SetlistEntryView {
      building(
        id: "g-b", item: "ex-b", title: "Broken arpeggios", type: .exercise, position: 1,
        group: "g1")
    }
    static var previewGroupedPiece: SetlistEntryView {
      building(
        id: "g-p", item: "piece-1", title: "Clair de Lune", type: .piece, position: 2, group: "g1")
    }
    static var previewStandaloneExercise: SetlistEntryView {
      building(id: "g-s", item: "ex-c", title: "Sight-reading", type: .exercise, position: 3)
    }

    /// All three per-entry settings set — the "populated" `EntrySettingsSheet` snapshot.
    static var previewGroupedScalesConfigured: SetlistEntryView {
      var e = previewGroupedScales
      e.intention = "Even RH over the LH arpeggios"
      e.plannedRepTarget = 7
      e.plannedDurationSecs = 360
      e.plannedDurationDisplay = "6 min"
      return e
    }

    private static func building(
      id: String, item: String, title: String, type: ItemKind, position: UInt64,
      group: String? = nil
    ) -> SetlistEntryView {
      SetlistEntryView(
        id: id, itemId: item, itemTitle: title, itemType: type, position: position,
        // Escaped rather than the glyph: check-dashes.sh reads changed lines,
        // and an em dash is what the builder row has always shown here.
        durationDisplay: "\u{2014}", status: .notAttempted, notes: nil, intention: nil,
        plannedDurationSecs: nil, plannedDurationDisplay: nil, groupId: group,
        plannedVariationId: nil, plannedRepTarget: nil, plays: [], scoreSummary: nil)
    }
  }

  extension LastPractisedView {
    /// What the core projects for `previewCompleted` (Sat 30 May, headline
    /// Clair de Lune) read on `previewReferenceDate`, the Sunday after.
    static var previewYesterday: LastPractisedView {
      LastPractisedView(
        itemTitle: "Clair de Lune", relativeDay: "Yesterday",
        label: "Last practised yesterday")
    }
  }

  // Reason strings below are copies of the core's, per the table in
  // specs/up-next-card.md decision 6 — reword there and these go stale silently.
  extension SuggestedSession {
    /// The mock's case: starred and cold, one drill unmarked, one on a variation.
    static var previewStarred: SuggestedSession {
      SuggestedSession(
        pieceId: "piece-1", pieceTitle: "Like Someone in Love",
        pieceSubtitle: "Jimmy Van Heusen",
        reason: "A priority · cold for 5 weeks", priority: true,
        items: [
          SuggestedItem(
            itemId: "ex-1", itemTitle: "Guide tones", itemType: .exercise,
            variantId: nil, variantLabel: nil, latestScore: nil,
            reason: "Not marked with this piece"),
          SuggestedItem(
            itemId: "ex-2", itemTitle: "Shell voicings", itemType: .exercise,
            variantId: "variation-f", variantLabel: "F", latestScore: 4,
            reason: "Marked 4 of 10 last time"),
          SuggestedItem(
            itemId: "piece-1", itemTitle: "Like Someone in Love", itemType: .piece,
            variantId: nil, variantLabel: nil, latestScore: 6,
            reason: "Marked 6 of 10 last time"),
        ],
        estimatedMinutes: 15)
    }

    /// Unstarred and untouched — the longest copy the card has to fit.
    static var previewFresh: SuggestedSession {
      SuggestedSession(
        pieceId: "piece-2", pieceTitle: "Prelude in C", pieceSubtitle: nil,
        reason: "Not practised yet", priority: false,
        items: [
          SuggestedItem(
            itemId: "ex-3", itemTitle: "Contrary-motion scales", itemType: .exercise,
            variantId: nil, variantLabel: nil, latestScore: nil,
            reason: "Not marked with this piece"),
          SuggestedItem(
            itemId: "piece-2", itemTitle: "Prelude in C", itemType: .piece,
            variantId: nil, variantLabel: nil, latestScore: nil,
            reason: "Not marked yet"),
        ],
        estimatedMinutes: 10)
    }
  }

  extension PracticeSessionView {
    /// Sunday 31 May 2026 (noon UTC), the "today" `PracticeWeekView.previewWeek`
    /// is read on: the same week as the sessions below (Thursday 28th, Saturday 30th).
    static var previewReferenceDate: Date {
      var components = DateComponents()
      components.year = 2026
      components.month = 5
      components.day = 31
      components.hour = 12
      return PreviewCalendar.utc.date(from: components) ?? .distantPast
    }

    /// Fixed past dates (not "now") so the card renders a deterministic absolute
    /// date — reusable from snapshot tests as well as the canvas.
    static var previewCompleted: PracticeSessionView {
      PracticeSessionView(
        id: "session-1", startedAt: "2026-05-30T09:00:00Z", finishedAt: "2026-05-30T09:32:00Z",
        totalDurationDisplay: "32m 0s", totalDurationSummary: "32m",
        completionStatus: .completed,
        notes: "Left hand steadier once I slowed the middle section right down.",
        entries: [
          previewEntry(0, "Clair de Lune", .piece, score: 7, tempo: 66),
          previewEntry(
            1, "Gymnopédie No. 1", .piece, score: 8,
            notes: "Pedal changes cleaner than last week."),
          previewEntry(
            2, "Nocturne Op. 9 No. 2", .piece, score: 6, tempo: 54, repTarget: 5, repCount: 5),
        ],
        sessionScore: 7,
        playedSummary: "Clair de Lune · Gymnopédie No. 1 · Nocturne Op. 9 No. 2")
    }

    /// One exercise practised across three keys, so the detail screen lists a
    /// line per variation rather than only the last (#1739).
    static var previewWithVariations: PracticeSessionView {
      PracticeSessionView(
        id: "session-3", startedAt: "2026-05-29T10:00:00Z", finishedAt: "2026-05-29T10:20:00Z",
        totalDurationDisplay: "20m 30s", totalDurationSummary: "20m",
        completionStatus: .completed, notes: nil,
        entries: [SetlistEntryView.previewThreeVariations], sessionScore: 8,
        playedSummary: "Major Scales in C major, G major and D major")
    }

    /// One exercise practised in a single key, so the detail screen still
    /// names it rather than leaving it unattributed (#1785).
    static var previewWithOneVariation: PracticeSessionView {
      PracticeSessionView(
        id: "session-4", startedAt: "2026-05-27T09:00:00Z", finishedAt: "2026-05-27T09:06:00Z",
        totalDurationDisplay: "6m 0s", totalDurationSummary: "6m",
        completionStatus: .completed, notes: nil,
        entries: [SetlistEntryView.previewOneVariation], sessionScore: nil,
        playedSummary: "Arpeggios in E\u{266d} major")
    }

    /// No session mark, and the two statuses a detail view must state plainly
    /// rather than leave looking unmarked.
    static var previewEndedEarly: PracticeSessionView {
      PracticeSessionView(
        id: "session-2", startedAt: "2026-05-28T18:00:00Z", finishedAt: "2026-05-28T18:14:00Z",
        totalDurationDisplay: "14m 0s", totalDurationSummary: "14m",
        completionStatus: .endedEarly, notes: nil,
        entries: [
          previewEntry(0, "Hanon No. 1", .exercise, score: 5, repTarget: 10, repCount: 4),
          previewEntry(1, "Major Scales", .exercise, status: .notAttempted),
        ], sessionScore: nil, playedSummary: "Hanon No. 1")
    }

    private static func previewEntry(
      _ position: UInt64, _ title: String, _ type: ItemKind,
      status: EntryStatus = .completed, score: UInt8? = nil, tempo: UInt16? = nil,
      repTarget: UInt8? = nil, repCount: UInt8? = nil, notes: String? = nil
    ) -> SetlistEntryView {
      let plays =
        status == .completed
        ? [
          VariationPlayView(
            id: "entry-\(position)-p1", variationId: nil, variationLabel: nil, seconds: 600,
            durationDisplay: "10 min", repTarget: repTarget, repCount: repCount,
            repTargetReached: repTarget.map { repCount ?? 0 >= $0 }, repHistory: nil,
            achievedTempo: tempo, clickPattern: nil, tempoDisplay: tempo, score: score,
            isMarkable: true)
        ] : []
      return SetlistEntryView(
        id: "entry-\(position)", itemId: "item-\(position)", itemTitle: title, itemType: type,
        position: position, durationDisplay: "10 min", status: status, notes: notes,
        intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil, groupId: nil,
        plannedVariationId: nil, plannedRepTarget: nil, plays: plays,
        scoreSummary: plays.isEmpty ? nil : score)
    }
  }

  extension ActiveSessionView {
    /// Item start instant + the snapshot reference (start + 4:12) so the timer
    /// renders a fixed `04:12` deterministically.
    static let previewStartedAt = "2026-05-30T09:00:00Z"
    /// Deliberately before the item started, so the session timer and the item
    /// ring cannot both be right by reading the same field.
    static let previewSessionStartedAt = "2026-05-30T08:47:00Z"
    /// Past an hour, where `clockDisplay` switches to `H:MM:SS` and the reading
    /// outgrows the orientation slot. An hour at the piano is ordinary, not an
    /// edge case.
    static let previewLongSessionStartedAt = "2026-05-30T07:52:00Z"
    static var previewReferenceDate: Date {
      (SessionClock.parseRFC3339(previewStartedAt) ?? .distantPast).addingTimeInterval(252)
    }

    private static func previewEntry(
      _ position: UInt64, _ title: String, _ type: ItemKind, groupId: String? = nil
    )
      -> SetlistEntryView
    {
      SetlistEntryView(
        id: "entry-\(position)", itemId: "item-\(position)", itemTitle: title, itemType: type,
        position: position, durationDisplay: "10 min", status: .completed, notes: nil,
        intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil,
        groupId: groupId, plannedVariationId: nil, plannedRepTarget: nil,
        plays: [
          VariationPlayView(
            id: "entry-\(position)-p1", variationId: nil, variationLabel: nil, seconds: 600,
            durationDisplay: "10 min", repTarget: nil, repCount: nil, repTargetReached: nil,
            repHistory: nil, achievedTempo: nil, clickPattern: nil, tempoDisplay: nil, score: nil,
            isMarkable: true)
        ], scoreSummary: nil)
    }

    static var previewActive: ActiveSessionView {
      ActiveSessionView(
        currentItemTitle: "Clair de Lune", currentItemType: .piece,
        currentPosition: 1, totalItems: 5,
        startedAt: previewSessionStartedAt, currentItemStartedAt: previewStartedAt,
        entries: [
          previewEntry(0, "Hanon No. 1", .exercise),
          previewEntry(1, "Clair de Lune", .piece),
          previewEntry(2, "Major Scales", .exercise),
          previewEntry(3, "Gymnopédie No. 1", .piece),
          previewEntry(4, "Czerny Op. 299", .exercise),
        ],
        currentRepTarget: nil, currentRepCount: nil, currentRepTargetReached: nil,
        currentRepHistory: nil, currentRepSlots: 10,
        currentVariationId: nil, currentVariationLabel: nil,
        currentPlannedDurationSecs: 480,
        nextItemTitle: "Hanon No. 1",
        currentItemIntention: "Let the melody breathe", currentItemNotes: nil,
        currentRelatedPieceTitle: nil,
        currentItemTempoMarking: "Andante", currentItemTempoBpm: 66, currentItemMetre: nil,
        currentVariations: [])
    }

    /// The same session, run past an hour, so the `H:MM:SS` reading is drawn.
    static var previewActiveLongSession: ActiveSessionView {
      let base = previewActive
      return ActiveSessionView(
        currentItemTitle: base.currentItemTitle, currentItemType: base.currentItemType,
        currentPosition: base.currentPosition, totalItems: base.totalItems,
        startedAt: previewLongSessionStartedAt, currentItemStartedAt: base.currentItemStartedAt,
        entries: base.entries,
        currentRepTarget: base.currentRepTarget, currentRepCount: base.currentRepCount,
        currentRepTargetReached: base.currentRepTargetReached,
        currentRepHistory: base.currentRepHistory, currentRepSlots: 10,
        currentVariationId: base.currentVariationId,
        currentVariationLabel: base.currentVariationLabel,
        currentPlannedDurationSecs: base.currentPlannedDurationSecs,
        nextItemTitle: base.nextItemTitle, currentItemIntention: base.currentItemIntention,
        currentItemNotes: base.currentItemNotes,
        currentRelatedPieceTitle: base.currentRelatedPieceTitle,
        currentItemTempoMarking: base.currentItemTempoMarking,
        currentItemTempoBpm: base.currentItemTempoBpm, currentItemMetre: nil,
        currentVariations: [])
    }

    /// The current item is an exercise practised in C, now on G: the chip reads
    /// the open play and the picker switches it (#1739).
    static var previewActiveVariations: ActiveSessionView {
      ActiveSessionView(
        currentItemTitle: LibraryItemView.previewExerciseWithVariations.title,
        currentItemType: .exercise,
        currentPosition: 0, totalItems: 3,
        startedAt: previewSessionStartedAt, currentItemStartedAt: previewStartedAt,
        entries: [
          variationEntry(), previewEntry(1, "Clair de Lune", .piece),
          previewEntry(2, "Czerny Op. 299", .exercise),
        ],
        currentRepTarget: 10, currentRepCount: 4, currentRepTargetReached: false,
        currentRepHistory: nil, currentRepSlots: 10,
        currentVariationId: "variation-f", currentVariationLabel: "F",
        currentPlannedDurationSecs: 480,
        nextItemTitle: "Clair de Lune",
        currentItemIntention: "Even tone through the turn",
        currentItemNotes: nil,
        currentRelatedPieceTitle: nil,
        currentItemTempoMarking: nil, currentItemTempoBpm: 104, currentItemMetre: nil,
        currentVariations: [
          PickerVariationView(
            id: "variation-c", label: "C", caption: "Played this session · 3m 10s",
            isSolid: true),
          PickerVariationView(
            id: "variation-f", label: "F", caption: "Playing now", isSolid: false),
          PickerVariationView(
            id: "variation-bb", label: "B♭", caption: "Not yet played", isSolid: false),
        ])
    }

    /// The current entry of `previewActiveVariations`: one closed play in C and
    /// an open one in F, whose seconds the core has not stamped yet.
    private static func variationEntry() -> SetlistEntryView {
      SetlistEntryView(
        id: "entry-0", itemId: "exercise-2",
        itemTitle: LibraryItemView.previewExerciseWithVariations.title,
        itemType: .exercise, position: 0, durationDisplay: "10 min", status: .notAttempted,
        notes: nil, intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil,
        groupId: nil, plannedVariationId: "variation-c", plannedRepTarget: 10,
        plays: [
          VariationPlayView(
            id: "entry-0-p1", variationId: "variation-c", variationLabel: "C", seconds: 190,
            durationDisplay: "3m 10s", repTarget: 10, repCount: 10, repTargetReached: true,
            repHistory: nil, achievedTempo: nil, clickPattern: nil, tempoDisplay: nil, score: nil,
            isMarkable: true),
          VariationPlayView(
            id: "entry-0-p2", variationId: "variation-f", variationLabel: "F", seconds: 0,
            durationDisplay: "0s", repTarget: 10, repCount: 4, repTargetReached: false,
            repHistory: nil, achievedTempo: nil, clickPattern: nil, tempoDisplay: nil, score: nil,
            isMarkable: true),
        ], scoreSummary: nil)
    }

    static var previewActiveReps: ActiveSessionView {
      ActiveSessionView(
        currentItemTitle: "Hanon No. 1", currentItemType: .exercise,
        currentPosition: 2, totalItems: 5,
        startedAt: previewSessionStartedAt, currentItemStartedAt: previewStartedAt,
        entries: [
          previewEntry(0, "Warm-up Scales", .exercise),
          previewEntry(1, "Etude No. 3", .exercise),
          previewEntry(2, "Hanon No. 1", .exercise, groupId: "g1"),
          previewEntry(3, "Moonlight Sonata", .piece, groupId: "g1"),
          previewEntry(4, "Czerny Op. 299", .exercise),
        ],
        currentRepTarget: 10, currentRepCount: 7, currentRepTargetReached: false,
        currentRepHistory: nil, currentRepSlots: 10,
        currentVariationId: nil, currentVariationLabel: nil,
        currentPlannedDurationSecs: nil,
        nextItemTitle: "Czerny Op. 299",
        currentItemIntention: "Land each finger evenly",
        currentItemNotes: nil,
        currentRelatedPieceTitle: "Moonlight Sonata",
        currentItemTempoMarking: "Allegro", currentItemTempoBpm: 132, currentItemMetre: nil,
        currentVariations: [])
    }
  }

  extension SummaryView {
    /// One exercise across three keys beside a piece with a single play, so
    /// the screen shows both shapes at once (#1739).
    static var previewSummaryVariations: SummaryView {
      SummaryView(
        totalDurationDisplay: "20m 30s", completionStatus: .completed, notes: nil,
        entries: [
          SetlistEntryView.previewThreeVariations,
          summaryEntry("e2", "Clair de Lune", .piece, "7m 50s", 470, .completed, score: 6),
        ], sessionScore: nil)
    }

    static var previewSummary: SummaryView {
      SummaryView(
        totalDurationDisplay: "37m 50s", completionStatus: .completed, notes: nil,
        entries: [
          summaryEntry("e1", "Clair de Lune", .piece, "12m 40s", 760, .completed, score: 3),
          summaryEntry(
            "e2", "Hanon No. 1", .exercise, "8m 10s", 490, .completed, score: 4, tempo: 96,
            intention: "Land each finger evenly"),
          summaryEntry(
            "e3", "Gymnopédie No. 1", .piece, "11m 30s", 690, .completed, score: 5,
            notes: "Pedal changes cleaner than last week."),
          summaryEntry("e4", "Czerny Op. 299", .exercise, "5m 30s", 330, .completed, score: 3),
        ], sessionScore: 8)
    }

    static var previewSummaryEndedEarly: SummaryView {
      SummaryView(
        totalDurationDisplay: "20m 50s", completionStatus: .endedEarly, notes: nil,
        entries: [
          summaryEntry("e1", "Clair de Lune", .piece, "12m 40s", 760, .completed, score: 3),
          summaryEntry("e2", "Hanon No. 1", .exercise, "8m 10s", 490, .completed, score: 4),
          summaryEntry("e3", "Étude Op. 10", .piece, "0s", 0, .notAttempted, score: nil),
        ], sessionScore: 8)
    }

    private static func summaryEntry(
      _ id: String, _ title: String, _ type: ItemKind, _ duration: String, _ seconds: UInt64,
      _ status: EntryStatus, score: UInt8?, tempo: UInt16? = nil, intention: String? = nil,
      notes: String? = nil
    ) -> SetlistEntryView {
      let plays =
        status == .completed
        ? [
          VariationPlayView(
            id: "\(id)-p1", variationId: nil, variationLabel: nil, seconds: seconds,
            durationDisplay: duration, repTarget: nil, repCount: nil, repTargetReached: nil,
            repHistory: nil, achievedTempo: tempo, clickPattern: nil, tempoDisplay: tempo,
            score: score, isMarkable: true
          )
        ] : []
      return SetlistEntryView(
        id: id, itemId: id, itemTitle: title, itemType: type, position: 0,
        durationDisplay: duration, status: status, notes: notes, intention: intention,
        plannedDurationSecs: nil, plannedDurationDisplay: nil, groupId: nil,
        plannedVariationId: nil, plannedRepTarget: nil, plays: plays,
        scoreSummary: plays.isEmpty ? nil : score)
    }
  }

  extension ReflectionPlay {
    static func preview(
      _ id: String, _ label: String?, _ duration: String, _ repCount: UInt8? = nil,
      _ repTarget: UInt8? = nil, isMarkable: Bool = true, tempoDisplay: UInt16? = nil,
      clickPattern: ClickState? = nil
    ) -> ReflectionPlay {
      ReflectionPlay(
        id: id, variationLabel: label, durationDisplay: duration, repCount: repCount,
        repTarget: repTarget, isMarkable: isMarkable, tempoDisplay: tempoDisplay,
        clickPattern: clickPattern)
    }
  }

  extension VariationPlayView {
    /// Three variations of one item, as the item-complete sheet and the session
    /// detail read them (#1739).
    static func preview(
      _ id: String, _ variationLabel: String?, seconds: UInt64, duration: String,
      score: UInt8? = nil, tempo: UInt16? = nil, repCount: UInt8? = nil,
      repTarget: UInt8? = nil
    ) -> VariationPlayView {
      VariationPlayView(
        id: id, variationId: variationLabel.map { "v-\($0)" }, variationLabel: variationLabel,
        seconds: seconds, durationDisplay: duration, repTarget: repTarget, repCount: repCount,
        repTargetReached: repTarget.map { (repCount ?? 0) >= $0 }, repHistory: nil,
        achievedTempo: tempo, clickPattern: nil, tempoDisplay: tempo, score: score, isMarkable: true
      )
    }
  }

  extension SetlistEntryView {
    /// One exercise practised across three keys: the case #1739 exists for.
    static var previewThreeVariations: SetlistEntryView {
      let plays = [
        VariationPlayView.preview(
          "p1", "C major", seconds: 250, duration: "4m 10s", score: 7, repCount: 8, repTarget: 10),
        VariationPlayView.preview(
          "p2", "G major", seconds: 200, duration: "3m 20s", score: 9, tempo: 104, repCount: 10,
          repTarget: 10),
        VariationPlayView.preview(
          "p3", "D major", seconds: 310, duration: "5m 10s", repCount: 4, repTarget: 10),
      ]
      return SetlistEntryView(
        id: "entry-variations", itemId: "exercise-2", itemTitle: "Major Scales",
        itemType: .exercise, position: 0, durationDisplay: "12m 40s", status: .completed,
        notes: nil, intention: "Even tone through the turn", plannedDurationSecs: nil,
        plannedDurationDisplay: nil, groupId: nil, plannedVariationId: "v-C major",
        plannedRepTarget: 10, plays: plays, scoreSummary: 8)
    }

    /// One exercise practised in a single key: the detail screen names it
    /// rather than folding it silently into "type and time" (#1785).
    static var previewOneVariation: SetlistEntryView {
      let plays = [
        VariationPlayView.preview(
          "p1", "E\u{266d} major", seconds: 360, duration: "6m 0s", tempo: 96)
      ]
      return SetlistEntryView(
        id: "entry-one-variation", itemId: "exercise-3", itemTitle: "Arpeggios",
        itemType: .exercise, position: 0, durationDisplay: "6m 0s", status: .completed,
        notes: nil, intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil,
        groupId: nil, plannedVariationId: "v-E\u{266d} major", plannedRepTarget: nil,
        plays: plays, scoreSummary: nil)
    }
  }
#endif
