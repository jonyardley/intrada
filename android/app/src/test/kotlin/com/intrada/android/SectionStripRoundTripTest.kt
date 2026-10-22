package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.BarsInput
import com.intrada.shared.CreateItem
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.SectionChange
import com.intrada.shared.SectionEdit
import com.intrada.shared.SectionKind
import com.intrada.shared.Segment
import com.intrada.shared.SegmentProgress
import com.intrada.shared.SessionEvent
import com.intrada.shared.TempoReading
import org.junit.Assert.assertEquals
import org.junit.Test

// The practice screen's strip of sections arrives ready on Kotlin's decoder (#846, #2394).
class SectionStripRoundTripTest {
    @Test
    fun theStripShowsWhatStayTookAndFillsOnceASectionIsDone() {
        val bridge = LiveBridge()
        bridge.update(Event.StartApp)
        val nocturne = savedItem(bridge, Event.Item(ItemEvent.Add(piece)))
        val sectionIds =
            listOf("A", "B")
                .map { name ->
                    val edit = SectionEdit(null, name, BarsInput.Blank, SectionKind.FORM, "")
                    savedItem(
                        bridge,
                        Event.Item(ItemEvent.ChangeSection(nocturne.id, SectionChange.Save(edit))),
                    )
                }
                .last()
                .sections
                .map { it.id }
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(nocturne.id)))
        val entryId = bridge.view().buildingSetlist?.entries?.firstOrNull()?.id.orEmpty()
        bridge.update(Event.Session(SessionEvent.SetEntryDuration(entryId, 1200u)))
        bridge.update(
            Event.Session(SessionEvent.SetSegments(entryId, sectionIds.map { Segment(it, 0u) }))
        )
        bridge.update(Event.Session(SessionEvent.StartSession("2026-10-04T09:00:00Z")))

        bridge.update(Event.Session(SessionEvent.StayOnSegment))
        val stayed = bridge.view().activeSession?.record?.segment
        assertEquals(listOf("A", "B"), stayed?.bars?.map { it.label })
        assertEquals(
            listOf(SegmentProgress.RUNNING, SegmentProgress.TOCOME),
            stayed?.bars?.map { it.progress },
        )
        assertEquals(listOf(null, "A took 2 min"), stayed?.bars?.map { it.caption })
        assertEquals("Section 1 of 2", stayed?.positionLabel)

        bridge.update(
            Event.Session(
                SessionEvent.MoveToNextSegment(
                    "2026-10-04T09:12:00Z",
                    TempoReading(bpm = 84.toUShort(), clickSounding = false),
                )
            )
        )
        val moved = bridge.view().activeSession?.record?.segment
        assertEquals(
            listOf(SegmentProgress.DONE, SegmentProgress.RUNNING),
            moved?.bars?.map { it.progress },
        )
        assertEquals("Section 2 of 2", moved?.positionLabel)
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
