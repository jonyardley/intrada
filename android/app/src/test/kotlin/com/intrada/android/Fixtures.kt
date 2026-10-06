package com.intrada.android

import com.intrada.android.core.ItemStore
import com.intrada.shared.Accidental
import com.intrada.shared.CreateItem
import com.intrada.shared.ExerciseLink
import com.intrada.shared.Item
import com.intrada.shared.ItemKind
import com.intrada.shared.Key
import com.intrada.shared.Letter
import com.intrada.shared.Modality
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.Tempo

object Fixtures {
    fun item(
        id: String = "01J0000000000000000000ITEM",
        title: String = "Clair de Lune",
        kind: ItemKind = ItemKind.PIECE,
        composer: String? = "Claude Debussy",
        key: Key? = null,
        tempo: Tempo? = null,
        tags: List<String> = emptyList(),
        exerciseLinks: List<ExerciseLink> = emptyList(),
    ) =
        Item(
            id = id,
            title = title,
            kind = kind,
            composer = composer,
            key = key,
            tempo = tempo,
            tags = tags,
            createdAt = "2026-09-01T09:00:00Z",
            updatedAt = "2026-09-01T09:00:00Z",
            priority = false,
            sections = emptyList(),
            variationIds = emptyList(),
            keys = emptyList(),
            exerciseLinks = exerciseLinks,
        )

    val library =
        listOf(
            item(),
            item(
                id = "01J0000000000000000000SATI",
                title = "Gymnopédie No. 1",
                composer = "Erik Satie",
                key = Key(Letter.D, Accidental.NATURAL, Modality.MAJOR),
                tempo = Tempo("Lent", 70u),
                tags = listOf("recital"),
            ),
            item(
                id = "01J0000000000000000000HANO",
                title = "Hanon No. 1",
                kind = ItemKind.EXERCISE,
                composer = null,
            ),
        )

    val scales =
        CreateItem(
            title = "Scales in thirds",
            kind = ItemKind.EXERCISE,
            tags = listOf("warm-up"),
            variationLabels = listOf("Slow", "Swung"),
        )

    object FailingItemStore : ItemStore {
        override fun run(operation: PersistenceOperation): PersistenceOutput =
            error("disk unavailable")
    }
}
