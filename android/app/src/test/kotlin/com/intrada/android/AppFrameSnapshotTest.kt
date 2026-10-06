package com.intrada.android

import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
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

    @Test fun library() = capture(AppTab.LIBRARY, "src/test/snapshots/frame-library.png")

    @Test fun practice() = capture(AppTab.PRACTICE, "src/test/snapshots/frame-practice.png")

    @Test fun routines() = capture(AppTab.ROUTINES, "src/test/snapshots/frame-routines.png")

    @Test fun progress() = capture(AppTab.PROGRESS, "src/test/snapshots/frame-progress.png")

    private fun capture(tab: AppTab, reference: String) = runTest {
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
        compose.setContent { AppFrame(store) }
        compose.onNodeWithTag(tab.tag).performClick()
        compose.onRoot().captureRoboImage(reference)
    }
}
