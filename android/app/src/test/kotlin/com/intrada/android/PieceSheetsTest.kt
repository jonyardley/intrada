package com.intrada.android

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.ExercisePickerRoute
import com.intrada.android.ui.KeysSheetScreen
import com.intrada.android.ui.PieceNavigation
import com.intrada.android.ui.PieceRoute
import com.intrada.android.ui.VariationsSheetScreen
import com.intrada.shared.Event
import com.intrada.shared.ItemKind
import com.intrada.shared.LibraryItemView
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class PieceSheetsTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun allMajorSavesTheTwelveMajorKeysOnDone() = runTest {
        val store = startedStore()
        compose.setContent { PieceRoute(store, PIECE, navigation) }

        compose.onNodeWithTag("keys.choose").performScrollTo().performClick()
        compose.onNodeWithTag("keysSheet.allMajor").performClick()
        assertTrue(store.piece().keys.isEmpty())
        compose.onNodeWithTag("sheet.done").performClick()
        store.settle()

        assertEquals(12, store.piece().keys.size)
        compose.onNodeWithContentDescription("C major,", substring = true).assertExists()
    }

    @Test
    fun aRenameSavesStraightAwayAndATickSavesOnDone() = runTest {
        val store = startedStore()
        val first = store.viewModel.value?.variations?.firstOrNull()?.label.orEmpty()
        compose.setContent { PieceRoute(store, PIECE, navigation) }

        compose.onNodeWithTag("variations.choose").performScrollTo().performClick()
        compose.onNodeWithContentDescription(first, substring = true).performTouchInput {
            longClick()
        }
        compose.onNodeWithTag("variationRename.name").performTextClearance()
        compose.onNodeWithTag("variationRename.name").performTextInput("Hands apart")
        compose.onNodeWithTag("variationRename.confirm").performClick()
        store.settle()

        val labels = store.viewModel.value?.variations.orEmpty().map { it.label }
        assertTrue("Hands apart" in labels)
        assertTrue(first !in labels)
        assertTrue(store.piece().variations.isEmpty())

        compose.onNodeWithContentDescription("Hands apart", substring = true).performClick()
        compose.onNodeWithTag("sheet.done").performClick()
        store.settle()

        assertEquals(listOf("Hands apart"), store.piece().variations.map { it.label })
    }

    @Test
    fun doneWithNothingChangedSendsNothing() = runTest {
        val store = startedStore()
        val piece = store.piece()
        val sent = mutableListOf<Event>()
        var closed = 0
        var keys by mutableStateOf(false)
        compose.setContent {
            if (keys) KeysSheetScreen(piece, { sent.add(it).let { null } }) { closed++ }
            else
                VariationsSheetScreen(piece, emptyList(), { sent.add(it).let { null } }) {
                    closed++
                }
        }
        compose.onNodeWithTag("sheet.done").performClick()
        keys = true
        compose.onNodeWithTag("sheet.done").performClick()

        assertEquals(emptyList<Event>(), sent)
        assertEquals(2, closed)
    }

    @Test
    fun anExerciseCreatedInThePickerIsLinkedOnDone() = runTest {
        val store = startedStore()
        var done = false
        compose.setContent { ExercisePickerRoute(store, PIECE, onDone = { done = true }) }

        compose.onNodeWithTag("exercisePicker.create").performClick()
        compose.onNodeWithTag("writtenExercise.title").performTextInput("Chromatic run")
        compose.onNodeWithTag("writtenExercise.done").performClick()
        compose.onNodeWithTag("exercisePicker.written").assertExists()
        assertTrue(store.piece().linkedExercises.isEmpty())
        compose.onNodeWithTag("exercisePicker.done").performClick()
        store.settle()

        assertTrue(done)
        assertEquals(listOf("Chromatic run"), store.piece().linkedExercises.map { it.title })
        assertTrue(
            store.libraryRows.value.any {
                it.title == "Chromatic run" && it.itemType == ItemKind.EXERCISE
            }
        )
    }

    private val navigation = PieceNavigation({}, {}, {}, {}, {}, {})

    private fun Store.piece(): LibraryItemView = libraryRows.value.single { it.id == PIECE }

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
    }
}
