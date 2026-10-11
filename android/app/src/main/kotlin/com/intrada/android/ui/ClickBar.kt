package com.intrada.android.ui

import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.intrada.shared.ActiveSessionView
import com.intrada.shared.ClickPreset
import com.intrada.shared.ClickPresetOption
import com.intrada.shared.LimitsView
import com.intrada.shared.Metre

/**
 * The click's bar and which of its beats sound. A session-local override: the item keeps its own
 * metre. [changed] runs after every change the musician makes, so the click can follow it.
 */
@Stable
class ClickBar(private val changed: () -> Unit) {
    var metre by mutableStateOf(Metre(4u, 4u))
        private set

    var sounding: UShort by mutableStateOf(0b1111u)
        private set

    private var limits: LimitsView? = null
    // The item's own bar can hold a grouping the core's table does not list (4/8 as 2 + 2), so its
    // patterns come with the seed (#2225).
    private var seedMetre: Metre? = null
    private var seedPresets: List<ClickPresetOption> = emptyList()

    val presets: List<ClickPresetOption>
        get() {
            val seed = seedMetre
            if (seed != null && seed.beats == metre.beats && seed.groups == metre.groups) {
                return seedPresets
            }
            return limits?.clickPresets(metre).orEmpty()
        }

    val matchingPreset: ClickPreset?
        get() = presets.firstOrNull { it.sounding == sounding }?.preset

    /** Bar and pattern open on the core's answer for the item (T19, #2225). */
    fun reseed(active: ActiveSessionView, limits: LimitsView) {
        this.limits = limits
        metre = active.clickSeedMetre
        seedMetre = active.clickSeedMetre
        seedPresets = active.clickSeedPresets
        sounding = active.currentClickSounding
    }

    fun choose(next: Metre) {
        if (next == metre) return
        metre = next
        sounding = presets.firstOrNull { it.preset == ClickPreset.EVERYBEAT }?.sounding ?: 1u
        changed()
    }

    fun apply(preset: ClickPreset) {
        presets.firstOrNull { it.preset == preset }?.let { sound(it.sounding) }
    }

    /** The last sounding beat cannot be silenced: a click that sounds nothing is not a click. */
    fun toggleBeat(index: Int) = sound((sounding.toInt() xor (1 shl index)).toUShort())

    private fun sound(next: UShort) {
        if (next == sounding || next.toInt() == 0) return
        sounding = next
        changed()
    }
}
