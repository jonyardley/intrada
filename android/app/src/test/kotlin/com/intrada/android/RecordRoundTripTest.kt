package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.Accidental
import com.intrada.shared.AppEffect
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.Key
import com.intrada.shared.Letter
import com.intrada.shared.LibraryItemView
import com.intrada.shared.Modality
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.Request
import com.intrada.shared.SessionEvent
import com.intrada.shared.TempoReading
import org.junit.Assert.assertEquals
import org.junit.Test

// The v0.17 record's keys on Kotlin's own encoder, through the real core (#846, #2249).
class RecordRoundTripTest {
    // A planned key opens the first play, and the finish sheet's confirmed key replaces it, on
    // Kotlin's own encoder both ways (#846, #2249).
    @Test
    fun aPlannedKeyOpensThePlayAndTheConfirmedKeyLands() {
        val bridge = LiveBridge()
        val load =
            bridge.update(Event.StartApp).single {
                (it.effect as? Effect.Persistence)?.value == PersistenceOperation.LoadItems
            }
        val rows =
            libraryChanged(bridge.resolve(load.id, PersistenceOutput.Items(Fixtures.library)))
        bridge.update(Event.Session(SessionEvent.StartBuilding))
        bridge.update(Event.Session(SessionEvent.AddToSetlist(rows.first().id)))
        val entryId = bridge.view().buildingSetlist?.entries?.firstOrNull()?.id.orEmpty()
        val bFlatMinor = Key(Letter.B, Accidental.FLAT, Modality.MINOR)
        val gSharpMinor = Key(Letter.G, Accidental.SHARP, Modality.MINOR)
        val reading = TempoReading(bpm = 72.toUShort(), clickSounding = false)

        bridge.update(Event.Session(SessionEvent.SetEntryKey(entryId, bFlatMinor)))
        assertEquals(bFlatMinor, bridge.view().buildingSetlist?.entries?.firstOrNull()?.plannedKey)
        bridge.update(Event.Session(SessionEvent.StartSession("2026-10-05T09:00:00Z")))
        val play = bridge.view().activeSession?.entries?.firstOrNull()?.plays?.firstOrNull()
        assertEquals(bFlatMinor, play?.key)

        bridge.update(
            Event.Session(SessionEvent.PrepareReflection("2026-10-05T09:05:00Z", reading))
        )
        bridge.update(
            Event.Session(
                SessionEvent.NextItem("2026-10-05T09:05:00Z", "2026-10-05T09:05:00Z", reading)
            )
        )
        bridge.update(
            Event.Session(
                SessionEvent.UpdatePlayWay(
                    entryId,
                    play?.id.orEmpty(),
                    null,
                    gSharpMinor,
                    emptyList(),
                )
            )
        )

        assertEquals(
            gSharpMinor,
            bridge.view().summary?.entries?.firstOrNull()?.plays?.firstOrNull()?.key,
        )
    }

    private fun libraryChanged(requests: List<Request>): List<LibraryItemView> =
        requests
            .mapNotNull { ((it.effect as? Effect.App)?.value as? AppEffect.LibraryChanged)?.value }
            .single()
}
