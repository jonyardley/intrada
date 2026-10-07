package com.intrada.android.core

import kotlin.math.PI
import kotlin.math.floor
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToLong
import kotlin.math.sin

/**
 * Where each click falls in the output stream, in frames. Every beat is placed from the start of
 * the pulse, never from the beat before, so a long run cannot gather rounding error (the same rule
 * as iOS's host-time grid, #1282).
 */
class ClickGrid
private constructor(
    private val framesPerBeat: Double,
    private val beats: Int,
    private val sounding: Int,
    private val leadInFrames: Long,
) {
    fun beatFrame(index: Long): Long = leadInFrames + (index * framesPerBeat).roundToLong()

    /** The pattern only gates which beats sound; the grid never changes rate (T19). */
    fun sounds(index: Long): Boolean =
        beats <= 0 || sounding and (1 shl (index % beats).toInt()) != 0

    /** Adds every sounding click overlapping `[start, start + out.size)` into [out]. */
    fun render(out: FloatArray, start: Long, click: FloatArray) {
        val end = start + out.size
        var index = max(0L, floor((start - leadInFrames - click.size) / framesPerBeat).toLong())
        while (true) {
            val at = beatFrame(index)
            if (at >= end) return
            if (sounds(index) && at + click.size > start) {
                val from = max(at, start)
                val to = min(at + click.size, end)
                for (frame in from until to) {
                    out[(frame - start).toInt()] += click[(frame - at).toInt()]
                }
            }
            index++
        }
    }

    companion object {
        /** Null for a tempo the grid cannot hold, so a zero or negative bpm never reaches audio. */
        fun of(
            sampleRate: Int,
            bpm: Int,
            beats: Int,
            sounding: Int,
            leadInFrames: Long = 0,
        ): ClickGrid? {
            if (sampleRate <= 0 || bpm <= 0) return null
            return ClickGrid(sampleRate * SECONDS_PER_MINUTE / bpm, beats, sounding, leadInFrames)
        }

        /** A 1 kHz tone with a linear decay, which avoids a pop; synthesised to need no asset. */
        fun click(sampleRate: Int): FloatArray {
            val frames = (sampleRate * CLICK_SECONDS).toInt()
            return FloatArray(frames) { frame ->
                val t = frame.toDouble() / sampleRate
                (sin(2 * PI * CLICK_HZ * t) * (1 - t / CLICK_SECONDS) * CLICK_GAIN).toFloat()
            }
        }

        private const val SECONDS_PER_MINUTE = 60.0
        private const val CLICK_SECONDS = 0.03
        private const val CLICK_HZ = 1000.0
        private const val CLICK_GAIN = 0.5
    }
}
