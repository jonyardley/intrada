package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.BarsInput
import com.intrada.shared.CreateItem
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.FocusKind
import com.intrada.shared.IntentionFocus
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.SectionChange
import com.intrada.shared.SectionEdit
import com.intrada.shared.SectionKind
import com.intrada.shared.Segment
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
        val sections =
            listOf("A", "B").map { name ->
                val edit = SectionEdit(null, name, BarsInput.Blank, SectionKind.FORM, "")
                savedItem(
                    bridge,
                    Event.Item(ItemEvent.ChangeSection(nocturne.id, SectionChange.Save(edit))),
                )
            }
        val sectionIds = sections.last().sections.map { it.id }
        val dotted = bridge.view().variations.single { it.label == "Dotted" }.id
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne.id)))
        val entryId = bridge.view().buildingSetlist?.entries?.firstOrNull()?.id.orEmpty()
        bridge.update(Event.Session(SessionEvent.SetEntryDuration(entryId, 600u)))
        bridge.update(
            Event.Session(SessionEvent.SetSegments(entryId, sectionIds.map { Segment(it, 0u) }))
        )
        bridge.update(Event.Session(SessionEvent.StepSegment(entryId, sectionIds.first(), 1)))
        bridge.update(Event.Session(SessionEvent.SetEntryVariations(entryId, listOf(dotted))))
        bridge.update(
            Event.Session(
                SessionEvent.SetFocus(entryId, IntentionFocus(FocusKind.FROMMEMORY, null, null))
            )
        )
        bridge.update(Event.Session(SessionEvent.SetEntryIntention(entryId, "  Even quavers \n")))

        val building = bridge.view().buildingSetlist
        val entry = building?.entries?.firstOrNull()
        assertEquals(listOf("A 6 min", "B 4 min", "Dotted", "Focus: From memory"), entry?.planTags)
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
