package com.intrada.android

import com.intrada.ffi.declaredTempoWords
import org.junit.Assert.assertEquals
import org.junit.Test

// A bare declared tempo, worded by the core on Kotlin's decoder (#846, #2486).
class TempoWordsRoundTripTest {
    @Test
    fun aBareDeclaredTempoComesFromTheCore() {
        val words = declaredTempoWords(66u)
        assertEquals("♩ = 66", words.text)
        assertEquals("66 beats per minute", words.spoken)
    }
}
