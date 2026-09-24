#if DEBUG
  import Foundation
  import SharedTypes

  extension TempoTrendDisplay {
    /// Nine sessions, seven measured: two breaks in the line, one of them two
    /// sessions wide.
    static let previewWithGaps = make([88, 92, 96, nil, 100, 104, nil, 108, 116])
    /// One measurement, which is less than a trend.
    static let previewSingleMeasurement = make([108])

    /// Built through the shipped mapping, so a fixture cannot drift from what
    /// the screen actually renders.
    static func make(_ tempos: [Int?]) -> TempoTrendDisplay {
      let summary = ItemPracticeSummary.fixture(
        tempoTrend: .fixture(tempos.map { $0.map { UInt16($0) } }))
      guard
        let display = summary.tempoTrendDisplay(
          locale: Locale(identifier: "en_US"), calendar: PreviewCalendar.utc)
      else { preconditionFailure("preview tempos must include a measurement") }
      return display
    }
  }

  extension TempoTrendView {
    /// `tempos[i]` is the i-th session's tempo, `nil` where none was measured.
    /// Dates run three days apart and end on 2026-06-24, matching the
    /// `scoreHistory` fixtures beside it, so a summary's `lastPracticedAt` and
    /// its trend agree however many tempos are passed.
    static func fixture(_ tempos: [UInt16?]) -> TempoTrendView {
      let newest = Date(timeIntervalSince1970: 1_782_291_600)
      let formatter = ISO8601DateFormatter()
      formatter.formatOptions = [.withInternetDateTime]
      let points = tempos.enumerated().map { index, tempo in
        let daysBack = Double(tempos.count - 1 - index) * 3 * 86_400
        return TempoTrendPoint(
          sessionDate: formatter.string(from: newest.addingTimeInterval(-daysBack)),
          sessionId: "t\(index)",
          tempo: tempo)
      }
      return TempoTrendView(points: points, hasTrend: tempos.compactMap { $0 }.count >= 2)
    }
  }

  extension ItemPracticeSummary {
    /// Preview and snapshot fixture. A new core field costs one edit here rather
    /// than one per call site (CLAUDE.md, Testing).
    static func fixture(
      sessionCount: UInt64 = 0,
      totalMinutes: UInt32 = 0,
      latestScore: UInt8? = nil,
      scoreHistory: [ScoreHistoryEntry] = [],
      tempoTrend: TempoTrendView = TempoTrendView(points: [], hasTrend: false),
      lastPracticedAt: String? = nil
    ) -> ItemPracticeSummary {
      ItemPracticeSummary(
        sessionCount: sessionCount, totalMinutes: totalMinutes, latestScore: latestScore,
        scoreHistory: scoreHistory, tempoTrend: tempoTrend,
        lastPracticedAt: lastPracticedAt)
    }
  }

  extension LibraryItemView {
    static var previewPiece: LibraryItemView {
      LibraryItemView(
        id: "piece-1", itemType: .piece, title: "Clair de Lune", subtitle: "Claude Debussy",
        key: "Db", modality: .major, tempo: "Andante (72 BPM)", tempoMarking: "Andante",
        tempoBpm: 72,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        priority: false,
        linkedExercises: [
          LinkedExerciseView(
            id: "exercise-1", title: "Hanon No. 1", key: "C major", tempo: "♩ = 108",
            practice: nil, pieceContextScore: 7),
          LinkedExerciseView(
            id: "exercise-2", title: "Db Major Scale", key: "Db major", tempo: nil, practice: nil,
            pieceContextScore: nil),
        ],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil,
        variants: [], ladderIsKeys: false, photoId: nil, showsKey: true, solidVariationCount: 0)
    }

    static var previewExercise: LibraryItemView {
      LibraryItemView(
        id: "exercise-1", itemType: .exercise, title: "Hanon No. 1",
        subtitle: "Charles-Louis Hanon",
        key: "C", modality: .major, tempo: "108 BPM", tempoMarking: nil, tempoBpm: 108,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        priority: false, linkedExercises: [],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil, variants: [],
        ladderIsKeys: false,
        photoId: nil, showsKey: true, solidVariationCount: 0)
    }

    /// The library item behind `previewGroupedScales`, so a block member and a
    /// library row can share an id.
    static var previewScales: LibraryItemView {
      LibraryItemView(
        id: "ex-a", itemType: .exercise, title: "Scales", subtitle: "",
        key: nil, modality: nil, tempo: nil, tempoMarking: nil, tempoBpm: nil,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        priority: false, linkedExercises: [],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil, variants: [],
        ladderIsKeys: false,
        photoId: nil, showsKey: true, solidVariationCount: 0)
    }

    static var previewDetail: LibraryItemView {
      LibraryItemView(
        id: "piece-3", itemType: .piece, title: "Clair de Lune", subtitle: "Claude Debussy",
        key: "Db", modality: .major, tempo: "Andante (72 BPM)", tempoMarking: "Andante",
        tempoBpm: 72,
        notes: "Focus on the rubato in the opening phrase; keep the left hand soft.",
        tags: ["recital", "impressionist", "memorised"], createdAt: "", updatedAt: "",
        practice: nil, priority: false, linkedExercises: [],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil,
        variants: [], ladderIsKeys: false, photoId: nil, showsKey: true, solidVariationCount: 0)
    }

    /// A charted piece: exercises the chord-chart card (parsed grid + preview).
    static var previewCharted: LibraryItemView {
      func chord(_ raw: String, _ root: UInt8, _ q: ChordQuality) -> ChartChord {
        ChartChord(
          symbol: ChordSymbol(root: root, quality: q, extensions: [], bass: nil, raw: raw))
      }
      let chart = ChordChart(
        key: "G", modality: .minor,
        sections: [
          ChartSection(
            label: "A",
            bars: [
              Bar(chords: [chord("Cm7", 0, .min7)]),
              Bar(chords: [chord("F7", 5, .dom7)]),
              Bar(chords: [chord("Bbmaj7", 10, .maj7)]),
              Bar(chords: [chord("Ebmaj7", 3, .maj7)]),
            ])
        ])
      return LibraryItemView(
        id: "piece-charted", itemType: .piece, title: "Autumn Leaves", subtitle: "Standard",
        key: "G", modality: .minor, tempo: nil, tempoMarking: nil, tempoBpm: nil,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        priority: false, linkedExercises: [],
        usedIn: [], scaffoldPreview: .preview, chordChart: chart, metre: nil, variants: [],
        ladderIsKeys: false,
        photoId: nil, showsKey: true, solidVariationCount: 0)
    }

    static var previewMinimal: LibraryItemView { LibraryItemFixture.view() }

    /// A piece with a populated linked-exercises list (3 items, varied scores including
    /// one unrated), for the linked-exercises section snapshots.
    static var previewDetailWithLinkedExercises: LibraryItemView {
      LibraryItemView(
        id: "piece-3", itemType: .piece, title: "Clair de Lune", subtitle: "Claude Debussy",
        key: "Db", modality: .major, tempo: "Andante (72 BPM)", tempoMarking: "Andante",
        tempoBpm: 72,
        notes: "Focus on the rubato in the opening phrase; keep the left hand soft.",
        tags: ["recital", "impressionist"], createdAt: "", updatedAt: "",
        practice: ItemPracticeSummary.fixture(
          sessionCount: 12, totalMinutes: 240, latestScore: 6,
          scoreHistory: [
            ScoreHistoryEntry(sessionDate: "2026-06-24T09:00:00Z", score: 6, sessionId: "s1"),
            ScoreHistoryEntry(sessionDate: "2026-06-21T09:00:00Z", score: 5, sessionId: "s2"),
            ScoreHistoryEntry(sessionDate: "2026-06-18T09:00:00Z", score: 4, sessionId: "s3"),
          ],
          tempoTrend: .fixture([66, nil, 69, 72]),
          lastPracticedAt: "2026-06-24T09:00:00Z"),
        priority: false,
        linkedExercises: [
          // Per-piece scores deliberately differ from the exercises' overall
          // `latestScore` (7 / 4), so the snapshot shows the B2 re-source.
          LinkedExerciseView(
            id: "exercise-1", title: "Hanon No. 1", key: "C major", tempo: "♩ = 108",
            practice: ItemPracticeSummary.fixture(
              sessionCount: 8, totalMinutes: 60, latestScore: 7, scoreHistory: [],
              tempoTrend: .fixture([100, 104, 108]),
              lastPracticedAt: "2026-06-28T09:00:00Z"),
            pieceContextScore: 5),
          LinkedExerciseView(
            id: "exercise-2", title: "Db Major Scale", key: "Db major", tempo: nil,
            practice: ItemPracticeSummary.fixture(
              sessionCount: 3, totalMinutes: 20, latestScore: 4, scoreHistory: [],
              lastPracticedAt: "2026-06-25T09:00:00Z"),
            pieceContextScore: 6),
          LinkedExerciseView(
            id: "exercise-3", title: "Arpeggios in Db", key: nil, tempo: nil,
            practice: nil, pieceContextScore: nil),
        ],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil,
        variants: [], ladderIsKeys: false, photoId: nil, showsKey: true, solidVariationCount: 0)
    }

    /// Linked to 2 pieces, neither practised yet: every row unrated (#1363).
    static var previewExerciseLinkedOnly: LibraryItemView {
      LibraryItemView(
        id: "exercise-1", itemType: .exercise, title: "Hanon No. 1",
        subtitle: "Charles-Louis Hanon",
        key: "C", modality: .major, tempo: "108 BPM", tempoMarking: nil, tempoBpm: 108,
        notes: nil, tags: [], createdAt: "", updatedAt: "",
        practice: ItemPracticeSummary.fixture(
          sessionCount: 9, totalMinutes: 90, latestScore: 7,
          scoreHistory: [
            ScoreHistoryEntry(sessionDate: "2026-06-24T09:00:00Z", score: 7, sessionId: "e1"),
            ScoreHistoryEntry(sessionDate: "2026-06-21T09:00:00Z", score: 6, sessionId: "e2"),
            ScoreHistoryEntry(sessionDate: "2026-06-18T09:00:00Z", score: 5, sessionId: "e3"),
          ],
          tempoTrend: .fixture([96, 100, nil, 104, 108]),
          lastPracticedAt: "2026-06-24T09:00:00Z"),
        priority: false, linkedExercises: [],
        usedIn: [
          ExerciseUsageView(
            piece: PieceRefView(id: "piece-1", title: "Clair de Lune", subtitle: "Claude Debussy"),
            linked: true, latestScore: nil, sessionCount: 0, lastPracticedAt: nil,
            pieceRemoved: false),
          ExerciseUsageView(
            piece: PieceRefView(id: "piece-2", title: "Gymnopédie No. 1", subtitle: "Erik Satie"),
            linked: true, latestScore: nil, sessionCount: 0, lastPracticedAt: nil,
            pieceRemoved: false),
        ], scaffoldPreview: nil, chordChart: nil, metre: nil, variants: [], ladderIsKeys: false,
        photoId: nil, showsKey: true, solidVariationCount: 0)
    }

    /// Every "Used in" row state at once: linked and practised, practised
    /// only, linked only, removed, and "On its own" (#1087 B2, #1363).
    static var previewExerciseUsedIn: LibraryItemView {
      LibraryItemView(
        id: "exercise-1", itemType: .exercise, title: "Enclosures",
        subtitle: "Bebop vocabulary",
        key: "C", modality: .major, tempo: "120 BPM", tempoMarking: nil, tempoBpm: 120,
        notes: nil, tags: [], createdAt: "", updatedAt: "",
        practice: ItemPracticeSummary.fixture(
          sessionCount: 10, totalMinutes: 120, latestScore: 7,
          scoreHistory: [
            ScoreHistoryEntry(sessionDate: "2026-06-24T09:00:00Z", score: 7, sessionId: "e1")
          ],
          tempoTrend: .fixture([120]),
          lastPracticedAt: "2026-06-24T09:00:00Z"),
        priority: false, linkedExercises: [],
        usedIn: [
          ExerciseUsageView(
            piece: PieceRefView(
              id: "piece-1", title: "Strasbourg / St. Denis", subtitle: "Woody Shaw"),
            linked: true, latestScore: 7, sessionCount: 3,
            lastPracticedAt: "2026-06-24T09:00:00Z", pieceRemoved: false),
          ExerciseUsageView(
            piece: PieceRefView(id: "piece-2", title: "Blue Bossa", subtitle: "Kenny Dorham"),
            linked: false, latestScore: 5, sessionCount: 2,
            lastPracticedAt: "2026-06-22T09:00:00Z", pieceRemoved: false),
          ExerciseUsageView(
            piece: PieceRefView(id: "piece-3", title: "Autumn Leaves", subtitle: "Kosma"),
            linked: true, latestScore: nil, sessionCount: 0, lastPracticedAt: nil,
            pieceRemoved: false),
          ExerciseUsageView(
            piece: PieceRefView(id: "piece-gone", title: "Solar", subtitle: nil),
            linked: false, latestScore: 4, sessionCount: 2,
            lastPracticedAt: "2026-06-20T09:00:00Z", pieceRemoved: true),
          ExerciseUsageView(
            piece: nil, linked: false, latestScore: 6, sessionCount: 4,
            lastPracticedAt: "2026-06-21T09:00:00Z", pieceRemoved: false),
        ], scaffoldPreview: nil, chordChart: nil, metre: nil, variants: [], ladderIsKeys: false,
        photoId: nil, showsKey: true, solidVariationCount: 0)
    }

    /// An exercise with variations: one solid, one marked but not yet solid,
    /// one unrated.
    static var previewExerciseWithVariations: LibraryItemView {
      LibraryItemView(
        id: "exercise-2", itemType: .exercise, title: "ii–V–i Enclosures",
        subtitle: "Bebop vocabulary, 12 keys",
        key: "C", modality: .major, tempo: "132 BPM", tempoMarking: nil, tempoBpm: 132,
        notes: nil, tags: [], createdAt: "", updatedAt: "",
        practice: nil, priority: false, linkedExercises: [],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil,
        variants: [
          VariantView(
            id: "variation-c", label: "C", position: 0, latestScore: 9, scoreHistory: [],
            isSolid: true, caption: "Solid · 9 of 10"),
          VariantView(
            id: "variation-f", label: "F", position: 1, latestScore: 5, scoreHistory: [],
            isSolid: false, caption: "5 of 10"),
          VariantView(
            id: "variation-bb", label: "B♭", position: 2, latestScore: nil, scoreHistory: [],
            isSolid: false, caption: "Not yet played"),
        ], ladderIsKeys: true, photoId: nil, showsKey: false, solidVariationCount: 1)
    }

    /// Twelve chromatic variations, stress-testing the horizontal scroller
    /// at max realistic length (#1083 C2).
    static var previewExerciseWithTwelveVariations: LibraryItemView {
      let keys = [
        "C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B",
      ]
      return LibraryItemView(
        id: "exercise-3", itemType: .exercise, title: "Chromatic run",
        subtitle: "All 12 keys",
        key: nil, modality: nil, tempo: "72 BPM", tempoMarking: nil, tempoBpm: 72,
        notes: nil, tags: [], createdAt: "", updatedAt: "",
        practice: nil, priority: false, linkedExercises: [],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil,
        variants: keys.enumerated().map { index, label in
          let solid = index < 4
          let current = index == 4
          return VariantView(
            id: "variation-\(index)", label: label, position: UInt64(index),
            latestScore: solid ? 9 : (current ? 6 : nil), scoreHistory: [],
            isSolid: solid,
            caption: solid ? "Solid · 9 of 10" : (current ? "6 of 10" : "Not yet played"))
        }, ladderIsKeys: true, photoId: nil, showsKey: false, solidVariationCount: 4)
    }

    /// Variations that are inversions, not keys: pins the "variations" word and the stairs
    /// glyph, the one judgement the shell still makes for itself (#1467).
    static var previewExerciseWithNamedVariations: LibraryItemView {
      let rungs = ["Root position", "1st inversion", "2nd inversion"]
      return LibraryItemView(
        id: "exercise-4", itemType: .exercise, title: "Triad inversions", subtitle: "",
        key: nil, modality: nil, tempo: nil, tempoMarking: nil, tempoBpm: nil,
        notes: nil, tags: [], createdAt: "", updatedAt: "",
        practice: nil, priority: false, linkedExercises: [],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil,
        variants: rungs.enumerated().map { index, label in
          VariantView(
            id: "rung-\(index)", label: label, position: UInt64(index), latestScore: nil,
            scoreHistory: [], isSolid: false, caption: "Not yet played")
        }, ladderIsKeys: false, photoId: nil, showsKey: false, solidVariationCount: 0)
    }

    /// Free-text variation names, matching the issue's own example (#1786).
    static var previewExerciseWithLongVariationName: LibraryItemView {
      let names = [
        "Hands together, two octaves", "Hands separately", "Slow, with the metronome",
      ]
      return LibraryItemView(
        id: "exercise-5", itemType: .exercise, title: "Arpeggios, four octaves", subtitle: "",
        key: nil, modality: nil, tempo: nil, tempoMarking: nil, tempoBpm: nil,
        notes: nil, tags: [], createdAt: "", updatedAt: "",
        practice: nil, priority: false, linkedExercises: [],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil,
        variants: names.enumerated().map { index, label in
          VariantView(
            id: "long-variation-\(index)", label: label, position: UInt64(index),
            latestScore: index == 0 ? 8 : nil, scoreHistory: [], isSolid: index == 0,
            caption: index == 0 ? "Solid · 8 of 10" : "Not yet played")
        }, ladderIsKeys: false, photoId: nil, showsKey: false, solidVariationCount: 1)
    }

    /// A piece with no linked exercises, for the empty-state snapshot.
    static var previewDetailLinkedEmpty: LibraryItemView {
      LibraryItemView(
        id: "piece-4", itemType: .piece, title: "Gymnopédie No. 1", subtitle: "Erik Satie",
        key: "D", modality: .major, tempo: "Lent et douloureux (60 BPM)",
        tempoMarking: "Lent et douloureux",
        tempoBpm: 60, notes: nil, tags: [], createdAt: "", updatedAt: "",
        practice: nil, priority: false,
        linkedExercises: [], usedIn: [], scaffoldPreview: nil,
        chordChart: nil, metre: nil, variants: [], ladderIsKeys: false, photoId: nil,
        showsKey: true, solidVariationCount: 0
      )
    }
  }

  extension PhotoDraft {
    /// A clean title and tempo, and a composer the OCR was not sure of (#1436).
    static var readPage: PhotoDraft {
      PhotoDraft(
        title: TextDraftField(
          value: "Autumn Leaves", source: .recognised, confidence: 0.93, weak: false),
        composer: TextDraftField(
          value: "Joseph Kosmo", source: .recognised, confidence: 0.34, weak: true),
        tempo: TempoDraftField(
          value: Tempo(marking: "Moderato", bpm: 120), source: .recognised, confidence: 0.9,
          weak: false),
        chartText: nil)
    }

    static var otherReadPage: PhotoDraft {
      PhotoDraft(
        title: TextDraftField(
          value: "Blues in F", source: .recognised, confidence: 0.9, weak: false),
        composer: TextDraftField(
          value: "Count Basie", source: .recognised, confidence: 0.88, weak: false),
        tempo: nil,
        chartText: nil)
    }

    static var readNothing: PhotoDraft {
      PhotoDraft(title: nil, composer: nil, tempo: nil, chartText: nil)
    }
  }

  /// One library item for previews and tests: override only the fields in play.
  enum LibraryItemFixture {
    static func view(
      id: String = "piece-2", itemType: ItemKind = .piece, title: String = "Prelude in C",
      subtitle: String = "", key: String? = nil, modality: Modality? = nil
    ) -> LibraryItemView {
      LibraryItemView(
        id: id, itemType: itemType, title: title, subtitle: subtitle,
        key: key, modality: modality, tempo: nil, tempoMarking: nil, tempoBpm: nil,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        priority: false, linkedExercises: [],
        usedIn: [], scaffoldPreview: nil, chordChart: nil, metre: nil, variants: [],
        ladderIsKeys: false,
        photoId: nil, showsKey: true, solidVariationCount: 0)
    }

    static func record(
      id: String = "p1", title: String = "Etude", kind: ItemKind = .piece,
      composer: String? = nil, key: String? = nil, modality: Modality? = nil,
      tempo: Tempo? = nil, notes: String? = nil, tags: [String] = [],
      linkedExerciseIds: [String] = [], createdAt: String = "2026-01-01T00:00:00Z",
      updatedAt: String? = nil, priority: Bool = false, chordChart: ChordChart? = nil,
      photoId: String? = nil, metre: Metre? = nil
    ) -> Item {
      Item(
        id: id, title: title, kind: kind, composer: composer, key: key, modality: modality,
        tempo: tempo, notes: notes, tags: tags, linkedExerciseIds: linkedExerciseIds,
        createdAt: createdAt, updatedAt: updatedAt ?? createdAt, priority: priority,
        chordChart: chordChart, variants: [], photoId: photoId, metre: metre)
    }
  }
#endif
