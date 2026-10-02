package com.intrada.android

import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Density
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.core.withIds
import com.intrada.android.ui.LibraryScreen
import com.intrada.shared.Event
import com.intrada.shared.LibraryItemView
import kotlinx.coroutines.test.StandardTestDispatcher
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
class LibraryScreenSnapshotTest {
    @Test
    fun library() = runTest {
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
        val view = store.viewModel.value
        val rows = store.libraryRows.value.withIds(view?.visibleIds.orEmpty())

        captureRoboImage("src/test/snapshots/library.png") {
            LibraryScreen(rows, error = null, halted = false, onDismissError = {})
        }
    }

    @Test
    fun libraryWithAnError() = runTest {
        val rows = loadedRows()
        captureRoboImage("src/test/snapshots/library-error.png") {
            LibraryScreen(rows, error = ERROR, halted = false, onDismissError = {})
        }
    }

    @Test
    fun libraryHaltedWithAnErrorAtTheLargestFontScale() = runTest {
        val rows = loadedRows()
        captureRoboImage("src/test/snapshots/library-error-largest-font.png") {
            val density = LocalDensity.current
            CompositionLocalProvider(
                LocalDensity provides Density(density.density, fontScale = LARGEST_FONT_SCALE)
            ) {
                LibraryScreen(rows, error = ERROR, halted = true, onDismissError = {})
            }
        }
    }

    private suspend fun TestScope.loadedRows(): List<LibraryItemView> {
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
        return store.libraryRows.value.withIds(store.viewModel.value?.visibleIds.orEmpty())
    }

    private companion object {
        const val ERROR = "Couldn't delete that item."
        const val LARGEST_FONT_SCALE = 2f
    }
}
