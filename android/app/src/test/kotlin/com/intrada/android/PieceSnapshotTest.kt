package com.intrada.android

import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.ExercisePickerScreen
import com.intrada.android.ui.ExercisePickerState
import com.intrada.android.ui.LinkSectionsScreen
import com.intrada.android.ui.LinkSectionsState
import com.intrada.android.ui.PieceActions
import com.intrada.android.ui.PieceNavigation
import com.intrada.android.ui.PieceScreen
import com.intrada.android.ui.PieceScreenState
import com.intrada.android.ui.SectionFormState
import com.intrada.android.ui.SectionScreen
import com.intrada.shared.BarsInput
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.LibraryItemView
import com.intrada.shared.LinkChange
import com.intrada.shared.SectionChange
import com.intrada.shared.SectionEdit
import com.intrada.shared.SectionKind
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
            PieceScreen(piece, PieceScreenState(piece.sections, editingLinks = true), actions)
        }
    }

    @Test
    fun section() = runTest {
        val piece = populatedPiece()
        captureRoboImage("src/test/snapshots/piece-section.png") {
            SectionScreen(piece, SectionFormState(piece.sections.last()), {}, {}, {})
        }
    }

    @Test
    fun linkSections() = runTest {
        val piece = populatedPiece()
        captureRoboImage("src/test/snapshots/piece-link-sections.png") {
            LinkSectionsScreen(piece, LinkSectionsState(piece.linkedExercises.first()), {}, {})
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

    private val actions =
        PieceActions(
            PieceNavigation({}, {}, {}, {}, {}, {}),
            send = { true },
            onDismissError = {},
        )

    private suspend fun TestScope.populatedPiece(): LibraryItemView {
        val store = startedStore()
        listOf(
                SectionEdit(null, "A1", BarsInput.Typed("1 to 12"), SectionKind.FORM, "54"),
                SectionEdit(null, "Run", BarsInput.Typed("19 to 20"), SectionKind.TROUBLESPOT, ""),
            )
            .forEach {
                store.send(Event.Item(ItemEvent.ChangeSection(PIECE, SectionChange.Save(it))))
            }
        store.send(Event.Item(ItemEvent.ChoosePieceExercises(PIECE, listOf(HANON), emptyList())))
        val run = store.libraryRows.value.single { it.id == PIECE }.sections.last().id
        store.send(
            Event.Item(ItemEvent.ChangePieceLink(PIECE, LinkChange.Set(HANON, true, listOf(run))))
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
