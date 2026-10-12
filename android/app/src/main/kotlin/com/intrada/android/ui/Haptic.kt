package com.intrada.android.ui

import androidx.compose.ui.hapticfeedback.HapticFeedback
import androidx.compose.ui.hapticfeedback.HapticFeedbackType

/** Every Android haptic goes through here, so a change to the feel is made once (#2013). */
enum class Haptic(val type: HapticFeedbackType) {
    SELECTION(HapticFeedbackType.SegmentTick),
    IMPACT(HapticFeedbackType.LongPress),
}

fun HapticFeedback.play(haptic: Haptic) = performHapticFeedback(haptic.type)
