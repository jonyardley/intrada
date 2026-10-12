package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.ffi.declaredTempoWords
import com.intrada.shared.AppEffect
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
import org.junit.Assert.assertEquals
import org.junit.Test

// A section's target and a bare declared tempo, worded by the core on Kotlin's decoder (#846,
// #2486).
class TempoWordsRoundTripTest {
    @Test
    fun aSectionsTargetCarriesItsWords() {
        val bridge = LiveBridge()
        val piece =
            bridge
                .update(
                    Event.Item(
                        ItemEvent.Add(
                            CreateItem(
                                title = "Rondo",
                                kind = ItemKind.PIECE,
                                tags = emptyList(),
                                variationLabels = emptyList(),
                            )
                        )
                    )
                )
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)
                        ?.value
                }
                .single()
                .id
        val edit = SectionEdit(null, "", BarsInput.Typed("12-14"), SectionKind.TROUBLESPOT, "60")
        val section =
            bridge
                .update(Event.Item(ItemEvent.ChangeSection(piece, SectionChange.Save(edit))))
                .mapNotNull {
                    ((it.effect as? Effect.App)?.value as? AppEffect.LibraryChanged)?.value
                }
                .single()
                .single { it.id == piece }
                .sections
                .single()
        assertEquals("♩ 60", section.targetTempo)
        assertEquals("60 beats per minute", section.targetTempoSpoken)
    }

    @Test
    fun aBareDeclaredTempoComesFromTheCore() {
        val words = declaredTempoWords(66u)
        assertEquals("♩ = 66", words.text)
        assertEquals("66 beats per minute", words.spoken)
    }
}
