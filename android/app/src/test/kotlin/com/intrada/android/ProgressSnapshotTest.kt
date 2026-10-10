package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.ProgressScreen
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class ProgressSnapshotTest {
    // Tall enough to show the bars and recent mastery below the fold.
    @Test
    @Config(qualifiers = "w411dp-h1500dp-xxhdpi")
    fun progress() = runTest {
        val topMark = topMark()
        captureRoboImage("src/test/snapshots/progress.png") {
            ProgressScreen(ProgressFixtures.analytics, topMark, onBuild = {})
        }
    }

    @Test
    fun progressEmpty() = runTest {
        val topMark = topMark()
        captureRoboImage("src/test/snapshots/progress-empty.png") {
            ProgressScreen(null, topMark, onBuild = {})
        }
    }

    private suspend fun TestScope.topMark(): Int =
        openedStore().viewModel.value?.limits?.scoreMax?.toInt() ?: error("no view")
}
