package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.PracticeModel
import com.intrada.android.ui.PracticeScreen
import com.intrada.android.ui.SessionDetailScreen
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
    fun practiceUpNext() =
        captureRoboImage("src/test/snapshots/practice-up-next.png") {
            PracticeScreen(PracticeFixtures.suggested, onStart = {}, onOpen = {})
        }

    @Test
    fun practiceUpNextFilled() =
        captureRoboImage("src/test/snapshots/practice-up-next-filled.png") {
            PracticeScreen(
                PracticeModel(
                    PracticeFixtures.filled.weeks,
                    PracticeFixtures.filled.sessions,
                    PracticeFixtures.filled.lastPractised,
                    upNext = PracticeFixtures.filledPlan,
                ),
                onStart = {},
                onOpen = {},
            )
        }

    @Test
    fun practiceUpNextDismissed() =
        captureRoboImage("src/test/snapshots/practice-up-next-dismissed.png") {
            PracticeScreen(
                PracticeFixtures.suggested,
                onStart = {},
                onOpen = {},
                suggestionDismissed = true,
            )
        }

    @Test
    fun sessionDetail() = runTest {
        val topMark = topMark()
        captureRoboImage("src/test/snapshots/session-detail.png") {
            SessionDetailScreen(PracticeFixtures.completed, topMark)
        }
    }

    @Test
    fun sessionDetailVariations() = runTest {
        val topMark = topMark()
        captureRoboImage("src/test/snapshots/session-detail-variations.png") {
            SessionDetailScreen(PracticeFixtures.withVariations, topMark)
        }
    }

    private suspend fun TestScope.topMark(): Int =
        openedStore().viewModel.value?.limits?.scoreMax?.toInt() ?: error("no view")
}
