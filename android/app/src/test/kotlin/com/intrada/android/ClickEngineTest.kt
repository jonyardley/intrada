package com.intrada.android

import android.app.Application
import android.content.Intent
import android.media.AudioManager
import androidx.test.core.app.ApplicationProvider
import com.intrada.android.core.ClickEngine
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ClickEngineTest {
    private val application = ApplicationProvider.getApplicationContext<Application>()
    private val engine = ClickEngine(application)
    private var deaths = 0

    @After fun silence() = engine.stop()

    private val noisyReceivers
        get() =
            shadowOf(application).registeredReceivers.filter {
                it.intentFilter.hasAction(AudioManager.ACTION_AUDIO_BECOMING_NOISY)
            }

    private val listening: Boolean
        get() = noisyReceivers.isNotEmpty()

    private fun startEngine() {
        engine.onPulseDied = { deaths++ }
        assertTrue(engine.start(BPM, BEATS, SOUNDING))
        assertTrue(listening)
    }

    // Robolectric's AudioTrack cannot write floats, so the pulse soon dies by itself and posts its
    // stop to the main looper. Delivered by hand and checked before that looper idles, the
    // broadcast
    // is the only thing that can have stopped it.
    @Test
    fun unpluggedHeadphonesStopTheClick() {
        startEngine()

        val noisy = Intent(AudioManager.ACTION_AUDIO_BECOMING_NOISY)
        noisyReceivers.forEach { it.broadcastReceiver.onReceive(application, noisy) }

        assertEquals(1, deaths)
        assertFalse(listening)
    }

    @Test
    fun aChosenStopLetsGoOfTheHeadphones() {
        startEngine()

        engine.stop()

        assertEquals(0, deaths)
        assertFalse(listening)
    }

    @Test
    fun aFailedRestartLetsGoOfTheHeadphones() {
        startEngine()

        assertFalse(engine.start(0, BEATS, SOUNDING))

        assertFalse(listening)
    }

    private companion object {
        const val BPM = 120
        const val BEATS = 4
        const val SOUNDING = 0b1111
    }
}
