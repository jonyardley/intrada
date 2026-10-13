package com.intrada.android

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaSpacing
import com.intrada.android.ui.MasteryDelta
import com.intrada.android.ui.ProgressScreen
import com.intrada.shared.Direction
import com.intrada.shared.ScoreChange
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

    @Test
    fun masteryAfterAFall() {
        captureRoboImage("src/test/snapshots/progress-mastery-fall.png") {
            Box(
                Modifier.width(360.dp)
                    .background(IntradaColor.paperTop)
                    .padding(IntradaSpacing.section)
            ) {
                MasteryDelta(
                    ScoreChange(
                        itemId = "exercise-2",
                        itemTitle = "Arpeggios",
                        previousScore = 6u,
                        currentScore = 4u,
                        delta = -2,
                        isNew = false,
                        subtitle = null,
                        spoken = "Arpeggios, mastery down from 6 to 4",
                        direction = Direction.DOWN,
                    )
                )
            }
        }
    }

    private suspend fun TestScope.topMark(): Int =
        openedStore().viewModel.value?.limits?.scoreMax?.toInt() ?: error("no view")
}
