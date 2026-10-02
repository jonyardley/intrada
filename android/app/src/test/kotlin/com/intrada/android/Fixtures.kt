package com.intrada.android

import com.intrada.android.core.ItemStore
import com.intrada.shared.Item
import com.intrada.shared.ItemKind
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.Tempo

object Fixtures {
    fun item(
        id: String = "01J0000000000000000000ITEM",
        title: String = "Clair de Lune",
        kind: ItemKind = ItemKind.PIECE,
        composer: String? = "Claude Debussy",
        key: String? = null,
        tempo: Tempo? = null,
        tags: List<String> = emptyList(),
    ) =
        Item(
            id = id,
            title = title,
            kind = kind,
            composer = composer,
            key = key,
            tempo = tempo,
            tags = tags,
            linkedExerciseIds = emptyList(),
            createdAt = "2026-09-01T09:00:00Z",
            updatedAt = "2026-09-01T09:00:00Z",
            priority = false,
            variants = emptyList(),
        )

    val library =
        listOf(
            item(),
            item(
                id = "01J0000000000000000000SATI",
                title = "Gymnopédie No. 1",
                composer = "Erik Satie",
                key = "D major",
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

    object FailingItemStore : ItemStore {
        override fun run(operation: PersistenceOperation): PersistenceOutput =
            error("disk unavailable")
    }
}
