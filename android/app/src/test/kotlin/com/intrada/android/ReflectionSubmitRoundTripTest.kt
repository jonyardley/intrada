package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.AppEffect
import com.intrada.shared.ClickState
import com.intrada.shared.DraftMark
import com.intrada.shared.DraftTempo
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.LibraryItemView
import com.intrada.shared.Metre
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.ReflectionAnswers
import com.intrada.shared.Request
import com.intrada.shared.SessionEvent
import com.intrada.shared.TempoBand
import com.intrada.shared.TempoReading
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

// The item-complete sheet's one submit and its tempo rows on Kotlin's own encoder, through the
// real core (#846, #2434).
class ReflectionSubmitRoundTripTest {
    private val silent = TempoReading(bpm = 72.toUShort(), clickSounding = false)
    private val sixEight =
        ClickState(Metre(6.toUByte(), 8.toUByte(), null), sounding = 0b111111.toUShort())

    @Test
    fun theSubmitLandsTheNoteTheMarkAndTheTempo() {
        val bridge = LiveBridge()
        val playId = sheetOpen(bridge, silent)

        bridge.update(
            Event.Session(
                SessionEvent.SubmitReflection(
                    "2026-10-06T10:05:30Z",
                    answers(DraftTempo(playId, 96.toUShort()), note = "Pedal clearer"),
                )
            )
        )

        val entry = bridge.view().summary?.entries?.firstOrNull()
        assertEquals("Pedal clearer", entry?.notes)
        assertEquals(4.toUByte(), entry?.plays?.lastOrNull()?.score)
        assertEquals(96.toUShort(), entry?.plays?.lastOrNull()?.achievedTempo)
    }

    @Test
    fun aRefusedNoteKeepsTheSheetUp() {
        val bridge = LiveBridge()
        val playId = sheetOpen(bridge, silent)
        val before = bridge.view().errorSeq

        bridge.update(
            Event.Session(
                SessionEvent.SubmitReflection(
                    "2026-10-06T10:05:30Z",
                    answers(DraftTempo(playId, 96.toUShort()), note = "a".repeat(5001)),
                )
            )
        )

        assertNotEquals(before, bridge.view().errorSeq)
        assertNotNull(bridge.view().activeSession?.reflection)
    }

    @Test
    fun theOpenSheetSeedsEachRowInTheClicksUnit() {
        val bridge = LiveBridge()
        val playId =
            sheetOpen(bridge, TempoReading(120.toUShort(), clickSounding = true, click = sixEight))

        val row = bridge.view().activeSession?.reflection?.tempos?.single()
        assertEquals(playId, row?.playId)
        assertEquals(120.toUShort(), row?.tempo)
        assertEquals(sixEight, row?.click)
        assertEquals(TempoBand(8.toUByte(), 80.toUShort(), 416.toUShort()), row?.band)
        assertFalse(row?.setByHand ?: true)
    }

    @Test
    fun aRowSetByHandReopensOnThatNumber() {
        val bridge = LiveBridge()
        val playId =
            sheetOpen(bridge, TempoReading(120.toUShort(), clickSounding = true, click = sixEight))

        bridge.update(
            Event.Session(
                SessionEvent.UpdateReflectionDraft(
                    answers(DraftTempo(playId, 150.toUShort()), note = "")
                )
            )
        )

        val row = bridge.view().activeSession?.reflection?.tempos?.single()
        assertEquals(150.toUShort(), row?.tempo)
        assertTrue(row?.setByHand ?: false)
    }

    private fun sheetOpen(bridge: LiveBridge, reading: TempoReading): String {
        val load =
            bridge.update(Event.StartApp).single {
                (it.effect as? Effect.Persistence)?.value == PersistenceOperation.LoadItems
            }
        val rows =
            libraryChanged(bridge.resolve(load.id, PersistenceOutput.Items(Fixtures.library)))
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(rows.first().id)))
        bridge.update(Event.Session(SessionEvent.StartSession("2026-10-06T10:00:00Z")))
        bridge.update(
            Event.Session(SessionEvent.PrepareReflection("2026-10-06T10:05:00Z", reading))
        )
        return bridge.view().activeSession?.entries?.firstOrNull()?.plays?.lastOrNull()?.id.orEmpty()
    }

    private fun answers(tempo: DraftTempo, note: String) =
        ReflectionAnswers(
            marks = listOf(DraftMark(tempo.playId, 4.toUByte())),
            note = note,
            tempos = listOf(tempo),
            gotInTheWay = emptyList(),
            notePoints = emptyList(),
            ways = emptyList(),
        )

    private fun libraryChanged(requests: List<Request>): List<LibraryItemView> =
        requests
            .mapNotNull { ((it.effect as? Effect.App)?.value as? AppEffect.LibraryChanged)?.value }
            .single()
}
