package com.intrada.android.ui

import com.intrada.ffi.clickTempoWords
import com.intrada.shared.CreateItem
import com.intrada.shared.ItemKind
import com.intrada.shared.Key
import com.intrada.shared.ScaffoldEntry
import com.intrada.shared.TempoInput
import java.util.UUID

/**
 * An exercise the add form will save with the piece: one written here, or one ticked from the
 * library.
 */
sealed interface StagedExercise {
    val id: String
    val title: String
    val meta: String?
    val entry: ScaffoldEntry

    class Written(
        override val title: String,
        val key: Key?,
        val bpm: String,
        override val id: String = UUID.randomUUID().toString(),
    ) : StagedExercise {
        override val meta: String?
            get() {
                val tempo = bpm.trim().toUShortOrNull()?.let { clickTempoWords(it, 4u).text }
                return listOfNotNull(key?.let(::keyDisplay), tempo).joinToString(" · ").ifEmpty {
                    null
                }
            }

        override val entry: ScaffoldEntry
            get() =
                ScaffoldEntry.New(
                    CreateItem(
                        title = title.trim(),
                        kind = ItemKind.EXERCISE,
                        key = key,
                        tempo = TempoInput(null, bpm.trim()),
                        tags = emptyList(),
                        variationLabels = emptyList(),
                    )
                )
    }

    class Chosen(override val id: String, override val title: String, override val meta: String?) :
        StagedExercise {
        override val entry: ScaffoldEntry
            get() = ScaffoldEntry.Existing(id)
    }
}
