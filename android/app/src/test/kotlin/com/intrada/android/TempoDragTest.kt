package com.intrada.android

import com.intrada.android.ui.TempoDrag
import org.junit.Assert.assertEquals
import org.junit.Test

class TempoDragTest {
    private var clock = 0L
    private val committed = mutableListOf<Int>()

    private fun drag(anchor: Int = 96) =
        TempoDrag(anchor, step = 2, band = 40..208, commit = { committed += it }, now = { clock })

    @Test
    fun draggingUpEightDpIsOneStepFaster() {
        assertEquals(98, TempoDrag.bpm(-8f, 96, 2, 40..208))
        assertEquals(96, TempoDrag.bpm(-7.9f, 96, 2, 40..208))
        assertEquals(94, TempoDrag.bpm(8f, 96, 2, 40..208))
    }

    @Test
    fun aLongDragStopsAtTheBand() {
        assertEquals(208, TempoDrag.bpm(-10_000f, 96, 2, 40..208))
        assertEquals(40, TempoDrag.bpm(10_000f, 96, 2, 40..208))
    }

    // A commit inside the lead-in cancels the previous start before it sounds (#1823).
    @Test
    fun stepsInsideTheLeadInReachTheReadoutButNotTheClick() {
        val drag = drag()
        clock = TempoDrag.COMMIT_INTERVAL_MILLIS - 1
        drag.moved(-8f)
        drag.moved(-16f)

        assertEquals(100, drag.live)
        assertEquals(emptyList<Int>(), committed)

        clock = TempoDrag.COMMIT_INTERVAL_MILLIS
        drag.moved(-24f)
        assertEquals(listOf(102), committed)
    }

    @Test
    fun liftingTheFingerCommitsWhereItStopped() {
        val drag = drag()
        drag.moved(-16f)

        drag.ended()

        assertEquals(listOf(100), committed)
    }

    @Test
    fun aDragBackToTheStartCommitsNothing() {
        val drag = drag()
        drag.moved(-4f)

        drag.ended()

        assertEquals(emptyList<Int>(), committed)
    }
}
