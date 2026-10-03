package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.android.core.withIds
import com.intrada.shared.AppEffect
import com.intrada.shared.ClickBarOption
import com.intrada.shared.ClickPreset
import com.intrada.shared.ClickPresetOption
import com.intrada.shared.CreateItem
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.LibraryItemView
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.Request
import com.intrada.shared.TempoBand
import com.intrada.shared.TempoInput
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

    // A second decoder of the same wire: a skewed TempoInput shows only here (#846, #2224).
    @Test
    fun aTypedBpmCrossesAsTextAndLandsAsANumber() {
        val bridge = LiveBridge()

        val saved =
            bridge
                .update(
                    Event.Item(
                        ItemEvent.Add(
                            CreateItem(
                                title = "Nocturne",
                                kind = ItemKind.PIECE,
                                tempo = TempoInput(marking = "Lento", bpm = " 60 "),
                                tags = emptyList(),
                                variantLabels = emptyList(),
                            )
                        )
                    )
                )
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)
                        ?.value
                }
                .single()

        assertEquals("Lento", saved.tempo?.marking)
        assertEquals(60.toUShort(), saved.tempo?.bpm)
    }

    // The click's band and bars sit mid-ViewModel, so a skew here also garbles the fields after
    // them (#2225).
    @Test
    fun theClicksBandAndBarsDecode() {
        val limits = LiveBridge().view().limits

        assertEquals(
            TempoBand(unit = 8.toUByte(), min = 80.toUShort(), max = 416.toUShort()),
            limits.clickTempoBands.single { it.unit == 8.toUByte() },
        )
        val groups = listOf(3, 2, 2).map(Int::toUByte)
        assertEquals(
            ClickBarOption(
                beats = 7.toUByte(),
                groups = groups,
                presets =
                    listOf(
                        ClickPresetOption(ClickPreset.EVERYBEAT, 0b1111111.toUShort()),
                        ClickPresetOption(ClickPreset.GROUPSTARTS, 0b0101001.toUShort()),
                        ClickPresetOption(ClickPreset.DOWNBEAT, 1.toUShort()),
                    ),
            ),
            limits.clickBars.single { it.beats == 7.toUByte() && it.groups == groups },
        )
    }

    private fun libraryChanged(requests: List<Request>): List<LibraryItemView> =
        requests
            .mapNotNull { ((it.effect as? Effect.App)?.value as? AppEffect.LibraryChanged)?.value }
            .single()
}
