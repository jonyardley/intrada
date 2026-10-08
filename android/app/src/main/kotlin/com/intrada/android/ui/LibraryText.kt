package com.intrada.android.ui

import com.intrada.shared.LinkedExerciseView

// ── Display formatting ──

internal val LinkedExerciseView.metaLine: String?
    get() = listOfNotNull(keyLabel, tempoLine).takeIf { it.isNotEmpty() }?.joinToString(" · ")

internal val LinkedExerciseView.spoken: String
    get() =
        listOfNotNull(
                "Exercise",
                title,
                listOfNotNull(keyLabel, tempoLineSpoken)
                    .takeIf { it.isNotEmpty() }
                    ?.joinToString(", "),
                linkCaption,
            )
            .joinToString(", ")
