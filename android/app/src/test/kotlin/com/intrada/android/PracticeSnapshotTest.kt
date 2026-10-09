package com.intrada.android

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaSpacing
import com.intrada.android.ui.PracticeScreen
import com.intrada.android.ui.SessionDetailScreen
import com.intrada.android.ui.UpNextActions
import com.intrada.android.ui.UpNextCard
import com.intrada.shared.HighlighterColour
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
            CardOnPaper {
                UpNextCard(PracticeFixtures.plan, HighlighterColour.BUTTER, UpNextActions())
            }
        }

    @Test
    fun practiceUpNextFilled() =
        captureRoboImage("src/test/snapshots/practice-up-next-filled.png") {
            CardOnPaper {
                UpNextCard(PracticeFixtures.filledPlan, HighlighterColour.BUTTER, UpNextActions())
            }
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

    @Composable
    private fun CardOnPaper(content: @Composable () -> Unit) {
        Box(Modifier.width(360.dp).background(IntradaColor.paperTop).padding(IntradaSpacing.card)) {
            content()
        }
    }

    private suspend fun TestScope.topMark(): Int =
        openedStore().viewModel.value?.limits?.scoreMax?.toInt() ?: error("no view")
}
