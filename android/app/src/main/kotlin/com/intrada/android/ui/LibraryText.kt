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

// The note value one click stands for: `♩ = 168` in 7/8 would be a lie the ear catches (T19).
// The minim is spelt out, since the bundled faces have no glyph for it.
internal fun tempoReadout(bpm: Int, unit: UByte): String =
    when (unit) {
        MINIM -> "minim = $bpm"
        QUAVER -> "♪ = $bpm"
        else -> "♩ = $bpm"
    }

internal fun tempoReadoutSpoken(bpm: Int, unit: UByte): String =
    when (unit) {
        MINIM -> "$bpm minim beats per minute"
        QUAVER -> "$bpm quaver beats per minute"
        else -> "$bpm beats per minute"
    }

private const val MINIM: UByte = 2u
private const val QUAVER: UByte = 8u
