package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.ExercisePickerScreen
import com.intrada.android.ui.ExercisePickerState
import com.intrada.android.ui.KeysSheetScreen
import com.intrada.android.ui.PieceActions
import com.intrada.android.ui.PieceNavigation
import com.intrada.android.ui.PieceScreen
import com.intrada.android.ui.PieceScreenState
import com.intrada.android.ui.VariationsSheetScreen
import com.intrada.android.ui.keysWithAll
import com.intrada.ffi.WheelMode
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
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
class PieceSnapshotTest {
    @Test
    fun piece() = runTest {
        val piece = populatedPiece()
        captureRoboImage("src/test/snapshots/piece.png") {
            PieceScreen(piece, PieceScreenState(), actions)
        }
    }

    @Test
    fun pieceEmpty() = runTest {
        val piece = startedStore().libraryRows.value.single { it.id == PIECE }
        captureRoboImage("src/test/snapshots/piece-empty.png") {
            PieceScreen(piece, PieceScreenState(), actions)
        }
    }

    @Test
    fun pieceEditing() = runTest {
        val piece = populatedPiece()
        captureRoboImage("src/test/snapshots/piece-editing.png") {
            PieceScreen(piece, PieceScreenState(editingLinks = true), actions)
        }
    }

    @Test
    fun exercisePicker() = runTest {
        val store = startedStore()
        val exercises = store.libraryRows.value.filter { it.itemType == ItemKind.EXERCISE }
        captureRoboImage("src/test/snapshots/piece-exercise-picker.png") {
            ExercisePickerScreen(exercises, ExercisePickerState(setOf(HANON)), {}, {})
        }
    }

    @Test
    fun keysSheet() = runTest {
        val piece = populatedPiece()
        captureRoboImage("src/test/snapshots/piece-keys-sheet.png") {
            KeysSheetScreen(piece, { null }, {})
        }
    }

    @Test
    fun variationsSheet() = runTest {
        val store = startedStore()
        val piece = store.populate()
        val library = store.viewModel.value?.variations.orEmpty()
        captureRoboImage("src/test/snapshots/piece-variations-sheet.png") {
            VariationsSheetScreen(piece, library, { null }, {})
        }
    }

    private val actions =
        PieceActions(
            PieceNavigation({}, {}, {}, {}),
            send = { true },
            sendFromSheet = { null },
        )

    private suspend fun TestScope.populatedPiece(): LibraryItemView = startedStore().populate()

    private suspend fun Store.populate(): LibraryItemView {
        val store = this
        store.send(Event.Item(ItemEvent.ChoosePieceExercises(PIECE, listOf(HANON), emptyList())))
        val keys = keysWithAll(emptyList(), WheelMode.MAJOR).take(3)
        store.send(Event.Item(ItemEvent.UpdateKeys(PIECE, keys)))
        store.send(
            Event.Item(
                ItemEvent.UpdateItemVariations(PIECE, emptyList(), listOf("Left hand alone"))
            )
        )
        store.settle()
        return store.libraryRows.value.single { it.id == PIECE }
    }

    private suspend fun TestScope.startedStore(): Store {
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
        return store
    }

    private companion object {
        const val PIECE = "01J0000000000000000000ITEM"
        const val HANON = "01J0000000000000000000HANO"
    }
}
