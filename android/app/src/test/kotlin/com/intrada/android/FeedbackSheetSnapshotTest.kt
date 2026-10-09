package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.FeedbackSheet
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class FeedbackSheetSnapshotTest {
    @Test
    fun sheetFromAShake() {
        val screenshot = feedbackScreenshot()
        captureRoboImage("src/test/snapshots/feedback-sheet.png") {
            FeedbackSheet(screenshot, onDismiss = {}, send = {})
        }
    }
}
