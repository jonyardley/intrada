package com.intrada.android

import com.intrada.shared.AnalyticsView
import com.intrada.shared.ConsistencyWeekView
import com.intrada.shared.Direction
import com.intrada.shared.ItemKind
import com.intrada.shared.ItemRanking
import com.intrada.shared.PooledMarkView
import com.intrada.shared.ScoreChange
import com.intrada.shared.VariationCoverageView
import com.intrada.shared.WeeklySummary

// The iPhone's previewAnalytics, fixed because the sample sessions are dated from today.
object ProgressFixtures {
    private val hanon = ScoreChange("exercise-1", "Hanon No. 1", 2u, 3u, 1, false)

    val analytics =
        AnalyticsView(
            weeklySummary =
                WeeklySummary(
                    380u,
                    14uL,
                    11uL,
                    300u,
                    11uL,
                    9uL,
                    Direction.UP,
                    Direction.UP,
                    Direction.UP,
                    true,
                ),
            topItems = listOf(ItemRanking("piece-1", "Clair de Lune", ItemKind.PIECE, 180u, 9uL)),
            neglectedItems = emptyList(),
            scoreChanges =
                listOf(
                    ScoreChange("piece-1", "Clair de Lune", 3u, 4u, 1, false),
                    hanon,
                    ScoreChange("piece-2", "Gymnopédie No. 1", null, 3u, 0, true),
                ),
            variationCoverage =
                listOf(
                    VariationCoverageView(
                        "exercise-2",
                        "ii–V–i Enclosures",
                        1uL,
                        3uL,
                        "1 of 3 solid",
                        "ii–V–i Enclosures, 1 of 3 variations solid",
                    ),
                    VariationCoverageView(
                        "exercise-3",
                        "Chromatic run",
                        4uL,
                        12uL,
                        "4 of 12 solid",
                        "Chromatic run, 4 of 12 variations solid",
                    ),
                ),
            variationCoverageCaption = "5 of 15 solid",
            weekLine = "14 sessions · 6h 20m this week",
            consistencyWeeks =
                listOf(
                    ConsistencyWeekView("W1", 40u, false, "4 weeks ago: 40 minutes"),
                    ConsistencyWeekView("W2", 75u, false, "3 weeks ago: 75 minutes"),
                    ConsistencyWeekView("W3", 55u, false, "2 weeks ago: 55 minutes"),
                    ConsistencyWeekView("W4", 95u, false, "Last week: 95 minutes"),
                    ConsistencyWeekView("Now", 82u, true, "This week: 82 minutes"),
                ),
            bestWeek = "best week · 95m",
            overallMastery = 3.4,
            topMover = hanon,
            masteryChange = "+1.0 this week",
            climbing = "Climbing steadily across 2 items.",
            pooledVariations =
                listOf(
                    PooledMarkView("Dotted rhythms", 2uL, 5uL, "Solid on 2 of 5 items"),
                    PooledMarkView("Hands separately", 1uL, 3uL, "Solid on 1 of 3 items"),
                ),
            pooledKeys = listOf(PooledMarkView("E\u266D major", 1uL, 2uL, "Solid on 1 of 2 items")),
        )
}
