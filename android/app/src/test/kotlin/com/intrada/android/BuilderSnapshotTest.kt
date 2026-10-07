package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.Store
import com.intrada.android.ui.BuilderActions
import com.intrada.android.ui.BuilderModel
import com.intrada.android.ui.BuilderNavigation
import com.intrada.android.ui.BuilderScreen
import com.intrada.android.ui.BuilderScreenState
import com.intrada.android.ui.EntrySettingsScreen
import com.intrada.android.ui.LibraryPickerScreen
import com.intrada.android.ui.PickerActions
import com.intrada.android.ui.PickerCopy
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import com.intrada.shared.ViewModel
import kotlinx.coroutines.test.runTest
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class BuilderSnapshotTest {
    @Test
    fun builderEmpty() = runTest {
        val store = libraryStore()
        store.send(Event.Session(SessionEvent.StartBuilding))
        val view = store.viewModel.value
        captureRoboImage("src/test/snapshots/builder-empty.png") { Builder(view) }
    }

    @Test
    fun builder() = runTest {
        val view = plannedStore().viewModel.value
        captureRoboImage("src/test/snapshots/builder.png") { Builder(view) }
    }

    @Test
    fun builderEditing() = runTest {
        val view = plannedStore().viewModel.value
        captureRoboImage("src/test/snapshots/builder-editing.png") {
            Builder(view, BuilderScreenState(editing = true))
        }
    }

    @Test
    fun entrySettings() = runTest {
        val store = plannedStore()
        val piece = store.setlist().entries.single { it.itemId == BuilderFixtures.PIECE }
        val limits = store.viewModel.value?.limits ?: error("no limits")
        captureRoboImage("src/test/snapshots/builder-entry-settings.png") {
            EntrySettingsScreen(piece, store.setlist(), limits, send = { true }, onDone = {})
        }
    }

    @Test
    fun addToSession() = runTest {
        val store = buildingStore()
        val added = store.setlist().entries.map { it.itemId }.toSet()
        val rows = store.libraryRows.value
        captureRoboImage("src/test/snapshots/builder-add.png") {
            LibraryPickerScreen(
                PickerCopy(
                    "Add to session",
                    "addToSession",
                    "Pieces bring their related exercises as a group.",
                    "",
                ),
                rows,
                added,
                PickerActions(onDone = {}, onToggle = {}),
            )
        }
    }

    private suspend fun kotlinx.coroutines.test.TestScope.plannedStore(): Store {
        val store = buildingStore()
        val setlist = store.setlist()
        val piece = setlist.entries.single { it.itemId == BuilderFixtures.PIECE }
        val run = setlist.entryVariations.single { it.entryId == piece.id }.sections.last().id
        store.send(Event.Session(SessionEvent.SetEntryDuration(piece.id, 600u)))
        store.send(Event.Session(SessionEvent.AddSegment(piece.id, run)))
        store.send(Event.Session(SessionEvent.SetSessionLength(30u)))
        store.settle()
        return store
    }

    @androidx.compose.runtime.Composable
    private fun Builder(view: ViewModel?, state: BuilderScreenState = BuilderScreenState()) {
        if (view == null) return
        val setlist = view.buildingSetlist ?: return
        BuilderScreen(
            BuilderModel(setlist, view.limits),
            state,
            BuilderActions({ true }, BuilderNavigation({}, {}, {}, {}), {}),
        )
    }
}
