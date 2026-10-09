package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.PracticeScreen
import com.intrada.android.ui.SessionDetailScreen
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class PracticeSnapshotTest {
    @Test
    fun practiceEmpty() =
        captureRoboImage("src/test/snapshots/practice-empty.png") {
            PracticeScreen(PracticeFixtures.empty, onStart = {}, onOpen = {})
        }

    @Test
    fun practiceFilled() =
        captureRoboImage("src/test/snapshots/practice-filled.png") {
            PracticeScreen(PracticeFixtures.filled, onStart = {}, onOpen = {})
        }

    @Test
    fun sessionDetail() =
        captureRoboImage("src/test/snapshots/session-detail.png") {
            SessionDetailScreen(PracticeFixtures.completed, topMark = 10)
        }

    @Test
    fun sessionDetailVariations() =
        captureRoboImage("src/test/snapshots/session-detail-variations.png") {
            SessionDetailScreen(PracticeFixtures.withVariations, topMark = 10)
        }
}
