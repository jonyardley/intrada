package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.CreateItem
import com.intrada.shared.Direction
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.SessionEvent
import com.intrada.shared.TempoReading
import java.time.Duration
import java.time.Instant
import java.time.ZoneOffset
import java.time.temporal.ChronoUnit
import java.time.temporal.IsoFields
import org.junit.Assert.assertEquals
import org.junit.Assume.assumeTrue
import org.junit.Test

// Progress's spoken rows on Kotlin's decoder, through the real core (#846, #2506). Stamped now,
// since Recent mastery reads this week.
class ProgressRoundTripTest {
    private val silent = TempoReading(bpm = 72.toUShort(), clickSounding = false)

    @Test
    fun progressRowsCarryTheirSpokenWords() {
        val start = Instant.now().truncatedTo(ChronoUnit.SECONDS).minus(Duration.ofMinutes(30))
        val week = { at: Instant ->
            at.atZone(ZoneOffset.UTC).get(IsoFields.WEEK_OF_WEEK_BASED_YEAR)
        }
        assumeTrue(week(start) == week(Instant.now()))
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val scales = addExercise(bridge, "Major Scales")
        bridge.update(
            Event.Item(
                ItemEvent.UpdateItemVariations(scales, emptyList(), listOf("Dotted rhythms"))
            )
        )
        val dotted = bridge.view().variations.single { it.label == "Dotted rhythms" }.id
        val arpeggios = addExercise(bridge, "Arpeggios")
        bridge.update(
            Event.Item(ItemEvent.UpdateItemVariations(arpeggios, listOf(dotted), emptyList()))
        )

        markOnce(bridge, scales, dotted, start, 9)
        markOnce(bridge, arpeggios, dotted, start.plus(Duration.ofMinutes(7)), 5)
        markOnce(bridge, scales, null, start.plus(Duration.ofMinutes(14)), 9)

        val analytics = bridge.view().analytics
        assertEquals(
            listOf("Dotted rhythms, Solid on 1 of 2 items"),
            analytics?.pooledVariations?.map { it.spoken },
        )
        val first = analytics?.scoreChanges?.firstOrNull { it.itemTitle == "Major Scales" }
        assertEquals("first time marked", first?.subtitle)
        assertEquals("Major Scales, first time marked, mastery 9", first?.spoken)
        assertEquals(Direction.UP, first?.direction)
    }

    // A fall says so, so the row is not painted as a win (#2551).
    @Test
    fun aFallenMarkSaysItFell() {
        val start = Instant.now().truncatedTo(ChronoUnit.SECONDS).minus(Duration.ofMinutes(30))
        val week = { at: Instant ->
            at.atZone(ZoneOffset.UTC).get(IsoFields.WEEK_OF_WEEK_BASED_YEAR)
        }
        assumeTrue(week(start) == week(Instant.now()))
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val scales = addExercise(bridge, "Major Scales")

        markOnce(bridge, scales, null, start.minus(Duration.ofDays(7)), 6)
        markOnce(bridge, scales, null, start, 4)

        val fall = bridge.view().analytics?.scoreChanges?.singleOrNull()
        assertEquals("Major Scales, mastery down from 6 to 4", fall?.spoken)
        assertEquals(Direction.DOWN, fall?.direction)
    }

    private fun addExercise(bridge: LiveBridge, title: String): String =
        bridge
            .update(
                Event.Item(
                    ItemEvent.Add(
                        CreateItem(
                            title = title,
                            kind = ItemKind.EXERCISE,
                            tags = emptyList(),
                            variationLabels = emptyList(),
                        )
                    )
                )
            )
            .mapNotNull {
                ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)?.value
            }
            .single()
            .id

    // Only a plain run-through counts towards Recent mastery, so a null variation is the one that
    // marks the item.
    private fun markOnce(
        bridge: LiveBridge,
        item: String,
        variation: String?,
        start: Instant,
        score: Int,
    ) {
        val at = { minutes: Long -> start.plus(Duration.ofMinutes(minutes)).toString() }
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(item)))
        val entryId = bridge.view().buildingSetlist?.entries?.firstOrNull()?.id.orEmpty()
        bridge.update(Event.Session(SessionEvent.StartSession(at(0))))
        if (variation != null) {
            bridge.update(
                Event.Session(
                    SessionEvent.SwitchPlay(entryId, null, null, listOf(variation), at(0), silent)
                )
            )
        }
        bridge.update(Event.Session(SessionEvent.NextItem(at(5), at(5), silent)))
        val playId = bridge.view().summary?.entries?.firstOrNull()?.plays?.lastOrNull()?.id
        bridge.update(
            Event.Session(SessionEvent.UpdateEntryScore(entryId, playId.orEmpty(), score.toUByte()))
        )
        val write =
            bridge.update(Event.Session(SessionEvent.SaveSession(at(6)))).single {
                (it.effect as? Effect.Persistence)?.value is PersistenceOperation.SaveSession
            }
        bridge.resolve(write.id, PersistenceOutput.Ack)
    }
}
