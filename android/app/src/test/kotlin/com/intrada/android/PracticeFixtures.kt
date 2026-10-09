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
