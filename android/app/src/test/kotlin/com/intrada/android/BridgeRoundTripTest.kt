package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.android.core.withIds
import com.intrada.shared.AppEffect
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.LibraryItemView
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.Request
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// Through the host build of the real core, never a stub: a stub cannot catch a decode that skews
// from the Rust side (#846).
class BridgeRoundTripTest {
    @Test
    fun itemsEncodedInKotlinComeBackAsLibraryRows() {
        val bridge = LiveBridge()
        val load =
            bridge.update(Event.StartApp).single {
                (it.effect as? Effect.Persistence)?.value == PersistenceOperation.LoadItems
            }

        val requests = bridge.resolve(load.id, PersistenceOutput.Items(Fixtures.library))

        val rows = libraryChanged(requests)
        val view = bridge.view()
        val shown = rows.withIds(view.visibleIds)
        assertEquals(
            setOf("Clair de Lune", "Gymnopédie No. 1", "Hanon No. 1"),
            shown.map { it.title }.toSet(),
        )
        val satie = shown.single { it.title == "Gymnopédie No. 1" }
        assertEquals("Erik Satie", satie.subtitle)
        assertEquals("D major", satie.key)
        assertEquals("Lent", satie.tempoMarking)
        assertEquals(70.toUShort(), satie.tempoBpm)
        assertEquals(listOf("recital"), satie.tags)
        assertEquals(2uL, view.visiblePieces)
        assertEquals(1uL, view.visibleExercises)
    }

    @Test
    fun theCoreSampleSetDecodes() {
        val bridge = LiveBridge()

        val rows = libraryChanged(bridge.update(Event.LoadSampleData))

        val view = bridge.view()
        assertTrue(rows.any { it.title == "Clair de Lune" && it.subtitle == "Claude Debussy" })
        assertEquals(rows.size, view.visibleIds.size)
    }

    private fun libraryChanged(requests: List<Request>): List<LibraryItemView> =
        requests
            .mapNotNull { ((it.effect as? Effect.App)?.value as? AppEffect.LibraryChanged)?.value }
            .single()
}
