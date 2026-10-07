package com.intrada.android

import com.intrada.android.core.ClickGrid
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ClickGridTest {
    private val rate = 48_000

    private fun grid(bpm: Int, beats: Int = 4, sounding: Int = 0b1111, leadIn: Long = 0) =
        ClickGrid.of(rate, bpm, beats, sounding, leadIn) ?: error("no grid at $bpm")

    @Test
    fun aBeatAtSixtyFallsOnEverySecond() {
        val grid = grid(60)
        assertEquals(listOf(0L, 48_000L, 96_000L), (0L..2L).map(grid::beatFrame))
    }

    // 7 bpm and 48 kHz give 411428.57 frames a beat: stepping beat to beat by the rounded
    // period would be 0.43 frames short each beat, about 26 ms after an hour.
    @Test
    fun aTempoThatDoesNotDivideTheRateNeverDrifts() {
        val grid = grid(7)
        val hour = 7L * 60
        assertEquals(rate * 3600L, grid.beatFrame(hour))
        assertEquals(411_429L, grid.beatFrame(1))
    }

    @Test
    fun atTheTopOfTheBandEachBeatStaysWithinHalfAFrame() {
        val grid = grid(208)
        val period = rate * 60.0 / 208
        (0L..10_000L).forEach { index ->
            assertTrue(kotlin.math.abs(grid.beatFrame(index) - index * period) <= 0.5)
        }
    }

    @Test
    fun theLeadInDelaysEveryBeatByTheSameFrames() {
        assertEquals(1_200L + 48_000L, grid(60, leadIn = 1_200).beatFrame(1))
    }

    @Test
    fun aSilencedBeatIsSkippedInEveryBar() {
        val downbeat = grid(120, beats = 3, sounding = 0b001)
        assertEquals(listOf(true, false, false, true), (0L..3L).map(downbeat::sounds))
    }

    @Test
    fun renderingInChunksMatchesRenderingInOne() {
        val grid = grid(97, beats = 4, sounding = 0b0101)
        val click = ClickGrid.click(rate)
        val whole = FloatArray(rate * 3).also { grid.render(it, 0, click) }
        val chunked = FloatArray(rate * 3)
        val chunk = 511
        var start = 0
        while (start < chunked.size) {
            val piece = FloatArray(minOf(chunk, chunked.size - start))
            grid.render(piece, start.toLong(), click)
            piece.copyInto(chunked, start)
            start += piece.size
        }
        assertTrue(whole.contentEquals(chunked))
    }

    @Test
    fun aClickSoundsOnlyFromItsBeatFrame() {
        val grid = grid(60, beats = 2, sounding = 0b01)
        val click = ClickGrid.click(rate)
        val out = FloatArray(rate * 2 + click.size).also { grid.render(it, 0, click) }
        assertEquals(click[1], out[1], 0f)
        assertEquals(0f, out[rate + 1], 0f)
        assertEquals(click[1], out[rate * 2 + 1], 0f)
        assertFalse(out.copyOfRange(click.size, rate).any { it != 0f })
    }

    @Test
    fun aTempoTheGridCannotHoldIsRefused() {
        assertNull(ClickGrid.of(rate, 0, 4, 0b1111))
        assertNull(ClickGrid.of(0, 60, 4, 0b1111))
    }
}
