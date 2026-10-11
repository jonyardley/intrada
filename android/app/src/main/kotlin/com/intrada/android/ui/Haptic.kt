package com.intrada.android.ui

import androidx.compose.ui.hapticfeedback.HapticFeedback
import androidx.compose.ui.hapticfeedback.HapticFeedbackType

/**
 * Every Android haptic goes through here, as on iOS (#2013), so a change to the feel is made once.
 */
enum class Haptic(val type: HapticFeedbackType) {
    SELECTION(HapticFeedbackType.SegmentTick),
    IMPACT(HapticFeedbackType.ContextClick),
}

fun HapticFeedback.play(haptic: Haptic) = performHapticFeedback(haptic.type)
