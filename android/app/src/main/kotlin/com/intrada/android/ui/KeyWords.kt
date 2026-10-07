package com.intrada.android.ui

import android.util.Log
import com.intrada.ffi.CoreException
import com.intrada.ffi.WheelMode
import com.intrada.ffi.WheelSelection
import com.intrada.ffi.WheelWedge
import com.intrada.ffi.keyLabel
import com.intrada.ffi.keyNextOnTap
import com.intrada.ffi.keyWheel
import com.intrada.ffi.keyWheelSelection
import com.intrada.shared.Key
import com.novi.serde.DeserializationError

// The circle of fifths and its tap rule come from the core (#2106, #2226).
private val wedges: List<WheelWedge> by lazy { keyWheel() }

internal fun wheelWedge(ring: Int, mode: WheelMode): WheelWedge? = wedges.firstOrNull {
    it.ring.toInt() == ring && it.mode == mode
}

internal fun wheelSelection(key: Key): WheelSelection? = bridged {
    keyWheelSelection(key.bincodeSerialize())
}

internal fun keyAfterTap(current: Key?, ring: Int, mode: WheelMode): Key? = bridged {
    keyNextOnTap(current?.bincodeSerialize(), ring.toUByte(), mode)?.let {
        Key.bincodeDeserialize(it.key)
    }
}

internal fun keyDisplay(key: Key): String? = bridged { keyLabel(key.bincodeSerialize()) }

// A failure is a wire break (#846): logged, and the wheel lights nothing.
private fun <T> bridged(call: () -> T?): T? =
    try {
        call()
    } catch (e: CoreException) {
        Log.w("intrada", "key wheel: $e")
        null
    } catch (e: DeserializationError) {
        Log.w("intrada", "key wheel: $e")
        null
    }

internal fun prettify(spelling: String): String = buildString {
    spelling.forEachIndexed { index, c ->
        val afterLetter = index > 0 && spelling[index - 1] in 'A'..'G'
        append(
            when {
                c == '#' -> '♯'
                c == 'b' && afterLetter -> '♭'
                else -> c
            }
        )
    }
}

internal fun spokenTonic(spelling: String): String = buildString {
    spelling.forEachIndexed { index, c ->
        val afterLetter = index > 0 && spelling[index - 1] in 'A'..'G'
        when {
            c == '#' -> append(" sharp")
            c == 'b' && afterLetter -> append(" flat")
            else -> append(c)
        }
    }
}

internal val WheelMode.word: String
    get() =
        when (this) {
            WheelMode.MAJOR -> "major"
            WheelMode.MINOR -> "minor"
        }

// Enharmonic spokes announce both spellings, since one tap selects and a second flips them.
internal fun wedgeSpoken(ring: Int, mode: WheelMode): String {
    val wedge = wheelWedge(ring, mode) ?: return ""
    val alt = wedge.alt
    val tonic =
        if (alt == null) spokenTonic(wedge.primary)
        else "${spokenTonic(wedge.primary)} or ${spokenTonic(alt)}"
    return "$tonic ${mode.word}"
}
