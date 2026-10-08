package com.intrada.android.ui

import com.intrada.shared.ClickPreset
import com.intrada.shared.ClickPresetOption
import com.intrada.shared.LimitsView
import com.intrada.shared.Metre

// ── Lookups on the core's click band and bars; the rules live in `domain/metre.rs` (#2225) ──

/** Counted in the bar's own unit: a 6/8 piece at quaver = 240 is inside the quaver band (#1499). */
internal fun LimitsView.clickBand(unit: UByte): IntRange {
    val band = clickTempoBands.firstOrNull { it.unit == unit }
    val default = clickTempoDefault.toInt()
    return band?.let { it.min.toInt()..it.max.toInt() } ?: default..default
}

internal fun LimitsView.clampClickTempo(value: Int, unit: UByte): Int =
    value.coerceIn(clickBand(unit))

internal fun LimitsView.clickPresets(metre: Metre): List<ClickPresetOption> =
    clickBars
        .firstOrNull { it.beats == metre.beats && it.groups == metre.groups }
        ?.presets
        .orEmpty()

internal fun LimitsView.clickGroupings(beats: UByte): List<List<UByte>> = clickBars.mapNotNull {
    if (it.beats == beats) it.groups else null
}

internal val ClickPreset.title: String
    get() =
        when (this) {
            ClickPreset.EVERYBEAT -> "Every beat"
            ClickPreset.GROUPSTARTS -> "Group starts"
            ClickPreset.DOWNBEAT -> "Downbeat"
            ClickPreset.BACKBEAT -> "2 and 4"
        }

internal val Metre.label: String
    get() = "$beats/$unit"

internal fun spokenUnit(unit: UByte): String =
    when (unit.toInt()) {
        MINIM -> "minim"
        QUAVER -> "quaver"
        else -> "crotchet"
    }

private const val MINIM = 2
private const val QUAVER = 8
