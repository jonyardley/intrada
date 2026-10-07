package com.intrada.android

import com.intrada.android.core.LiveBridge
import com.intrada.shared.Accidental
import com.intrada.shared.CreateItem
import com.intrada.shared.Effect
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.Key
import com.intrada.shared.KeyEdit
import com.intrada.shared.Letter
import com.intrada.shared.Modality
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.Request
import com.intrada.shared.UpdateItem
import org.junit.Assert.assertEquals
import org.junit.Test

class ItemEditRoundTripTest {
    // The edit form's one save crosses a second decoder: the fields, the variation ids and the
    // typed labels all land in one item write (#2228).
    @Test
    fun anEditSavesTheFieldsAndTheVariationsInOneWrite() {
        val bridge = LiveBridge()
        val added =
            bridge.update(
                Event.Item(
                    ItemEvent.Add(
                        CreateItem(
                            title = "Scales",
                            kind = ItemKind.EXERCISE,
                            tags = emptyList(),
                            variationLabels = listOf("Slow", "Swung"),
                        )
                    )
                )
            )
        val item =
            added
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)
                        ?.value
                }
                .single()
        val dMinor = Key(Letter.D, Accidental.NATURAL, Modality.MINOR)

        val effects =
            bridge.update(
                Event.Item(
                    ItemEvent.Edit(
                        item.id,
                        UpdateItem(title = "Scales in thirds", key = KeyEdit.Set(dMinor)),
                        listOf(item.variationIds.last()),
                        listOf("Staccato"),
                    )
                )
            )

        val saved =
            effects
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)
                        ?.value
                }
                .single()
        val minted =
            effects
                .mapNotNull {
                    ((it.effect as? Effect.Persistence)?.value
                            as? PersistenceOperation.SaveVariations)
                        ?.value
                }
                .flatten()
        assertEquals("Scales in thirds", saved.title)
        assertEquals(dMinor, saved.key)
        assertEquals(listOf("Staccato"), minted.map { it.label })
        assertEquals(listOf(item.variationIds.last(), minted.single().id), saved.variationIds)
    }

    // Kotlin's binding flattens a nested optional, so Keep and Clear need the enum (#2461).
    @Test
    fun anEditKeepsOrClearsTheKey() {
        val bridge = LiveBridge()
        val dMinor = Key(Letter.D, Accidental.NATURAL, Modality.MINOR)
        val item =
            savedItems(
                    bridge.update(
                        Event.Item(
                            ItemEvent.Add(
                                CreateItem(
                                    title = "Scales",
                                    kind = ItemKind.PIECE,
                                    key = dMinor,
                                    tags = emptyList(),
                                    variationLabels = emptyList(),
                                )
                            )
                        )
                    )
                )
                .single()
        fun edit(key: KeyEdit) =
            savedItems(
                    bridge.update(
                        Event.Item(
                            ItemEvent.Edit(item.id, UpdateItem(key = key), emptyList(), emptyList())
                        )
                    )
                )
                .single()

        assertEquals(dMinor, edit(KeyEdit.Keep).key)
        assertEquals(null, edit(KeyEdit.Clear).key)
    }

    private fun savedItems(effects: List<Request>) = effects.mapNotNull {
        ((it.effect as? Effect.Persistence)?.value as? PersistenceOperation.SaveItem)?.value
    }
}
