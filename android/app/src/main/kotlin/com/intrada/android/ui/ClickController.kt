package com.intrada.android.ui

import android.os.Handler
import android.os.Looper
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.intrada.android.core.ClickOutput
import com.intrada.ffi.TempoWords
import com.intrada.ffi.clickTempoWords
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

    /** Set only when the audio refused to start; a pulse another app took is not a fault. */
    var unavailable by mutableStateOf(false)
        private set

    /** False when the item declares no BPM, or one the band moved (#1942). */
    var soundsTarget by mutableStateOf(false)
        private set

    // A new unit keeps the tempo's number where the band allows: the pulse heard does not change
    // because the beat was renamed.
    val bar = ClickBar {
        bpm = limits?.clampClickTempo(bpm, metre.unit) ?: bpm
        configured = true
        if (isRunning) start()
    }

    private var seeded = 0
    private var seededUnit: UByte = 4u
    private var configured = false
    private var limits: LimitsView? = null
    private var output: ClickOutput? = null
    private var position: ULong? = null
    private val main = Handler(Looper.getMainLooper())
    private val backgroundStop = Runnable { stop() }

    val metre: Metre
        get() = bar.metre

    /** A bar change to another beat unit re-reads the same number as a different tempo (#1942). */
    val isAtSeededTempo: Boolean
        get() = bpm == seeded && metre.unit == seededUnit

    val step: Int
        get() = limits?.clickTempoStep?.toInt() ?: 0

    val band: IntRange
        get() = limits?.clickBand(metre.unit) ?: bpm..bpm

    /**
     * An output with no audio clock reports no beat. A fake answering true would keep the per-frame
     * loop running, and Compose tests would never go idle.
     */
    val tracksBeat: Boolean
        get() = isRunning && output?.tracksBeat == true

    val status: ClickStatus
        get() =
            when {
                isRunning -> ClickStatus.RUNNING
                unavailable -> ClickStatus.UNAVAILABLE
                else -> ClickStatus.STOPPED
            }

    /**
     * The bar is null until the click starts: the core reads one as a bar the musician chose
     * (#1499).
     */
    val reading: TempoReading
        get() =
            TempoReading(
                bpm.toUShort(),
                isRunning,
                if (configured) ClickState(metre, bar.sounding) else null,
            )

    /**
     * Reseeds only when the item changes, so a screen rebuilt by turning the phone keeps the click
     * sounding at the musician's tempo.
     */
    fun follow(active: ActiveSessionView, limits: LimitsView) {
        if (active.currentPosition == position) return
        position = active.currentPosition
        reseed(active, limits)
    }

    /** The session ended: the next one starts from its own first item. */
    fun release() {
        stop()
        position = null
    }

    /** Silences the click: its tempo belonged to the item that just finished (#2225). */
    private fun reseed(active: ActiveSessionView, limits: LimitsView) {
        stop()
        unavailable = false
        this.limits = limits
        bar.reseed(active, limits)
        seeded = active.clickSeedBpm.toInt()
        seededUnit = metre.unit
        soundsTarget = active.clickSeedSoundsTarget
        bpm = seeded
        configured = false
    }

    fun toggle() = if (isRunning) stop() else start()

    fun step(by: Int) = dragTo(bpm + by)

    fun dragTo(value: Int) {
        val next = limits?.clampClickTempo(value, metre.unit) ?: value
        if (next == bpm) return
        bpm = next
        if (isRunning) start()
    }

    fun currentBeat(): Int? = if (isRunning) output?.currentBeat() else null

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
        val started = out.start(bpm, metre.beats.toInt(), bar.sounding.toInt())
        isRunning = started
        unavailable = !started
        if (started) configured = true
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

data class ClickRowState(
    val bpm: Int,
    val unit: UByte,
    val status: ClickStatus,
    val atSeededTempo: Boolean,
    /** The item's declared tempo; null when the row names the click instead (#1942). */
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

    private val words: TempoWords
        get() = clickTempoWords(bpm.toUShort(), unit)

    val readout: String
        get() =
            when {
                unavailable -> "Metronome unavailable"
                showsBpm -> words.text
                else -> target ?: "Metronome"
            }

    val spoken: String
        get() =
            when {
                unavailable -> "unavailable"
                showsBpm -> words.spoken
                else -> targetSpoken ?: words.spoken
            }
}
