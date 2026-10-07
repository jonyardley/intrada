package com.intrada.android.ui

import com.intrada.shared.LibraryItemView
import com.intrada.shared.LinkedExerciseView

// ── Display formatting ──

internal fun tempoDisplay(marking: String?, bpm: UShort?): String? =
    listOfNotNull(marking?.takeIf { it.isNotEmpty() }, bpm?.let { "♩ = $it" })
        .takeIf { it.isNotEmpty() }
        ?.joinToString(" · ")

internal fun tempoSpoken(marking: String?, bpm: UShort?): String? =
    listOfNotNull(marking?.takeIf { it.isNotEmpty() }, bpm?.let { "$it beats per minute" })
        .takeIf { it.isNotEmpty() }
        ?.joinToString(", ")

internal fun tempoDisplay(item: LibraryItemView): String? =
    tempoDisplay(item.tempoMarking, item.tempoBpm)

internal val LinkedExerciseView.metaLine: String?
    get() =
        listOfNotNull(keyLabel, tempoDisplay(tempoMarking, tempoBpm))
            .takeIf { it.isNotEmpty() }
            ?.joinToString(" · ")

internal val LinkedExerciseView.spoken: String
    get() =
        listOfNotNull(
                "Exercise",
                title,
                listOfNotNull(keyLabel, tempoSpoken(tempoMarking, tempoBpm))
                    .takeIf { it.isNotEmpty() }
                    ?.joinToString(", "),
                linkCaption,
            )
            .joinToString(", ")
