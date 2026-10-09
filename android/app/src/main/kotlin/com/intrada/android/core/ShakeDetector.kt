package com.intrada.android.core

import kotlin.math.sqrt

/**
 * Android has no shake gesture, so a jolt past [threshold] g counts as one, at most once a second.
 */
class ShakeDetector(private val threshold: Double = 2.7, private val quietMillis: Long = 1_000) {
    private var lastShake: Long? = null

    fun feed(x: Float, y: Float, z: Float, atMillis: Long): Boolean {
        val gForce = sqrt((x * x + y * y + z * z).toDouble()) / EARTH_GRAVITY
        val last = lastShake
        val shook = gForce >= threshold && (last == null || atMillis - last >= quietMillis)
        if (shook) lastShake = atMillis
        return shook
    }

    private companion object {
        const val EARTH_GRAVITY = 9.80665
    }
}
