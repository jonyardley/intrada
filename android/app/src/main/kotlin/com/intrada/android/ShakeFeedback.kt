package com.intrada.android

import android.app.Activity
import android.graphics.Bitmap
import android.hardware.Sensor
import android.hardware.SensorEvent
import android.hardware.SensorEventListener
import android.hardware.SensorManager
import android.os.Handler
import android.os.Looper
import android.view.PixelCopy
import androidx.compose.runtime.MutableState
import androidx.core.graphics.createBitmap
import com.intrada.android.core.ShakeDetector
import java.io.ByteArrayOutputStream

/** The feedback form, opened by a shake with the screen as it was; one at a time. */
class FeedbackRequest(val screenshot: ByteArray?)

class ShakeFeedback(
    private val activity: Activity,
    private val request: MutableState<FeedbackRequest?>,
) : SensorEventListener {
    private val detector = ShakeDetector()
    private val sensors by lazy { activity.getSystemService(SensorManager::class.java) }

    fun listen() {
        val accelerometer = sensors?.getDefaultSensor(Sensor.TYPE_ACCELEROMETER) ?: return
        sensors.registerListener(this, accelerometer, SensorManager.SENSOR_DELAY_UI)
    }

    fun stop() {
        sensors?.unregisterListener(this)
    }

    override fun onSensorChanged(event: SensorEvent) {
        val (x, y, z) = event.values
        if (!detector.feed(x, y, z, event.timestamp / NANOS_PER_MILLI)) return
        if (request.value != null) return
        capture { request.value = FeedbackRequest(it) }
    }

    override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) = Unit

    private fun capture(done: (ByteArray?) -> Unit) {
        val view = activity.window.decorView
        if (view.width == 0 || view.height == 0) return done(null)
        val bitmap = createBitmap(view.width, view.height)
        PixelCopy.request(
            activity.window,
            bitmap,
            { result ->
                done(if (result == PixelCopy.SUCCESS) bitmap.png() else null)
                bitmap.recycle()
            },
            Handler(Looper.getMainLooper()),
        )
    }

    private fun Bitmap.png(): ByteArray =
        ByteArrayOutputStream()
            .also { compress(Bitmap.CompressFormat.PNG, PNG_QUALITY, it) }
            .toByteArray()

    private companion object {
        const val NANOS_PER_MILLI = 1_000_000
        const val PNG_QUALITY = 100
    }
}
