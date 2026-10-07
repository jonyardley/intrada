package com.intrada.android.core

import android.content.Context
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioFormat
import android.media.AudioManager
import android.media.AudioTrack
import android.os.Handler
import android.os.Looper
import android.os.Process
import android.util.Log

/** What the player's click drives; a fake stands in for it under test. */
interface ClickOutput {
    /** False when the audio could not start; the caller shows the click as unavailable. */
    fun start(bpm: Int, beats: Int, sounding: Int): Boolean

    fun stop()

    /** The pulse stopped without being asked (another app took the audio). */
    var onPulseDied: (() -> Unit)?
}

/**
 * The metronome on one streaming [AudioTrack]: an audio thread renders [ClickGrid] chunks and
 * writes them blocking, so the click's timing is the sample count, never a timer (spec
 * `android-shell.md`, Phase C).
 */
class ClickEngine(context: Context) : ClickOutput {
    private val audio = context.getSystemService(AudioManager::class.java)
    private val main = Handler(Looper.getMainLooper())
    private val sampleRate =
        audio?.getProperty(AudioManager.PROPERTY_OUTPUT_SAMPLE_RATE)?.toIntOrNull() ?: FALLBACK_RATE
    private val click = ClickGrid.click(sampleRate)
    private val attributes =
        AudioAttributes.Builder()
            .setUsage(AudioAttributes.USAGE_MEDIA)
            .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
            .build()
    // Transient and duckable, so a backing track keeps playing quieter rather than stopping; a
    // call or another app's full claim takes it, as an interruption does on iOS.
    private val focus =
        AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK)
            .setAudioAttributes(attributes)
            .setOnAudioFocusChangeListener(
                { change ->
                    if (
                        change == AudioManager.AUDIOFOCUS_LOSS ||
                            change == AudioManager.AUDIOFOCUS_LOSS_TRANSIENT
                    ) {
                        abandonPulse()
                    }
                },
                main,
            )
            .build()
    private var pulse: Pulse? = null

    override var onPulseDied: (() -> Unit)? = null

    override fun start(bpm: Int, beats: Int, sounding: Int): Boolean {
        halt()
        val grid =
            ClickGrid.of(sampleRate, bpm, beats, sounding, (sampleRate * LEAD_IN_SECONDS).toLong())
        val manager = audio
        val granted =
            grid != null &&
                manager != null &&
                manager.requestAudioFocus(focus) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED
        val track = if (granted) openTrack() else null
        if (grid == null || track == null) {
            if (granted) manager.abandonAudioFocusRequest(focus)
            return false
        }
        track.play()
        pulse =
            Pulse(grid, track, click) { dead -> main.post { if (pulse === dead) abandonPulse() } }
                .also { it.start() }
        return true
    }

    override fun stop() {
        val wasSounding = pulse != null
        halt()
        if (wasSounding) audio?.abandonAudioFocusRequest(focus)
    }

    private fun halt() {
        pulse?.finish()
        pulse = null
    }

    /** Never called from [stop], so a stop the shell chose is not mistaken for a death. */
    private fun abandonPulse() {
        if (pulse == null) return
        stop()
        onPulseDied?.invoke()
    }

    private fun openTrack(): AudioTrack? {
        val track =
            try {
                buildTrack()
            } catch (error: IllegalArgumentException) {
                Log.e(TAG, "click.start", error)
                null
            } catch (error: UnsupportedOperationException) {
                Log.e(TAG, "click.start", error)
                null
            }
        if (track?.state == AudioTrack.STATE_INITIALIZED) return track
        track?.release()
        return null
    }

    private fun buildTrack(): AudioTrack {
        val format =
            AudioFormat.Builder()
                .setEncoding(AudioFormat.ENCODING_PCM_FLOAT)
                .setSampleRate(sampleRate)
                .setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
                .build()
        val minimum =
            AudioTrack.getMinBufferSize(
                sampleRate,
                AudioFormat.CHANNEL_OUT_MONO,
                AudioFormat.ENCODING_PCM_FLOAT,
            )
        return AudioTrack.Builder()
            .setAudioAttributes(attributes)
            .setAudioFormat(format)
            .setBufferSizeInBytes(maxOf(minimum, CHUNK_FRAMES * Float.SIZE_BYTES) * 2)
            .setTransferMode(AudioTrack.MODE_STREAM)
            .setPerformanceMode(AudioTrack.PERFORMANCE_MODE_LOW_LATENCY)
            .build()
    }

    private class Pulse(
        private val grid: ClickGrid,
        private val track: AudioTrack,
        private val click: FloatArray,
        private val onDied: (Pulse) -> Unit,
    ) : Thread("click") {
        private val lock = Any()
        @Volatile private var running = true
        private var released = false

        override fun run() {
            Process.setThreadPriority(Process.THREAD_PRIORITY_URGENT_AUDIO)
            val chunk = FloatArray(CHUNK_FRAMES)
            var frame = 0L
            try {
                while (running) {
                    chunk.fill(0f)
                    grid.render(chunk, frame, click)
                    val written = track.write(chunk, 0, chunk.size, AudioTrack.WRITE_BLOCKING)
                    if (written < 0) {
                        if (running) onDied(this)
                        return
                    }
                    frame += written
                }
            } finally {
                synchronized(lock) {
                    released = true
                    track.release()
                }
            }
        }

        // The track is released on its own thread, so a write is never left holding a freed
        // track; stop() wakes a blocked write so that happens at once.
        fun finish() {
            synchronized(lock) {
                running = false
                if (!released) track.stop()
            }
        }
    }

    private companion object {
        const val TAG = "intrada"
        const val FALLBACK_RATE = 48_000
        const val LEAD_IN_SECONDS = 0.05
        const val CHUNK_FRAMES = 256
    }
}
