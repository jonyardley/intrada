package com.intrada.android.ui

import android.os.Handler
import android.os.Looper
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.intrada.android.core.ClickOutput
import com.intrada.shared.ActiveSessionView
import com.intrada.shared.ClickState
import com.intrada.shared.LimitsView
import com.intrada.shared.Metre
import com.intrada.shared.TempoReading

/**
 * The player's click. Tempo, bar and pattern are session state that never writes back to the item;
 * [reading] reports them and the core rules on what they evidence (#1499).
 */
@Stable
class ClickController(private val makeOutput: () -> ClickOutput) {
    var isRunning by mutableStateOf(false)
        private set

    var bpm by mutableIntStateOf(0)
        private set

    var metre by mutableStateOf(Metre(4u, 4u))
        private set

    /** Set only when the audio refused to start; a pulse another app took is not a fault. */
    var unavailable by mutableStateOf(false)
        private set

    /** False when the item declares no BPM, or one the band moved (#1942). */
    var soundsTarget by mutableStateOf(false)
        private set

    private var sounding: UShort = 0b1111u
    private var seeded = 0
    private var seededUnit: UByte = 4u
    private var configured = false
    private var limits: LimitsView? = null
    private var output: ClickOutput? = null
    private val main = Handler(Looper.getMainLooper())
    private val backgroundStop = Runnable { stop() }

    /** A bar change to another beat unit re-reads the same number as a different tempo (#1942). */
    val isAtSeededTempo: Boolean
        get() = bpm == seeded && metre.unit == seededUnit

    val step: Int
        get() = limits?.clickTempoStep?.toInt() ?: 0

    /** Nil until the click starts: the core reads `Some` as a bar the musician chose (#1499). */
    val status: ClickStatus
        get() =
            when {
                isRunning -> ClickStatus.RUNNING
                unavailable -> ClickStatus.UNAVAILABLE
                else -> ClickStatus.STOPPED
            }

    val reading: TempoReading
        get() =
            TempoReading(
                bpm.toUShort(),
                isRunning,
                if (configured) ClickState(metre, sounding) else null,
            )

    /** Silences the click: its tempo belonged to the item that just finished (#2225). */
    fun reseed(active: ActiveSessionView, limits: LimitsView) {
        stop()
        unavailable = false
        this.limits = limits
        metre = active.clickSeedMetre
        seeded = active.clickSeedBpm.toInt()
        seededUnit = metre.unit
        soundsTarget = active.clickSeedSoundsTarget
        bpm = seeded
        sounding = active.currentClickSounding
        configured = false
    }

    fun toggle() = if (isRunning) stop() else start()

    fun step(by: Int) {
        val stepped = clamped(bpm + by)
        if (stepped == bpm) return
        bpm = stepped
        if (isRunning) start()
    }

    fun stop() {
        main.removeCallbacks(backgroundStop)
        output?.stop()
        isRunning = false
    }

    /** A phone left locked would otherwise click until the battery goes (#1399). */
    fun enteredBackground() {
        main.removeCallbacks(backgroundStop)
        main.postDelayed(backgroundStop, BACKGROUND_GRACE_MILLIS)
    }

    fun enteredForeground() = main.removeCallbacks(backgroundStop)

    private fun start() {
        val out =
            output
                ?: makeOutput().also {
                    it.onPulseDied = { isRunning = false }
                    output = it
                }
        val started = out.start(bpm, metre.beats.toInt(), sounding.toInt())
        isRunning = started
        unavailable = !started
        if (started) configured = true
    }

    // The core's band in the bar's own unit, looked up as iOS's `ClickLimits` does (#2225).
    private fun clamped(value: Int): Int {
        val band = limits?.clickTempoBands?.firstOrNull { it.unit == metre.unit } ?: return value
        return value.coerceIn(band.min.toInt(), band.max.toInt())
    }

    private companion object {
        const val BACKGROUND_GRACE_MILLIS = 600_000L
    }
}

enum class ClickStatus {
    STOPPED,
    RUNNING,
    UNAVAILABLE,
}

class ClickRowState(
    val bpm: Int,
    val unit: UByte,
    val status: ClickStatus,
    val atSeededTempo: Boolean,
    /** The item's declared tempo; nil when the row names the click instead (#1942). */
    val target: String?,
    val targetSpoken: String?,
) {
    val isRunning: Boolean
        get() = status == ClickStatus.RUNNING

    val unavailable: Boolean
        get() = status == ClickStatus.UNAVAILABLE

    /** Once stepped off the seed, the row reads as the click, never a tempo it would not play. */
    private val showsBpm: Boolean
        get() = isRunning || !atSeededTempo

    val readout: String
        get() =
            when {
                unavailable -> "Metronome unavailable"
                showsBpm -> tempoReadout(bpm, unit)
                else -> target ?: "Metronome"
            }

    val spoken: String
        get() =
            when {
                unavailable -> "unavailable"
                showsBpm -> tempoReadoutSpoken(bpm, unit)
                else -> targetSpoken ?: tempoReadoutSpoken(bpm, unit)
            }
}
