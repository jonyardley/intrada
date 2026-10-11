package com.intrada.android.ui

import android.os.SystemClock
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import com.intrada.android.core.ClickEngine

/**
 * One drag on the metronome button (#1823). The readout tracks every step crossed; the engine takes
 * only what its lead-in silence can absorb, or each restart cancels the last before it sounds and
 * the click goes quiet for the whole drag.
 */
class TempoDrag(
    private val anchor: Int,
    private val step: Int,
    private val band: IntRange,
    private val commit: (Int) -> Unit,
    private val now: () -> Long = SystemClock::uptimeMillis,
) {
    var live by mutableIntStateOf(anchor)
        private set

    private var committed = anchor
    private var committedAt = now()

    /** [travelDp] is the finger's travel since the drag began, downward positive. */
    fun moved(travelDp: Float) {
        val next = bpm(travelDp, anchor, step, band)
        if (next == live) return
        live = next
        val at = now()
        if (at - committedAt < COMMIT_INTERVAL_MILLIS) return
        committed = next
        committedAt = at
        commit(next)
    }

    fun ended() {
        if (live != committed) commit(live)
    }

    companion object {
        /** Tuned by feel on iOS: a step reads as deliberate, a thumb's drag crosses the band. */
        const val DP_PER_STEP = 8f
        val COMMIT_INTERVAL_MILLIS = (ClickEngine.LEAD_IN_SECONDS * 1000).toLong() + 20

        /** Anchored to where the drag began, so drift never compounds across steps. */
        fun bpm(travelDp: Float, anchor: Int, step: Int, band: IntRange): Int =
            (anchor + (-travelDp / DP_PER_STEP).toInt() * step).coerceIn(band)
    }
}
