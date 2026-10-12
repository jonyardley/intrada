package com.intrada.android.ui

import android.os.SystemClock
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import com.intrada.android.core.ClickEngine

/**
 * One drag on the metronome button (#1823). The readout tracks every step crossed; the engine takes
 * a new tempo only once the last restart has been [heard], up to a ceiling, or each restart cancels
 * the last before it sounds and the click goes quiet for the whole drag (#2539).
 */
class TempoDrag(
    private val anchor: Int,
    private val step: Int,
    private val band: IntRange,
    private val commit: (Int) -> Unit,
    private val heard: () -> Boolean,
    private val now: () -> Long = SystemClock::uptimeMillis,
) {
    var live by mutableIntStateOf(anchor)
        private set

    private var committed = anchor
    private var committedAt = now()

    /** [travelDp] is the finger's travel since the drag began, downward positive. */
    fun moved(travelDp: Float) {
        live = bpm(travelDp, anchor, step, band)
        settle()
    }

    /** Called each frame too, so a step held back reaches the click while the finger rests. */
    fun settle() {
        if (live == committed) return
        val at = now()
        if (ready(at - committedAt)) {
            committed = live
            committedAt = at
            commit(live)
        }
    }

    // A track that never reports where it is playing still follows the drag.
    private fun ready(since: Long) =
        since >= COMMIT_INTERVAL_MILLIS && (since >= UNHEARD_LIMIT_MILLIS || heard())

    fun ended() {
        if (live != committed) commit(live)
    }

    companion object {
        /** Tuned by feel on iOS: a step reads as deliberate, a thumb's drag crosses the band. */
        const val DP_PER_STEP = 8f
        val COMMIT_INTERVAL_MILLIS = (ClickEngine.LEAD_IN_SECONDS * 1000).toLong() + 20

        /** Past Bluetooth's 150 to 250ms output latency plus the lead-in. */
        const val UNHEARD_LIMIT_MILLIS = 500L

        /** Anchored to where the drag began, so drift never compounds across steps. */
        fun bpm(travelDp: Float, anchor: Int, step: Int, band: IntRange): Int =
            (anchor + (-travelDp / DP_PER_STEP).toInt() * step).coerceIn(band)
    }
}
