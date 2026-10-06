package com.intrada.android.ui

import com.intrada.shared.LibraryItemView
import com.intrada.shared.LinkedExerciseView
import com.intrada.shared.LinkedSectionView

// ── Formatting the core's values for display, as `LibraryItemView+Display.swift` does on iOS ──

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

internal fun sectionLinkCaption(wholePiece: Boolean, sections: List<LinkedSectionView>): String? {
    val parts =
        (if (wholePiece) listOf("the whole piece") else emptyList()) +
            sections.map { it.labelInText }
    return when {
        sections.isEmpty() -> null
        parts.size == 1 -> "For ${parts[0]}"
        else -> "For " + parts.dropLast(1).joinToString(", ") + " and " + parts.last()
    }
}

internal val LinkedExerciseView.sectionsCaption: String?
    get() = sectionLinkCaption(wholePiece, sections)

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
                sectionsCaption,
            )
            .joinToString(", ")
