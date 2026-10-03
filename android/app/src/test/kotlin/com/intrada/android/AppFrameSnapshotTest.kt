package com.intrada.android

import androidx.activity.ComponentActivity
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.unit.Density
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.AppFrame
import com.intrada.android.ui.AppTab
import com.intrada.shared.Event
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class AppFrameSnapshotTest {
    @get:Rule val compose = createAndroidComposeRule<ComponentActivity>()

    @Test fun library() = capture(AppTab.LIBRARY, fontScale = 1f)

    @Test fun libraryAtTheLargestFontScale() = capture(AppTab.LIBRARY, LARGEST_FONT_SCALE)

    @Test fun practice() = capture(AppTab.PRACTICE, fontScale = 1f)

    @Test fun practiceAtTheLargestFontScale() = capture(AppTab.PRACTICE, LARGEST_FONT_SCALE)

    @Test fun routines() = capture(AppTab.ROUTINES, fontScale = 1f)

    @Test fun routinesAtTheLargestFontScale() = capture(AppTab.ROUTINES, LARGEST_FONT_SCALE)

    @Test fun progress() = capture(AppTab.PROGRESS, fontScale = 1f)

    @Test fun progressAtTheLargestFontScale() = capture(AppTab.PROGRESS, LARGEST_FONT_SCALE)

    private fun capture(tab: AppTab, fontScale: Float) = runTest {
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(Fixtures.library),
                this,
                StandardTestDispatcher(testScheduler),
                log = {},
            )
        store.send(Event.StartApp)
        store.settle()
        compose.setContent {
            val density = LocalDensity.current
            CompositionLocalProvider(
                LocalDensity provides Density(density.density, fontScale = fontScale)
            ) {
                AppFrame(store)
            }
        }
        compose.onNodeWithTag(tab.tag).performClick()
        val suffix = if (fontScale == 1f) "" else "-largest-font"
        compose.onRoot().captureRoboImage("src/test/snapshots/frame-${tab.route}$suffix.png")
    }

    private companion object {
        const val LARGEST_FONT_SCALE = 2f
    }
}
