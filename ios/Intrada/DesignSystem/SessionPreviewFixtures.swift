#if DEBUG
  import Foundation
  import SharedTypes

  extension EntryRecordView {
    static var empty: EntryRecordView {
      EntryRecordView(
        segments: [], canAddSection: true, focus: nil, suggestedFocus: nil, intentionMet: nil,
        intentionMetRead: false,
        felt: nil, gotInTheWay: [], notePoints: [])
    }
  }

  extension ActiveRecordView {
    static var empty: ActiveRecordView {
      ActiveRecordView(segment: nil, awayOffer: nil, finish: nil, lastTime: nil)
    }
  }

  extension SectionView {
    static var previewClairSections: [SectionView] {
      [
        ("sec-a1", "A1", 1, 14, 8), ("sec-b", "B", 15, 26, 5), ("sec-a2", "A2", 27, 42, nil),
      ].map { (id: String, name: String, first: Int, last: Int, mark: UInt8?) in
        SectionView(
          id: id, name: name, kind: .form, targetBpm: nil, firstBar: UInt16(first),
          lastBar: UInt16(last), label: name, barsCaption: "Bars \(first) to \(last)",
          barsFieldText: "\(first) to \(last)",
          latestScore: mark, scoreHistory: [],
          caption: mark.map { "\($0) of 10" } ?? "Not yet played", isWeakest: id == "sec-b")
      }
    }
  }

  extension FocusChoiceView {
    static var previewChoices: [FocusChoiceView] {
      [
        FocusChoiceView(
          kind: .tempo, label: "Tempo", target: FocusTargetView(min: 40, max: 208, step: 2)),
        FocusChoiceView(
          kind: .cleanReps, label: "Clean in a row",
          target: FocusTargetView(min: 1, max: 100, step: 1)),
        FocusChoiceView(kind: .fromMemory, label: "From memory", target: nil),
        FocusChoiceView(kind: .evenness, label: "Evenness", target: nil),
      ]
    }
  }

  extension LimitsView {
    /// The core's own answer, so a preview or test never carries a band or bar
    /// table of its own that could drift from it (#2225).
    static var preview: LimitsView {
      let bridge = LiveBridge()
      guard let limits = try? bridge.view().limits else {
        preconditionFailure("the core must render its limits")
      }
      return limits
    }
  }

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
        masteryChange: "+1.0 this week",
        climbing: "Climbing steadily across 2 items.",
        pooledVariations: [
          PooledMarkView(
            label: "Dotted rhythms", solid: 2, total: 5, caption: "Solid on 2 of 5 items"),
          PooledMarkView(
            label: "Hands separately", solid: 1, total: 3, caption: "Solid on 1 of 3 items"),
        ],
        pooledKeys: [
          PooledMarkView(
            label: "E\u{266D} major", solid: 1, total: 2, caption: "Solid on 1 of 2 items")
        ])
    }
  }

  extension PracticeWeekView {
    /// The core's week of 25 May on `previewReferenceDate`: sessions Thursday
    /// and Saturday, opening on Saturday, the latest practice before today.
    static var previewWeek: PracticeWeekView {
      mayWeek(sessions: ["2026-05-28": ["session-2"], "2026-05-30": ["session-1"]], openingDay: 5)
    }

    static var previewEmptyWeek: PracticeWeekView { mayWeek(sessions: [:], openingDay: 6) }

    /// Read on Thursday 28 May, so Friday to Sunday are still to come (#2144).
    /// `session-4` stands in for a practised Wednesday; its own day label is not read here.
    static var previewMidWeek: PracticeWeekView {
      mayWeek(
        sessions: ["2026-05-27": ["session-4"]], openingDay: 2, today: 3)
    }

    private static func mayWeek(
      sessions: [String: [String]], openingDay: UInt64, today: Int = 6
    ) -> PracticeWeekView {
      let weekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]
      let days = weekdays.enumerated().map { index, weekday in
        let dayNumber = 25 + index
        let date = "2026-05-\(dayNumber)"
        let fullDate = "\(weekday) \(dayNumber) May"
        let heading = index == today ? "Today" : index == today - 1 ? "Yesterday" : fullDate
        return PracticeDayView(
          date: date, weekdayInitial: String(weekday.prefix(1)), dayNumber: UInt32(dayNumber),
          fullDate: fullDate, heading: heading, isToday: index == today, isFuture: index > today,
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
        id: "g-p", item: "piece-1", title: "Clair de Lune", type: .piece, position: 2, group: "g1",
        removable: false)
    }
    static var previewStandaloneExercise: SetlistEntryView {
      building(id: "g-s", item: "ex-c", title: "Sight-reading", type: .exercise, position: 3)
    }

    /// Twelve minutes split into A1, B and A2 with a tempo focus on A1 (#2315, #2303).
    static var previewPlannedPiece: SetlistEntryView {
      var e = building(
        id: "plan-p", item: "piece-1", title: "Clair de Lune", type: .piece, position: 0)
      e.intention = "Get A1 up to 84 with the click"
      e.plannedDurationSecs = 720
      e.plannedDurationDisplay = "12 min"
      e.record.segments = SectionView.previewClairSections.map {
        SegmentView(
          sectionId: $0.id, label: $0.label, plannedSecs: 240, plannedDisplay: "4 min",
          canAddMinute: true, canTakeMinute: true)
      }
      e.record.focus = FocusView(
        focus: IntentionFocus(kind: .tempo, sectionId: "sec-a1", target: 84), label: "A1 at 84",
        targetCaption: "\u{2669} = 84")
      return e
    }

    /// Nothing planned yet, with a typed aim the core reads a focus from.
    static var previewSuggestingPiece: SetlistEntryView {
      var e = previewPlannedPiece
      e.record = .empty
      e.record.suggestedFocus = previewPlannedPiece.record.focus
      return e
    }

    static var previewLastTimePiece: SetlistEntryView {
      var e = building(
        id: "plan-i", item: "piece-2", title: "Invention No. 8", type: .piece, position: 1)
      e.plannedDurationSecs = 600
      e.plannedDurationDisplay = "10 min"
      return e
    }

    /// All three per-entry settings set: the "populated" `EntrySettingsSheet` snapshot.
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
      group: String? = nil, removable: Bool = true
    ) -> SetlistEntryView {
      SetlistEntryView(
        id: id, itemId: item, itemTitle: title, itemType: type, position: position,
        // Escaped rather than the glyph: check-dashes.sh reads changed lines,
        // and an em dash is what the builder row has always shown here.
        durationDisplay: "\u{2014}", status: .notAttempted, notes: nil, intention: nil,
        plannedDurationSecs: nil, plannedDurationDisplay: nil, groupId: group, removable: removable,
        plannedSectionIds: [], plannedVariationIds: [], plannedLabel: nil, plannedRepTarget: nil,
        plays: [], scoreSummary: nil, record: .empty, plannedKey: nil)
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
  // specs/up-next-card.md decision 6: reword there and these go stale silently.
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
            latestScore: nil,
            reason: "Not marked with this piece", weakestSection: nil),
          SuggestedItem(
            itemId: "ex-2", itemTitle: "Shell voicings", itemType: .exercise,
            latestScore: 4,
            reason: "Marked 4 of 10 last time", weakestSection: nil),
          SuggestedItem(
            itemId: "piece-1", itemTitle: "Like Someone in Love", itemType: .piece,
            latestScore: 6,
            reason: "Marked 6 of 10 last time",
            weakestSection: "Weakest section · Bridge"),
        ],
        estimatedMinutes: 15)
    }

    /// Unstarred and untouched: the longest copy the card has to fit.
    static var previewFresh: SuggestedSession {
      SuggestedSession(
        pieceId: "piece-2", pieceTitle: "Prelude in C", pieceSubtitle: nil,
        reason: "Not practised yet", priority: false,
        items: [
          SuggestedItem(
            itemId: "ex-3", itemTitle: "Contrary-motion scales", itemType: .exercise,
            latestScore: nil,
            reason: "Not marked with this piece", weakestSection: nil),
          SuggestedItem(
            itemId: "piece-2", itemTitle: "Prelude in C", itemType: .piece,
            latestScore: nil,
            reason: "Not marked yet", weakestSection: nil),
        ],
        estimatedMinutes: 10)
    }
  }

  extension SuggestedSession {
    /// A later block in a filled plan, which the hero shows as one line.
    static var previewAutumnLeaves: SuggestedSession {
      SuggestedSession(
        pieceId: "piece-3", pieceTitle: "Autumn Leaves", pieceSubtitle: "Joseph Kosma",
        reason: "Cold for 3 weeks", priority: false,
        items: [
          SuggestedItem(
            itemId: "ex-4", itemTitle: "Two-five-one in G minor", itemType: .exercise,
            latestScore: 7,
            reason: "Marked 7 of 10 last time", weakestSection: nil),
          SuggestedItem(
            itemId: "piece-3", itemTitle: "Autumn Leaves", itemType: .piece,
            latestScore: 5,
            reason: "Marked 5 of 10 last time", weakestSection: nil),
        ],
        estimatedMinutes: 10)
    }
  }

  extension SuggestedPlan {
    /// A plan of one block, as the hero shows with no preferred length.
    static var previewStarred: SuggestedPlan {
      let block = SuggestedSession.previewStarred
      return SuggestedPlan(
        blocks: [block], estimatedMinutes: block.estimatedMinutes,
        itemCount: UInt32(block.items.count), lengthMins: nil)
    }

    /// One block, no star, never marked: the longest copy the card has to fit.
    static var previewFresh: SuggestedPlan {
      let block = SuggestedSession.previewFresh
      return SuggestedPlan(
        blocks: [block], estimatedMinutes: block.estimatedMinutes,
        itemCount: UInt32(block.items.count), lengthMins: nil)
    }

    /// Filled to a 30 minute preference: the lead block and two more.
    static var previewFilled: SuggestedPlan {
      let blocks = [SuggestedSession.previewStarred, .previewFresh, .previewAutumnLeaves]
      return SuggestedPlan(
        blocks: blocks, estimatedMinutes: blocks.reduce(0) { $0 + $1.estimatedMinutes },
        itemCount: UInt32(blocks.reduce(0) { $0 + $1.items.count }), lengthMins: 30)
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

    /// `dayLabel` matches `previewWeek`'s heading, read with 31 May as today.
    static var previewCompleted: PracticeSessionView {
      PracticeSessionView(
        id: "session-1",
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
        playedSummary: "Clair de Lune · Gymnopédie No. 1 · Nocturne Op. 9 No. 2",
        dayLabel: "Yesterday")
    }

    /// One exercise practised across three keys, so the detail screen lists a
    /// line per variation rather than only the last (#1739).
    static var previewWithVariations: PracticeSessionView {
      PracticeSessionView(
        id: "session-3",
        totalDurationDisplay: "20m 30s", totalDurationSummary: "20m",
        completionStatus: .completed, notes: nil,
        entries: [SetlistEntryView.previewThreeVariations], sessionScore: 8,
        playedSummary: "Major Scales in C major, G major and D major",
        dayLabel: "Fri 29 May")
    }

    /// One exercise practised in a single key, so the detail screen still
    /// names it rather than leaving it unattributed (#1785).
    static var previewWithOneVariation: PracticeSessionView {
      PracticeSessionView(
        id: "session-4",
        totalDurationDisplay: "6m 0s", totalDurationSummary: "6m",
        completionStatus: .completed, notes: nil,
        entries: [SetlistEntryView.previewOneVariation], sessionScore: nil,
        playedSummary: "Arpeggios in E\u{266d} major",
        dayLabel: "Wed 27 May")
    }

    /// No session mark, and the two statuses a detail view must state plainly
    /// rather than leave looking unmarked.
    static var previewEndedEarly: PracticeSessionView {
      PracticeSessionView(
        id: "session-2",
        totalDurationDisplay: "14m 0s", totalDurationSummary: "14m",
        completionStatus: .endedEarly, notes: nil,
        entries: [
          previewEntry(0, "Hanon No. 1", .exercise, score: 5, repTarget: 10, repCount: 4),
          previewEntry(1, "Major Scales", .exercise, status: .notAttempted),
        ], sessionScore: nil, playedSummary: "Hanon No. 1",
        dayLabel: "Thu 28 May")
    }

    private static func previewEntry(
      _ position: UInt64, _ title: String, _ type: ItemKind,
      status: EntryStatus = .completed, score: UInt8? = nil, tempo: UInt16? = nil,
      repTarget: UInt8? = nil, repCount: UInt8? = nil, notes: String? = nil
    ) -> SetlistEntryView {
      let plays =
        status == .completed
        ? [
          PlayView(
            id: "entry-\(position)-p1", sectionId: nil, key: nil, variationIds: [], label: nil,
            seconds: 600,
            durationDisplay: "10 min", repTarget: repTarget, repCount: repCount,
            repTargetReached: repTarget.map { repCount ?? 0 >= $0 }, repHistory: nil,
            achievedTempo: tempo, clickPattern: nil, tempoDisplay: tempo, score: score,
            isMarkable: true)
        ] : []
      return SetlistEntryView(
        id: "entry-\(position)", itemId: "item-\(position)", itemTitle: title, itemType: type,
        position: position, durationDisplay: "10 min", status: status, notes: notes,
        intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil, groupId: nil,
        removable: true,
        plannedSectionIds: [], plannedVariationIds: [], plannedLabel: nil, plannedRepTarget: nil,
        plays: plays,
        scoreSummary: plays.isEmpty ? nil : score, record: .empty, plannedKey: nil)
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
      _ position: UInt64, _ title: String, _ type: ItemKind, groupId: String? = nil,
      removable: Bool = true
    )
      -> SetlistEntryView
    {
      SetlistEntryView(
        id: "entry-\(position)", itemId: "item-\(position)", itemTitle: title, itemType: type,
        position: position, durationDisplay: "10 min", status: .completed, notes: nil,
        intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil,
        groupId: groupId, removable: removable, plannedSectionIds: [], plannedVariationIds: [],
        plannedLabel: nil,
        plannedRepTarget: nil,
        plays: [
          PlayView(
            id: "entry-\(position)-p1", sectionId: nil, key: nil, variationIds: [], label: nil,
            seconds: 600,
            durationDisplay: "10 min", repTarget: nil, repCount: nil, repTargetReached: nil,
            repHistory: nil, achievedTempo: nil, clickPattern: nil, tempoDisplay: nil, score: nil,
            isMarkable: true)
        ], scoreSummary: nil, record: .empty, plannedKey: nil)
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
        currentSectionId: nil, currentKey: nil, currentVariationIds: [], currentPlayLabel: nil,
        currentPlannedDurationSecs: 480,
        nextItemTitle: "Hanon No. 1",
        currentItemIntention: "Let the melody breathe", currentItemNotes: nil,
        currentRelatedPieceTitle: nil,
        currentItemTempoMarking: "Andante", currentItemTempoBpm: 66,
        currentItemTempoLine: "Andante · ♩ = 66",
        currentItemTempoLineSpoken: "Andante, 66 beats per minute",
        currentClickSounding: 0b1111,
        currentVariations: [], reflection: nil,
        clickSeedMetre: Metre(beats: 4, unit: 4, groups: nil), clickSeedBpm: 66,
        clickSeedSoundsTarget: true,
        clickSeedPresets: LimitsView.preview.clickPresets(
          for: Metre(beats: 4, unit: 4, groups: nil)),
        currentRepsPastTarget: 0, currentCanUndo: false, record: .empty)
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
        currentSectionId: base.currentSectionId, currentKey: base.currentKey,
        currentVariationIds: base.currentVariationIds, currentPlayLabel: base.currentPlayLabel,
        currentPlannedDurationSecs: base.currentPlannedDurationSecs,
        nextItemTitle: base.nextItemTitle, currentItemIntention: base.currentItemIntention,
        currentItemNotes: base.currentItemNotes,
        currentRelatedPieceTitle: base.currentRelatedPieceTitle,
        currentItemTempoMarking: base.currentItemTempoMarking,
        currentItemTempoBpm: base.currentItemTempoBpm,
        currentItemTempoLine: base.currentItemTempoLine,
        currentItemTempoLineSpoken: base.currentItemTempoLineSpoken,
        currentClickSounding: 0b1111,
        currentVariations: [], reflection: nil,
        clickSeedMetre: base.clickSeedMetre, clickSeedBpm: base.clickSeedBpm,
        clickSeedSoundsTarget: base.clickSeedSoundsTarget, clickSeedPresets: base.clickSeedPresets,
        currentRepsPastTarget: base.currentRepsPastTarget, currentCanUndo: base.currentCanUndo,
        record: .empty)
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
        currentSectionId: nil, currentKey: nil, currentVariationIds: ["variation-f"],
        currentPlayLabel: "F",
        currentPlannedDurationSecs: 480,
        nextItemTitle: "Clair de Lune",
        currentItemIntention: "Even tone through the turn",
        currentItemNotes: nil,
        currentRelatedPieceTitle: nil,
        currentItemTempoMarking: nil, currentItemTempoBpm: 104, currentItemTempoLine: "♩ = 104",
        currentItemTempoLineSpoken: "104 beats per minute",
        currentClickSounding: 0b1111,
        currentVariations: [
          PickerVariationView(
            id: "variation-c", label: "C", caption: "Played this session · 3m 10s"),
          PickerVariationView(
            id: "variation-f", label: "F", caption: "Playing now"),
          PickerVariationView(
            id: "variation-bb", label: "B♭", caption: "Not yet played"),
        ],
        reflection: nil,
        clickSeedMetre: Metre(beats: 4, unit: 4, groups: nil), clickSeedBpm: 104,
        clickSeedSoundsTarget: true,
        clickSeedPresets: LimitsView.preview.clickPresets(
          for: Metre(beats: 4, unit: 4, groups: nil)),
        currentRepsPastTarget: 0, currentCanUndo: false, record: .empty)
    }

    /// The current entry of `previewActiveVariations`: one closed play in C and
    /// an open one in F, whose seconds the core has not stamped yet.
    private static func variationEntry() -> SetlistEntryView {
      SetlistEntryView(
        id: "entry-0", itemId: "exercise-2",
        itemTitle: LibraryItemView.previewExerciseWithVariations.title,
        itemType: .exercise, position: 0, durationDisplay: "10 min", status: .notAttempted,
        notes: nil, intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil,
        groupId: nil, removable: true, plannedSectionIds: [], plannedVariationIds: ["variation-c"],
        plannedLabel: "C", plannedRepTarget: 10,
        plays: [
          PlayView(
            id: "entry-0-p1", sectionId: nil, key: nil, variationIds: ["variation-c"], label: "C",
            seconds: 190,
            durationDisplay: "3m 10s", repTarget: 10, repCount: 10, repTargetReached: true,
            repHistory: nil, achievedTempo: nil, clickPattern: nil, tempoDisplay: nil, score: nil,
            isMarkable: true),
          PlayView(
            id: "entry-0-p2", sectionId: nil, key: nil, variationIds: ["variation-f"], label: "F",
            seconds: 0,
            durationDisplay: "0s", repTarget: 10, repCount: 4, repTargetReached: false,
            repHistory: nil, achievedTempo: nil, clickPattern: nil, tempoDisplay: nil, score: nil,
            isMarkable: true),
        ], scoreSummary: nil, record: .empty, plannedKey: nil)
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
          previewEntry(3, "Moonlight Sonata", .piece, groupId: "g1", removable: false),
          previewEntry(4, "Czerny Op. 299", .exercise),
        ],
        currentRepTarget: 10, currentRepCount: 7, currentRepTargetReached: false,
        currentRepHistory: nil, currentRepSlots: 10,
        currentSectionId: nil, currentKey: nil, currentVariationIds: [], currentPlayLabel: nil,
        currentPlannedDurationSecs: nil,
        nextItemTitle: "Czerny Op. 299",
        currentItemIntention: "Land each finger evenly",
        currentItemNotes: nil,
        currentRelatedPieceTitle: "Moonlight Sonata",
        currentItemTempoMarking: "Allegro", currentItemTempoBpm: 132,
        currentItemTempoLine: "Allegro · ♩ = 132",
        currentItemTempoLineSpoken: "Allegro, 132 beats per minute",
        currentClickSounding: 0b1111,
        currentVariations: [], reflection: nil,
        clickSeedMetre: Metre(beats: 4, unit: 4, groups: nil), clickSeedBpm: 132,
        clickSeedSoundsTarget: true,
        clickSeedPresets: LimitsView.preview.clickPresets(
          for: Metre(beats: 4, unit: 4, groups: nil)),
        currentRepsPastTarget: 0, currentCanUndo: false, record: .empty)
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
        ], sessionScore: nil, completedCount: 2, topMover: nil)
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
        ], sessionScore: 8, completedCount: 4,
        topMover: ScoreChange(
          itemId: "piece-1", itemTitle: "Clair de Lune", previousScore: 3, currentScore: 4,
          delta: 1, isNew: false))
    }

    static var previewSummaryEndedEarly: SummaryView {
      SummaryView(
        totalDurationDisplay: "20m 50s", completionStatus: .endedEarly, notes: nil,
        entries: [
          summaryEntry("e1", "Clair de Lune", .piece, "12m 40s", 760, .completed, score: 3),
          summaryEntry("e2", "Hanon No. 1", .exercise, "8m 10s", 490, .completed, score: 4),
          summaryEntry("e3", "Étude Op. 10", .piece, "0s", 0, .notAttempted, score: nil),
        ], sessionScore: 8, completedCount: 2, topMover: nil)
    }

    private static func summaryEntry(
      _ id: String, _ title: String, _ type: ItemKind, _ duration: String, _ seconds: UInt64,
      _ status: EntryStatus, score: UInt8?, tempo: UInt16? = nil, intention: String? = nil,
      notes: String? = nil
    ) -> SetlistEntryView {
      let plays =
        status == .completed
        ? [
          PlayView(
            id: "\(id)-p1", sectionId: nil, key: nil, variationIds: [], label: nil,
            seconds: seconds,
            durationDisplay: duration, repTarget: nil, repCount: nil, repTargetReached: nil,
            repHistory: nil, achievedTempo: tempo, clickPattern: nil, tempoDisplay: tempo,
            score: score, isMarkable: true
          )
        ] : []
      return SetlistEntryView(
        id: id, itemId: id, itemTitle: title, itemType: type, position: 0,
        durationDisplay: duration, status: status, notes: notes, intention: intention,
        plannedDurationSecs: nil, plannedDurationDisplay: nil, groupId: nil, removable: true,
        plannedSectionIds: [], plannedVariationIds: [], plannedLabel: nil, plannedRepTarget: nil,
        plays: plays,
        scoreSummary: plays.isEmpty ? nil : score, record: .empty, plannedKey: nil)
    }
  }

  extension ReflectionPlay {
    static func preview(
      _ id: String, _ label: String?, _ duration: String, _ repCount: UInt8? = nil,
      _ repTarget: UInt8? = nil, isMarkable: Bool = true
    ) -> ReflectionPlay {
      ReflectionPlay(
        id: id, variationLabel: label, durationDisplay: duration, repCount: repCount,
        repTarget: repTarget, isMarkable: isMarkable)
    }
  }

  extension ReflectionTempoView {
    /// Unstamped crotchets at the click's default unless given a stamp.
    static func preview(
      _ playId: String, tempo: UInt16? = nil,
      click: ClickState = ClickState(metre: Metre(beats: 4, unit: 4, groups: nil), sounding: 0b1111)
    ) -> ReflectionTempoView {
      let limits = LimitsView.preview
      let range = limits.clickBand(unit: click.metre.unit)
      let band = TempoBand(
        unit: click.metre.unit, min: UInt16(range.lowerBound), max: UInt16(range.upperBound))
      return ReflectionTempoView(
        playId: playId, tempo: tempo ?? limits.clickTempoDefault, click: click, band: band,
        setByHand: false)
    }
  }

  extension ReflectionAnswers {
    static func preview(
      marks: [DraftMark] = [], note: String = "", tempos: [DraftTempo] = [], felt: Felt? = nil,
      gotInTheWay: [Obstacle] = [], notePoints: [NoteSpan] = [],
      intentionMet: IntentionMet? = nil, ways: [DraftWay] = []
    ) -> ReflectionAnswers {
      ReflectionAnswers(
        marks: marks, note: note, tempos: tempos, felt: felt, gotInTheWay: gotInTheWay,
        notePoints: notePoints, intentionMet: intentionMet, ways: ways)
    }
  }

  extension PlayView {
    /// Three variations of one item, as the item-complete sheet and the session
    /// detail read them (#1739).
    static func preview(
      _ id: String, _ variationLabel: String?, seconds: UInt64, duration: String,
      score: UInt8? = nil, tempo: UInt16? = nil, repCount: UInt8? = nil,
      repTarget: UInt8? = nil
    ) -> PlayView {
      PlayView(
        id: id, sectionId: nil, key: nil, variationIds: variationLabel.map { ["v-\($0)"] } ?? [],
        label: variationLabel,
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
        PlayView.preview(
          "p1", "C major", seconds: 250, duration: "4m 10s", score: 7, repCount: 8, repTarget: 10),
        PlayView.preview(
          "p2", "G major", seconds: 200, duration: "3m 20s", score: 9, tempo: 104, repCount: 10,
          repTarget: 10),
        PlayView.preview(
          "p3", "D major", seconds: 310, duration: "5m 10s", repCount: 4, repTarget: 10),
      ]
      return SetlistEntryView(
        id: "entry-variations", itemId: "exercise-2", itemTitle: "Major Scales",
        itemType: .exercise, position: 0, durationDisplay: "12m 40s", status: .completed,
        notes: nil, intention: "Even tone through the turn", plannedDurationSecs: nil,
        plannedDurationDisplay: nil, groupId: nil, removable: true, plannedSectionIds: [],
        plannedVariationIds: ["v-C major"], plannedLabel: "C major", plannedRepTarget: 10,
        plays: plays, scoreSummary: 8, record: .empty, plannedKey: nil)
    }

    /// One exercise practised in a single key: the detail screen names it
    /// rather than folding it silently into "type and time" (#1785).
    static var previewOneVariation: SetlistEntryView {
      let plays = [
        PlayView.preview(
          "p1", "E\u{266d} major", seconds: 360, duration: "6m 0s", tempo: 96)
      ]
      return SetlistEntryView(
        id: "entry-one-variation", itemId: "exercise-3", itemTitle: "Arpeggios",
        itemType: .exercise, position: 0, durationDisplay: "6m 0s", status: .completed,
        notes: nil, intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil,
        groupId: nil, removable: true, plannedSectionIds: [],
        plannedVariationIds: ["v-E\u{266d} major"],
        plannedLabel: "E\u{266d} major", plannedRepTarget: nil,
        plays: plays, scoreSummary: nil, record: .empty, plannedKey: nil)
    }
  }

  extension ActiveSessionView {
    /// Clair de Lune split into A1, B and A2, with A1's time up four seconds
    /// before the reference instant, so the move-on offer is drawn (#2315).
    static var previewActiveSectionTimeUp: ActiveSessionView {
      var active = previewActive
      active.record.segment = SegmentClockView(
        label: "A1", endsAt: "2026-05-30T09:04:08Z", moveLabel: "On to B",
        stayLabel: "Stay on A1, 2 more minutes taken from B")
      return active
    }

    /// Back after six minutes away (#2306).
    static var previewActiveAway: ActiveSessionView {
      var active = previewActive
      active.record.awayOffer = AwayOfferView(
        minutes: 6, label: "Away 6 minutes. Leave it out?")
      return active
    }

    static var previewActiveLastTime: ActiveSessionView {
      var active = previewActive
      active.record.lastTime = LastTimeView(
        entryId: "entry-1", sectionId: "sec-b", variationIds: ["v-dot"],
        label: "Last time: B · Dotted rhythms", key: nil)
      return active
    }
  }

  extension FinishSheetView {
    /// The core's words for the choices, copied for previews only.
    static func preview(asksIntention: Bool, read: IntentionMet? = nil) -> FinishSheetView {
      FinishSheetView(
        noteOffers: [
          NotePointView(
            span: NoteSpan(start: 23, end: 29), label: "Bar 12", sectionLabel: "A1",
            confirmed: false),
          NotePointView(
            span: NoteSpan(start: 38, end: 40), label: "\u{2669} = 84", sectionLabel: "A1",
            confirmed: true),
        ],
        asksIntention: asksIntention, intentionMetRead: read,
        feltChoices: [
          FeltChoiceView(felt: .comfortable, label: "Comfortable"),
          FeltChoiceView(felt: .hardWork, label: "Hard work"),
          FeltChoiceView(felt: .strained, label: "Strained"),
        ],
        obstacleChoices: [
          ObstacleChoiceView(obstacle: .notes, label: "Notes"),
          ObstacleChoiceView(obstacle: .rhythm, label: "Rhythm"),
          ObstacleChoiceView(obstacle: .fingering, label: "Fingering"),
          ObstacleChoiceView(obstacle: .memory, label: "Memory"),
          ObstacleChoiceView(obstacle: .tone, label: "Tone"),
          ObstacleChoiceView(obstacle: .tension, label: "Tension"),
        ], rows: [], sections: [], keys: [])
    }

    /// Clair de Lune's sections and keys, for changing what a row says was played (#2249).
    static var previewWayChoices: FinishSheetView {
      func part(_ id: String, _ label: String) -> SectionView {
        SectionView(
          id: id, name: label, kind: .form, targetBpm: nil, firstBar: nil, lastBar: nil,
          label: label, barsCaption: nil, barsFieldText: "", latestScore: nil, scoreHistory: [],
          caption: "Not yet played", isWeakest: false)
      }
      var finish = preview(asksIntention: false)
      finish.noteOffers = []
      finish.sections = [part("sec-a1", "A1"), part("sec-b", "B"), part("sec-a2", "A2")]
      finish.keys = [
        KeyChoiceView(key: nil, label: "Written key", caption: "D\u{266d} major"),
        KeyChoiceView(
          key: Key(letter: .d, accidental: .natural, mode: .major), label: "D major", caption: nil),
        KeyChoiceView(
          key: Key(letter: .g, accidental: .natural, mode: .major), label: "G major", caption: nil),
      ]
      finish.rows = [
        FinishRowView(
          playId: "p1", label: "A1 · G major", sectionId: "sec-a1",
          key: Key(letter: .g, accidental: .natural, mode: .major), variationIds: [],
          canChange: true),
        FinishRowView(
          playId: "p2", label: "B", sectionId: "sec-b", key: nil, variationIds: [], canChange: true),
      ]
      return finish
    }
  }
#endif
