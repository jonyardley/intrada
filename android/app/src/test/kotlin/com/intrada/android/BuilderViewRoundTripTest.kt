package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.CreateItem
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.FocusKind
import com.intrada.shared.IntentionFocus
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.SessionEvent
import org.junit.Assert.assertEquals
import org.junit.Test

// The builder's lines and the stepper's minutes arrive ready on Kotlin's decoder, so neither
// phone writes its own, and the aim is kept trimmed (#2467).
class BuilderViewRoundTripTest {
    @Test
    fun theBuildersSummaryCaptionTagsAndMinutesArriveReady() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val nocturne = savedItem(bridge, Event.Item(ItemEvent.Add(piece)))
        bridge.update(
            Event.Item(ItemEvent.UpdateItemVariations(nocturne.id, emptyList(), listOf("Dotted")))
        )
        val dotted = bridge.view().variations.single { it.label == "Dotted" }.id
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne.id)))
        val entryId = bridge.view().buildingSetlist?.entries?.firstOrNull()?.id.orEmpty()
        bridge.update(Event.Session(SessionEvent.SetEntryDuration(entryId, 600u)))
        bridge.update(Event.Session(SessionEvent.SetEntryVariations(entryId, listOf(dotted))))
        bridge.update(
            Event.Session(SessionEvent.SetFocus(entryId, IntentionFocus(FocusKind.FROMMEMORY)))
        )
        bridge.update(Event.Session(SessionEvent.SetEntryIntention(entryId, "  Even quavers \n")))

        val building = bridge.view().buildingSetlist
        val entry = building?.entries?.firstOrNull()
        assertEquals(listOf("Dotted", "Focus: From memory"), entry?.planTags)
        assertEquals("Even quavers", entry?.intention)
        val block = building?.blocks?.firstOrNull()
        assertEquals("Piece · 10 min", block?.caption)
        assertEquals("10 min", block?.plannedDisplay)
        assertEquals("10 min", block?.durationDisplay)
        assertEquals("piece only", block?.relatedLabel)
        assertEquals("10 min · 1 item", building?.summary)
        assertEquals("Start session · 10 min", building?.startTitle)
        val limits = bridge.view().limits
        assertEquals(
            listOf(1, 60, 6),
            listOf(
                    limits.plannedDurationMinMins,
                    limits.plannedDurationMaxMins,
                    limits.plannedDurationDefaultMins,
                )
                .map { it.toInt() },
        )

        bridge.update(Event.Session(SessionEvent.SetEntryIntention(entryId, "   ")))
        assertEquals(null, bridge.view().buildingSetlist?.entries?.firstOrNull()?.intention)
    }

    private val piece =
        CreateItem(
            title = "Nocturne",
            kind = ItemKind.PIECE,
            tags = emptyList(),
            variationLabels = emptyList(),
        )

    private fun savedItem(bridge: LiveBridge, event: Event) =
        bridge
            .update(event)
            .mapNotNull {
                ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)?.value
            }
            .single()
}
