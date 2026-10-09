package com.intrada.android

import com.intrada.android.ui.PracticeModel
import com.intrada.shared.CompletionStatus
import com.intrada.shared.EntryRecordView
import com.intrada.shared.EntryStatus
import com.intrada.shared.ItemKind
import com.intrada.shared.LastPractisedView
import com.intrada.shared.PlayView
import com.intrada.shared.PracticeDayView
import com.intrada.shared.PracticeSessionView
import com.intrada.shared.PracticeWeekView
import com.intrada.shared.SetlistEntryView
import com.intrada.shared.SuggestedItem
import com.intrada.shared.SuggestedPlan
import com.intrada.shared.SuggestedSession

// Fixed, because the core dates live sessions from today.
object PracticeFixtures {
    fun play(
        id: String = "play-1",
        label: String? = null,
        duration: String = "8m",
        score: UByte? = null,
        tempo: UShort? = null,
        repTarget: UByte? = null,
        repCount: UByte? = null,
    ) =
        PlayView(
            id = id,
            variationIds = emptyList(),
            label = label,
            seconds = 480uL,
            durationDisplay = duration,
            repTarget = repTarget,
            repCount = repCount,
            achievedTempo = tempo,
            score = score,
            isMarkable = true,
        )

    fun entry(
        id: String,
        title: String,
        kind: ItemKind = ItemKind.PIECE,
        status: EntryStatus = EntryStatus.COMPLETED,
        plays: List<PlayView> = listOf(play()),
        score: UByte? = plays.firstNotNullOfOrNull { it.score },
        notes: String? = null,
    ) =
        SetlistEntryView(
            id = id,
            itemId = "item-$id",
            itemTitle = title,
            itemType = kind,
            position = 0uL,
            durationDisplay = "8m",
            status = status,
            notes = notes,
            removable = true,
            plannedSectionIds = emptyList(),
            plannedVariationIds = emptyList(),
            plays = plays,
            scoreSummary = score,
            record =
                EntryRecordView(
                    segments = emptyList(),
                    canAddSection = false,
                    intentionMetRead = false,
                    gotInTheWay = emptyList(),
                    notePoints = emptyList(),
                ),
        )

    val completed =
        PracticeSessionView(
            id = "session-1",
            totalDurationDisplay = "24:00",
            totalDurationSummary = "24 min",
            completionStatus = CompletionStatus.COMPLETED,
            notes = "Left hand finally settled in the middle section.",
            entries =
                listOf(
                    entry("e1", "Clair de Lune", plays = listOf(play(score = 4u, tempo = 66u))),
                    entry(
                        "e2",
                        "Hanon No. 1",
                        ItemKind.EXERCISE,
                        plays = listOf(play(score = 3u, repTarget = 5u, repCount = 5u)),
                    ),
                    entry(
                        "e3",
                        "Gymnopédie No. 1",
                        status = EntryStatus.SKIPPED,
                        plays = emptyList(),
                    ),
                ),
            sessionScore = 4u,
            playedSummary = "Clair de Lune · Hanon No. 1",
            dayLabel = "Today",
        )

    val withVariations =
        PracticeSessionView(
            id = "session-2",
            totalDurationDisplay = "18:00",
            totalDurationSummary = "18 min",
            completionStatus = CompletionStatus.ENDEDEARLY,
            entries =
                listOf(
                    entry(
                        "e4",
                        "Major scales",
                        ItemKind.EXERCISE,
                        plays =
                            listOf(
                                play("p1", "C major", "4m", score = 4u, tempo = 92u),
                                play("p2", "G major", "3m", score = 2u),
                                play("p3", "D major", "2m"),
                            ),
                        score = 3u,
                    ),
                    entry(
                        "e5",
                        "Arpeggios",
                        ItemKind.EXERCISE,
                        plays = listOf(play(label = "E♭ major", score = 3u, tempo = 80u)),
                    ),
                    entry(
                        "e6",
                        "Bach Prelude in C",
                        status = EntryStatus.NOTATTEMPTED,
                        plays = emptyList(),
                    ),
                ),
            playedSummary = "Major scales in C, G and D · Arpeggios in E♭ major",
            dayLabel = "Tue 6 Oct",
        )

    private val dates = listOf(5, 6, 7, 8, 9, 10, 11)

    val week =
        PracticeWeekView(
            days =
                dates.mapIndexed { index, date ->
                    PracticeDayView(
                        date = "2026-10-${date.toString().padStart(2, '0')}",
                        weekdayInitial = "MTWTFSS"[index].toString(),
                        dayNumber = date.toUInt(),
                        fullDate = "${FULL[index]} $date October",
                        heading =
                            when (date) {
                                9 -> "Today"
                                8 -> "Yesterday"
                                else -> "${FULL[index]} $date October"
                            },
                        isToday = date == 9,
                        isFuture = date > 9,
                        sessionIds =
                            when (date) {
                                6 -> listOf(withVariations.id)
                                9 -> listOf(completed.id)
                                else -> emptyList()
                            },
                    )
                },
            practisedDays = 2uL,
            openingDay = 4uL,
        )

    val filled =
        PracticeModel(
            weeks = listOf(week),
            sessions = listOf(completed, withVariations),
            lastPractised = LastPractisedView("Clair de Lune", "Today", "Last practised today"),
            greeting = "Good morning, Jon",
        )

    val earlier =
        completed.copy(
            id = "session-3",
            totalDurationSummary = "30 min",
            sessionScore = 3u,
            dayLabel = "Thu 1 Oct",
        )

    val earlierWeek =
        PracticeWeekView(
            days =
                listOf(28, 29, 30, 1, 2, 3, 4).mapIndexed { index, date ->
                    val month = if (date > 20) "09" else "10"
                    val name = if (date > 20) "September" else "October"
                    PracticeDayView(
                        date = "2026-$month-${date.toString().padStart(2, '0')}",
                        weekdayInitial = "MTWTFSS"[index].toString(),
                        dayNumber = date.toUInt(),
                        fullDate = "${FULL[index]} $date $name",
                        heading = "${FULL[index]} $date $name",
                        isToday = false,
                        isFuture = false,
                        sessionIds = if (date == 1) listOf(earlier.id) else emptyList(),
                    )
                },
            practisedDays = 1uL,
            openingDay = 3uL,
        )

    val twoWeeks =
        PracticeModel(
            weeks = listOf(earlierWeek, week),
            sessions = listOf(completed, withVariations, earlier),
            lastPractised = filled.lastPractised,
            greeting = filled.greeting,
        )

    private val hanon =
        SuggestedItem(
            itemId = "item-hanon",
            itemTitle = "Hanon No. 1",
            itemType = ItemKind.EXERCISE,
            reason = "Linked to this piece",
        )

    val plan =
        SuggestedPlan(
            blocks =
                listOf(
                    SuggestedSession(
                        pieceId = "item-clair",
                        pieceTitle = "Clair de Lune",
                        pieceSubtitle = "Claude Debussy",
                        reason = "Starred · not played for 6 days",
                        priority = true,
                        items =
                            listOf(
                                hanon,
                                SuggestedItem(
                                    itemId = "item-clair",
                                    itemTitle = "Clair de Lune",
                                    itemType = ItemKind.PIECE,
                                    latestScore = 3u,
                                    reason = "Last marked 3",
                                    weakestSection = "Weakest section · Bars 15 to 18",
                                ),
                            ),
                        estimatedMinutes = 15u,
                    )
                ),
            estimatedMinutes = 15u,
            itemCount = 2u,
        )

    private val satieBlock =
        SuggestedSession(
            pieceId = "item-satie",
            pieceTitle = "Gymnopédie No. 1",
            pieceSubtitle = "Erik Satie",
            reason = "Not played yet",
            priority = false,
            items =
                listOf(
                    SuggestedItem(
                        itemId = "item-satie",
                        itemTitle = "Gymnopédie No. 1",
                        itemType = ItemKind.PIECE,
                        reason = "Not played yet",
                    )
                ),
            estimatedMinutes = 10u,
        )

    val filledPlan =
        plan.copy(
            blocks = plan.blocks + satieBlock,
            estimatedMinutes = 25u,
            itemCount = 3u,
            lengthMins = 25u,
        )

    val suggested =
        PracticeModel(
            weeks = filled.weeks,
            sessions = filled.sessions,
            lastPractised = filled.lastPractised,
            greeting = filled.greeting,
            upNext = plan,
            showsPriorities = true,
        )

    val empty =
        PracticeModel(
            weeks =
                listOf(
                    week.copy(
                        days = week.days.map { it.copy(sessionIds = emptyList()) },
                        practisedDays = 0uL,
                    )
                ),
            sessions = emptyList(),
            lastPractised = null,
        )
}

private val FULL =
    listOf("Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday")
