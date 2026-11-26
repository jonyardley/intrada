#if DEBUG
  import Foundation
  import SharedTypes

  func previewMetaParts(_ duration: String, tempo: UInt16? = nil) -> [String] {
    [duration] + [tempo.map { "\($0) bpm" }].compactMap { $0 }
  }

  extension ActiveRecordView {
    static var empty: ActiveRecordView {
      ActiveRecordView(awayOffer: nil, finish: nil, lastTime: nil)
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
    /// `consistencyWeeks` are the bars (40/75/55/95/82).
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
            currentScore: 4, delta: 1, isNew: false, subtitle: nil,
            spoken: "Clair de Lune, mastery up from 3 to 4", direction: .up),
          ScoreChange(
            itemId: "exercise-1", itemTitle: "Hanon No. 1", previousScore: 2,
            currentScore: 3, delta: 1, isNew: false, subtitle: nil,
            spoken: "Hanon No. 1, mastery up from 2 to 3", direction: .up),
          ScoreChange(
            itemId: "piece-2", itemTitle: "Gymnopédie No. 1", previousScore: nil,
            currentScore: 3, delta: 0, isNew: true, subtitle: "first confidence",
            spoken: "Gymnopédie No. 1, first confidence, mastery 3", direction: .up),
        ],
        variationCoverage: [
          VariationCoverageView(
            itemId: "exercise-2", title: LibraryItemView.previewExerciseWithVariations.title,
            solid: 1, total: 3, caption: "1 of 3 solid",
            spoken:
              "\(LibraryItemView.previewExerciseWithVariations.title), 1 of 3 variations solid"),
          VariationCoverageView(
            itemId: "exercise-3", title: "Chromatic run", solid: 4, total: 12,
            caption: "4 of 12 solid", spoken: "Chromatic run, 4 of 12 variations solid"),
        ],
        variationCoverageCaption: "5 of 15 solid",
        weekLine: "14 sessions · 6h 20m this week",
        consistencyWeeks: [
          ConsistencyWeekView(
            label: "W1", minutes: 40, isCurrent: false, spoken: "4 weeks ago: 40 minutes"),
          ConsistencyWeekView(
            label: "W2", minutes: 75, isCurrent: false, spoken: "3 weeks ago: 75 minutes"),
          ConsistencyWeekView(
            label: "W3", minutes: 55, isCurrent: false, spoken: "2 weeks ago: 55 minutes"),
          ConsistencyWeekView(
            label: "W4", minutes: 95, isCurrent: false, spoken: "Last week: 95 minutes"),
          ConsistencyWeekView(
            label: "Now", minutes: 82, isCurrent: true, spoken: "This week: 82 minutes"),
        ],
        bestWeek: "best week · 95m",
        overallMastery: 3.4,
        topMover: ScoreChange(
          itemId: "exercise-1", itemTitle: "Hanon No. 1", previousScore: 2,
          currentScore: 3, delta: 1, isNew: false, subtitle: nil,
          spoken: "Hanon No. 1, mastery up from 2 to 3", direction: .up),
        masteryChange: "+1.0 this week",
        climbing: "Climbing steadily across 2 items.",
        pooledVariations: [
          PooledMarkView(
            label: "Dotted rhythms", solid: 2, total: 5, caption: "Solid on 2 of 5 items",
            spoken: "Dotted rhythms, Solid on 2 of 5 items"),
          PooledMarkView(
            label: "Hands separately", solid: 1, total: 3, caption: "Solid on 1 of 3 items",
            spoken: "Hands separately, Solid on 1 of 3 items"),
        ],
        pooledKeys: [
          PooledMarkView(
            label: "E\u{266D} major", solid: 1, total: 2, caption: "Solid on 1 of 2 items",
            spoken: "E\u{266D} major, Solid on 1 of 2 items")
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
        days: days, practisedDays: UInt64(sessions.count), openingDay: openingDay,
        title: "This week")
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

    /// Twelve minutes with an aim.
    static var previewPlannedPiece: SetlistEntryView {
      var e = building(
        id: "plan-p", item: "piece-1", title: "Clair de Lune", type: .piece, position: 0)
      e.intention = "Get it up to 84 with the click"
      e.plannedDurationSecs = 720
      e.plannedDurationDisplay = "12 min"
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
        durationDisplay: "0s", status: .notAttempted, notes: nil, intention: nil,
        plannedDurationSecs: nil, plannedDurationDisplay: nil, groupId: group, removable: removable,
        plannedVariationIds: [], plannedLabel: nil, plays: [], scoreSummary: nil, plannedKey: nil,
        planTags: [])
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

  // Reason strings copy the core's plan rows (#3007): reword there and these go stale.
  extension PlanView {
    static func preview(
      lengthMins: UInt16 = 20, warmUp: PlanRowView? = .previewScales,
      hasWarmUpItems: Bool = true, warmUpEnabled: Bool = true,
      rows: [PlanRowView] = [.previewBach, .previewEnclosures]
    ) -> PlanView {
      let total = rows.reduce(warmUp?.minutes ?? 0) { $0 + $1.minutes }
      return PlanView(
        lengthMins: lengthMins, warmUp: warmUp, hasWarmUpItems: hasWarmUpItems,
        warmUpEnabled: warmUpEnabled, rows: rows, totalMins: total,
        startTitle: "Start session · \(total) min")
    }

    static var previewTypical: PlanView { preview() }
    static var previewEmptyWarmUp: PlanView { preview(warmUp: nil, hasWarmUpItems: false) }
    static var previewWarmUpOff: PlanView {
      preview(warmUp: nil, warmUpEnabled: false, rows: [.previewBach, .previewClair])
    }

    /// The library rows the plan's ids point at, so each row draws its kind's bar.
    static var previewItems: [LibraryItemView] {
      [
        LibraryItemFixture.view(id: "plan-scales", itemType: .exercise, title: "Major scales"),
        LibraryItemFixture.view(id: "plan-bach", title: "Bach Prelude in C"),
        LibraryItemFixture.view(
          id: "plan-enclosures", itemType: .exercise, title: "ii V i enclosures"),
        LibraryItemFixture.view(id: "plan-clair", title: "Clair de lune"),
      ]
    }
  }

  extension PlanRowView {
    static var previewScales: PlanRowView {
      PlanRowView(itemId: "plan-scales", title: "Major scales", reason: "Exercise", minutes: 3)
    }
    static var previewBach: PlanRowView {
      PlanRowView(
        itemId: "plan-bach", title: "Bach Prelude in C", reason: "Piece · a priority",
        minutes: 10)
    }
    static var previewEnclosures: PlanRowView {
      PlanRowView(
        itemId: "plan-enclosures", title: "ii V i enclosures",
        reason: "Exercise · going cold after 3 weeks", minutes: 7)
    }
    static var previewClair: PlanRowView {
      PlanRowView(
        itemId: "plan-clair", title: "Clair de lune", reason: "Piece · cold for 5 weeks",
        minutes: 10)
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
            2, "Nocturne Op. 9 No. 2", .piece, score: 6, tempo: 54),
        ],
        sessionScore: 7,
        playedSummary: "Clair de Lune · Gymnopédie No. 1 · Nocturne Op. 9 No. 2",
        dayLabel: "Yesterday", itemCountDisplay: "3 pieces")
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
        dayLabel: "Fri 29 May", itemCountDisplay: "1 exercise")
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
        dayLabel: "Wed 27 May", itemCountDisplay: "1 exercise")
    }

    /// No session mark, and the two statuses a detail view must state plainly
    /// rather than leave looking unmarked.
    static var previewEndedEarly: PracticeSessionView {
      PracticeSessionView(
        id: "session-2",
        totalDurationDisplay: "14m 0s", totalDurationSummary: "14m",
        completionStatus: .endedEarly, notes: nil,
        entries: [
          previewEntry(0, "Hanon No. 1", .exercise, score: 5),
          previewEntry(1, "Major Scales", .exercise, status: .notAttempted),
        ], sessionScore: nil, playedSummary: "Hanon No. 1",
        dayLabel: "Thu 28 May", itemCountDisplay: "2 exercises")
    }

    private static func previewEntry(
      _ position: UInt64, _ title: String, _ type: ItemKind,
      status: EntryStatus = .completed, score: UInt8? = nil, tempo: UInt16? = nil,
      notes: String? = nil
    ) -> SetlistEntryView {
      let plays =
        status == .completed
        ? [
          PlayView.fixture(
            id: "entry-\(position)-p1",
            metaParts: previewMetaParts("10 min", tempo: tempo),
            achievedTempo: tempo, tempoDisplay: tempo, score: score)
        ] : []
      return SetlistEntryView(
        id: "entry-\(position)", itemId: "item-\(position)", itemTitle: title, itemType: type,
        position: position, durationDisplay: "10 min", status: status, notes: notes,
        intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil, groupId: nil,
        removable: true, plannedVariationIds: [], plannedLabel: nil, plays: plays,
        scoreSummary: plays.isEmpty ? nil : score, plannedKey: nil,
        planTags: [])
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
        groupId: groupId, removable: removable, plannedVariationIds: [],
        plannedLabel: nil,
        plays: [PlayView.fixture(id: "entry-\(position)-p1")], scoreSummary: nil, plannedKey: nil,
        planTags: [])
    }

    /// Preview and snapshot fixture. A new core field costs one edit here rather
    /// than one per call site (#2921).
    static func fixture(
      currentItemTitle: String = "Clair de Lune", currentItemType: ItemKind = .piece,
      currentPosition: UInt64 = 0, totalItems: UInt64 = 1,
      startedAt: String = ActiveSessionView.previewSessionStartedAt, endsAt: String? = nil,
      currentItemStartedAt: String = ActiveSessionView.previewStartedAt,
      entries: [SetlistEntryView] = [], currentKey: Key? = nil,
      currentKeyChoices: [KeyChoiceView] = [], currentKeyLabel: String? = nil,
      currentVariationIds: [String] = [], currentPlayLabel: String? = nil,
      currentPlannedDurationSecs: UInt32? = nil, nextItemTitle: String? = nil,
      currentItemIntention: String? = nil, currentItemNotes: String? = nil,
      currentRelatedPieceTitle: String? = nil, currentItemTempoMarking: String? = nil,
      currentItemTempoBpm: UInt16? = nil, currentItemTempoLine: String? = nil,
      currentItemTempoLineSpoken: String? = nil, currentClickSounding: UInt16 = 0b1111,
      currentVariations: [PickerVariationView] = [], reflection: ReflectionView? = nil,
      clickSeedMetre: Metre = Metre(beats: 4, unit: 4, groups: nil), clickSeedBpm: UInt16 = 66,
      clickSeedSoundsTarget: Bool = true, clickSeedPresets: [ClickPresetOption] = [],
      clickSeedOpening: UInt16 = 0b1111, record: ActiveRecordView = .empty
    ) -> ActiveSessionView {
      ActiveSessionView(
        currentItemTitle: currentItemTitle, currentItemType: currentItemType,
        currentPosition: currentPosition, totalItems: totalItems, startedAt: startedAt,
        endsAt: endsAt, currentItemStartedAt: currentItemStartedAt, entries: entries,
        currentKey: currentKey,
        currentKeyChoices: currentKeyChoices, currentKeyLabel: currentKeyLabel,
        currentVariationIds: currentVariationIds, currentPlayLabel: currentPlayLabel,
        currentPlannedDurationSecs: currentPlannedDurationSecs, nextItemTitle: nextItemTitle,
        currentItemIntention: currentItemIntention, currentItemNotes: currentItemNotes,
        currentRelatedPieceTitle: currentRelatedPieceTitle,
        currentItemTempoMarking: currentItemTempoMarking,
        currentItemTempoBpm: currentItemTempoBpm, currentItemTempoLine: currentItemTempoLine,
        currentItemTempoLineSpoken: currentItemTempoLineSpoken,
        currentClickSounding: currentClickSounding, currentVariations: currentVariations,
        reflection: reflection, clickSeedMetre: clickSeedMetre, clickSeedBpm: clickSeedBpm,
        clickSeedSoundsTarget: clickSeedSoundsTarget, clickSeedPresets: clickSeedPresets,
        clickSeedOpening: clickSeedOpening, record: record)
    }

    static var previewActive: ActiveSessionView {
      .fixture(
        currentPosition: 1, totalItems: 5,
        entries: [
          previewEntry(0, "Hanon No. 1", .exercise),
          previewEntry(1, "Clair de Lune", .piece),
          previewEntry(2, "Major Scales", .exercise),
          previewEntry(3, "Gymnopédie No. 1", .piece),
          previewEntry(4, "Czerny Op. 299", .exercise),
        ],
        currentPlannedDurationSecs: 480,
        nextItemTitle: "Hanon No. 1",
        currentItemIntention: "Let the melody breathe",
        currentItemTempoMarking: "Andante", currentItemTempoBpm: 66,
        currentItemTempoLine: "Andante · ♩ = 66",
        currentItemTempoLineSpoken: "Andante, 66 beats per minute",
        clickSeedPresets: LimitsView.preview.clickPresets(
          for: Metre(beats: 4, unit: 4, groups: nil)))
    }

    /// The same session, run past an hour, so the `H:MM:SS` reading is drawn.
    static var previewActiveLongSession: ActiveSessionView {
      var active = previewActive
      active.startedAt = previewLongSessionStartedAt
      return active
    }

    /// The current item is an exercise practised in C, now on G: the chip reads
    /// the open play and the picker switches it (#1739).
    static var previewActiveVariations: ActiveSessionView {
      .fixture(
        currentItemTitle: LibraryItemView.previewExerciseWithVariations.title,
        currentItemType: .exercise,
        currentPosition: 0, totalItems: 3,
        entries: [
          variationEntry(), previewEntry(1, "Clair de Lune", .piece),
          previewEntry(2, "Czerny Op. 299", .exercise),
        ],
        currentVariationIds: ["variation-f"],
        currentPlayLabel: "F",
        currentPlannedDurationSecs: 480,
        nextItemTitle: "Clair de Lune",
        currentItemIntention: "Even tone through the turn",
        currentItemTempoBpm: 104, currentItemTempoLine: "♩ = 104",
        currentItemTempoLineSpoken: "104 beats per minute",
        currentVariations: [
          PickerVariationView(
            id: "variation-c", label: "C", caption: "Played this session · 3m 10s"),
          PickerVariationView(
            id: "variation-f", label: "F", caption: "Playing now"),
          PickerVariationView(
            id: "variation-bb", label: "B♭", caption: "Not yet played"),
        ],
        clickSeedBpm: 104,
        clickSeedPresets: LimitsView.preview.clickPresets(
          for: Metre(beats: 4, unit: 4, groups: nil)))
    }

    /// The current entry of `previewActiveVariations`: one closed play in C and
    /// an open one in F, whose seconds the core has not stamped yet.
    private static func variationEntry() -> SetlistEntryView {
      SetlistEntryView(
        id: "entry-0", itemId: "exercise-2",
        itemTitle: LibraryItemView.previewExerciseWithVariations.title,
        itemType: .exercise, position: 0, durationDisplay: "10 min", status: .notAttempted,
        notes: nil, intention: nil, plannedDurationSecs: nil, plannedDurationDisplay: nil,
        groupId: nil, removable: true, plannedVariationIds: ["variation-c"],
        plannedLabel: "C",
        plays: [
          PlayView.fixture(
            id: "entry-0-p1", variationIds: ["variation-c"], label: "C", seconds: 190,
            durationDisplay: "3m 10s", metaParts: ["3m 10s", "10 of 10 reps"]),
          PlayView.fixture(
            id: "entry-0-p2", variationIds: ["variation-f"], label: "F", seconds: 0,
            durationDisplay: "0s", metaParts: ["0s", "4 of 10 reps"]),
        ], scoreSummary: nil, plannedKey: nil,
        planTags: [])
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
          delta: 1, isNew: false, subtitle: nil, spoken: "Clair de Lune, mastery up from 3 to 4",
          direction: .up))
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
          PlayView.fixture(
            id: "\(id)-p1", seconds: seconds,
            durationDisplay: duration, metaParts: previewMetaParts(duration, tempo: tempo),
            achievedTempo: tempo, tempoDisplay: tempo, score: score)
        ] : []
      return SetlistEntryView(
        id: id, itemId: id, itemTitle: title, itemType: type, position: 0,
        durationDisplay: duration, status: status, notes: notes, intention: intention,
        plannedDurationSecs: nil, plannedDurationDisplay: nil, groupId: nil, removable: true,
        plannedVariationIds: [], plannedLabel: nil, plays: plays,
        scoreSummary: plays.isEmpty ? nil : score, plannedKey: nil,
        planTags: [])
    }
  }

  extension ReflectionPlay {
    static func preview(
      _ id: String, _ label: String?, _ duration: String, isMarkable: Bool = true
    ) -> ReflectionPlay {
      ReflectionPlay(
        id: id, variationLabel: label, durationDisplay: duration, isMarkable: isMarkable)
    }
  }

  extension ReflectionTempoView {
    /// Unstamped crotchets at the click's default unless given a stamp.
    static func preview(
      _ playId: String, tempo: UInt16? = nil,
      click: ClickState = ClickState(
        metre: Metre(beats: 4, unit: 4, groups: nil), sounding: 0b1111),
      lastScore: UInt8? = nil, startScore: UInt8 = 5
    ) -> ReflectionTempoView {
      let limits = LimitsView.preview
      let range = limits.clickBand(unit: click.metre.unit)
      let band = TempoBand(
        unit: click.metre.unit, min: UInt16(range.lowerBound), max: UInt16(range.upperBound))
      return ReflectionTempoView(
        playId: playId, tempo: tempo ?? limits.clickTempoDefault, click: click, band: band,
        setByHand: false, lastScore: lastScore, startScore: startScore)
    }
  }

  extension ReflectionAnswers {
    static func preview(
      marks: [DraftMark] = [], note: String = "", tempos: [DraftTempo] = [],
      ways: [DraftWay] = [], notes: [DraftNote] = []
    ) -> ReflectionAnswers {
      ReflectionAnswers(marks: marks, note: note, tempos: tempos, ways: ways, notes: notes)
    }
  }

  extension PlayView {
    /// Three variations of one item, as the item-complete sheet and the session
    /// detail read them (#1739).
    static func preview(
      _ id: String, _ variationLabel: String?, seconds: UInt64, duration: String,
      score: UInt8? = nil, tempo: UInt16? = nil
    ) -> PlayView {
      .fixture(
        id: id, variationIds: variationLabel.map { ["v-\($0)"] } ?? [],
        label: variationLabel,
        seconds: seconds, durationDisplay: duration,
        metaParts: previewMetaParts(duration, tempo: tempo),
        achievedTempo: tempo, tempoDisplay: tempo, score: score)
    }

    /// Preview and snapshot fixture. A new core field costs one edit here rather
    /// than one per call site (#2921).
    static func fixture(
      id: String = "play-1", key: Key? = nil, variationIds: [String] = [],
      label: String? = nil, seconds: UInt64 = 600, durationDisplay: String = "10 min",
      metaParts: [String] = ["10 min"], achievedTempo: UInt16? = nil,
      clickPattern: ClickState? = nil, tempoDisplay: UInt16? = nil, score: UInt8? = nil,
      isMarkable: Bool = true
    ) -> PlayView {
      PlayView(
        id: id, key: key, variationIds: variationIds, label: label, seconds: seconds,
        durationDisplay: durationDisplay, metaParts: metaParts, achievedTempo: achievedTempo,
        clickPattern: clickPattern, tempoDisplay: tempoDisplay, score: score,
        isMarkable: isMarkable)
    }
  }

  extension ClickBarOption {
    /// Preview and test fixture. A new core field costs one edit here rather
    /// than one per call site (#2921).
    static func fixture(
      beats: UInt8 = 4, groups: [UInt8]? = nil, presets: [ClickPresetOption] = [],
      opening: UInt16 = 0b1111
    ) -> ClickBarOption {
      ClickBarOption(beats: beats, groups: groups, presets: presets, opening: opening)
    }
  }

  extension SetlistEntryView {
    /// One exercise practised across three keys: the case #1739 exists for.
    static var previewThreeVariations: SetlistEntryView {
      let plays = [
        PlayView.preview(
          "p1", "C major", seconds: 250, duration: "4m 10s", score: 7),
        PlayView.preview(
          "p2", "G major", seconds: 200, duration: "3m 20s", score: 9, tempo: 104),
        PlayView.preview("p3", "D major", seconds: 310, duration: "5m 10s"),
      ]
      return SetlistEntryView(
        id: "entry-variations", itemId: "exercise-2", itemTitle: "Major Scales",
        itemType: .exercise, position: 0, durationDisplay: "12m 40s", status: .completed,
        notes: nil, intention: "Even tone through the turn", plannedDurationSecs: nil,
        plannedDurationDisplay: nil, groupId: nil, removable: true,
        plannedVariationIds: ["v-C major"], plannedLabel: "C major", plays: plays, scoreSummary: 8,
        plannedKey: nil,
        planTags: [])
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
        groupId: nil, removable: true,
        plannedVariationIds: ["v-E\u{266d} major"],
        plannedLabel: "E\u{266d} major", plays: plays, scoreSummary: nil, plannedKey: nil,
        planTags: [])
    }
  }

  extension ActiveSessionView {
    /// Back after six minutes away (#2306).
    static var previewActiveAway: ActiveSessionView {
      var active = previewActive
      active.record.awayOffer = AwayOfferView(
        minutes: 6, label: "Away 6 minutes. Leave it out?")
      return active
    }

    /// Clair de Lune kept in two keys, so the player shows the key chip (#2788).
    static var previewActiveKeys: ActiveSessionView {
      var active = previewActive
      active.currentKeyChoices = FinishSheetView.previewWayChoices.keys
      active.currentKeyLabel = "D\u{266d} major"
      return active
    }

    /// One key is no choice, so it reads as its name rather than a chip (#2788).
    static var previewActiveOneKey: ActiveSessionView {
      var active = previewActive
      active.currentKeyChoices = Array(FinishSheetView.previewWayChoices.keys.prefix(1))
      active.currentKeyLabel = "D\u{266d} major"
      return active
    }

    /// Both chips and the "Last time" offer on one row, the tightest the
    /// header gets (#2788).
    static var previewActiveKeysVariationsLastTime: ActiveSessionView {
      var active = previewActiveVariations
      active.currentKeyChoices = FinishSheetView.previewWayChoices.keys
      active.currentKey = Key(letter: .g, accidental: .natural, mode: .major)
      active.currentKeyLabel = "G major"
      active.record.lastTime = LastTimeView(
        entryId: "entry-0", variationIds: ["variation-c"],
        label: "Last time: D major · C", key: Key(letter: .d, accidental: .natural, mode: .major))
      return active
    }

    static var previewActiveLastTime: ActiveSessionView {
      var active = previewActive
      active.record.lastTime = LastTimeView(
        entryId: "entry-1", variationIds: ["v-dot"],
        label: "Last time: Dotted rhythms", key: nil)
      return active
    }
  }

  extension FinishSheetView {
    /// Clair de Lune's keys, for changing what a row says was played (#2249).
    static var previewWayChoices: FinishSheetView {
      var finish = FinishSheetView(rows: [], keys: [])
      finish.keys = [
        KeyChoiceView(key: nil, label: "D\u{266d} major", caption: "Written key"),
        KeyChoiceView(
          key: Key(letter: .d, accidental: .natural, mode: .major), label: "D major", caption: nil),
        KeyChoiceView(
          key: Key(letter: .g, accidental: .natural, mode: .major), label: "G major", caption: nil),
      ]
      finish.rows = [
        FinishRowView(
          playId: "p1", label: "G major",
          key: Key(letter: .g, accidental: .natural, mode: .major), variationIds: [],
          canChange: true),
        FinishRowView(
          playId: "p2", label: "No variation", key: nil, variationIds: [], canChange: true),
      ]
      return finish
    }
  }
#endif
