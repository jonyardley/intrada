package com.intrada.android.ui

import com.intrada.shared.BuildingSetlistView
import com.intrada.shared.SetlistBlockView
import com.intrada.shared.SetlistEntryView

internal fun summary(setlist: BuildingSetlistView): String {
    val items = setlist.itemCount.toInt()
    var counts = "$items item${if (items == 1) "" else "s"}"
    if (setlist.blocks.any { it.groupId != null }) {
        val n = setlist.blocks.size
        counts = "$n block${if (n == 1) "" else "s"} · $counts"
    }
    return setlist.totalDurationSummary?.let { "$it · $counts" } ?: counts
}

internal fun startTitle(setlist: BuildingSetlistView): String =
    setlist.totalDurationSummary?.let { "Start session · $it" } ?: "Start session"

internal fun relatedLabel(block: SetlistBlockView): String =
    if (block.relatedCount == 0UL) "piece only" else "+${block.relatedCount} related"

// HACK(#1101): the core renders "—" and "0s" for nothing planned; drop them until it sends "".
internal fun durationSuffix(display: String): String =
    if (display.isEmpty() || display == "—" || display == "0s") "" else " · $display"

internal fun headerSubtitle(block: SetlistBlockView, collapsed: Boolean): String? {
    val suffix = durationSuffix(block.durationDisplay)
    val n = block.relatedCount
    return when {
        collapsed -> block.durationDisplay.takeIf { suffix.isNotEmpty() }
        n == 0UL -> if (suffix.isEmpty()) "Piece only" else block.durationDisplay
        else -> "$n related, then piece$suffix"
    }
}

internal fun planTags(entry: SetlistEntryView, setlist: BuildingSetlistView): List<String> {
    val segments = entry.record.segments
    val split = segments.size > 1
    val variations =
        setlist.entryVariations.firstOrNull { it.entryId == entry.id }?.variations.orEmpty()
    return segments.map {
        if (split && it.plannedSecs > 0u) "${it.label} ${it.plannedDisplay}" else it.label
    } +
        entry.plannedVariationIds.mapNotNull { id ->
            variations.firstOrNull { it.id == id }?.label
        } +
        listOfNotNull(entry.record.focus?.let { "Focus: ${it.label}" })
}
