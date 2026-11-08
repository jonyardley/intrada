package com.intrada.android

import androidx.compose.runtime.remember
import androidx.compose.ui.test.assertIsOff
import androidx.compose.ui.test.assertIsOn
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.ExercisePickerRoute
import com.intrada.android.ui.ItemFormState
import com.intrada.android.ui.NO_LONGER_THERE
import com.intrada.android.ui.PieceActions
import com.intrada.android.ui.PieceNavigation
import com.intrada.android.ui.PieceRoute
import com.intrada.android.ui.PieceScreen
import com.intrada.android.ui.PieceScreenState
import com.intrada.android.ui.sendAccepted
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.LibraryItemView
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class PieceScreenTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun tickingAnExerciseLinksItToThePiece() = runTest {
        val store = startedStore()
        var done = false
        compose.setContent { ExercisePickerRoute(store, PIECE, onDone = { done = true }) }

        compose.onNodeWithContentDescription("Hanon No. 1").assertIsOff().performClick()
        compose.onNodeWithContentDescription("Hanon No. 1").assertIsOn()
        compose.onNodeWithTag("exercisePicker.done").performClick()
        store.settle()

        assertTrue(done)
        assertEquals(listOf(HANON), store.piece().linkedExercises.map { it.id })
    }

    @Test
    fun editingRelatedExercisesMovesAndUnlinksOneAtATime() = runTest {
        val store = startedStore()
        store.send(Event.Item(ItemEvent.Add(Fixtures.scales)))
        store.settle()
        val scales = store.libraryRows.value.single { it.title == Fixtures.scales.title }.id
        store.link(HANON, scales)
        compose.setContent { PieceRoute(store, PIECE, navigation()) }

        compose.onNodeWithTag("relatedExercises.header").performScrollTo().performClick()
        compose
            .onNodeWithContentDescription("Move Hanon No. 1 down")
            .performScrollTo()
            .performClick()
        store.settle()
        assertEquals(listOf(scales, HANON), store.piece().linkedExercises.map { it.id })

        compose
            .onNodeWithContentDescription("Remove Scales in thirds from related exercises")
            .performScrollTo()
            .performClick()
        store.settle()
        assertEquals(listOf(HANON), store.piece().linkedExercises.map { it.id })
    }

    @Test
    fun deletingThePieceAsksThenCloses() = runTest {
        val store = startedStore()
        var closed = false
        compose.setContent { PieceRoute(store, PIECE, navigation(onClosed = { closed = true })) }

        compose.onNodeWithTag("piece.delete").performScrollTo().performClick()
        assertFalse(closed)
        compose.onNodeWithTag("piece.delete.confirm").performClick()
        store.settle()

        assertTrue(closed)
        assertTrue(store.libraryRows.value.none { it.id == PIECE })
    }

    @Test
    fun aRefusedDeleteLeavesThePageOpen() = runTest {
        val store = startedStore()
        val stale = store.piece()
        store.send(Event.Item(ItemEvent.Delete(PIECE)))
        store.settle()
        var closed = false
        compose.setContent {
            PieceScreen(
                stale,
                remember { PieceScreenState() },
                PieceActions(
                    navigation(onClosed = { closed = true }),
                    send = { store.sendAccepted(it) },
                    sendFromSheet = { null },
                ),
            )
        }

        compose.onNodeWithTag("piece.delete").performScrollTo().performClick()
        compose.onNodeWithTag("piece.delete.confirm").performClick()
        store.settle()

        assertFalse(closed)
        assertNotNull(store.viewModel.value?.error)
    }

    @Test
    fun aPieceTurnedIntoAnExerciseIsNoLongerOnThePiecePage() = runTest {
        val store = startedStore()
        compose.setContent { PieceRoute(store, PIECE, navigation()) }
        compose.onNodeWithTag("piece.delete").assertExists()

        store.send(
            ItemFormState.of(store.piece()).apply { kind = ItemKind.EXERCISE }.editEvent(PIECE)
        )
        store.settle()

        assertEquals(ItemKind.EXERCISE, store.piece().itemType)
        compose.onNodeWithText(NO_LONGER_THERE).assertExists()
        compose.onNodeWithTag("piece.delete").assertDoesNotExist()
    }

    @Test
    fun linksSurviveAReload() = runTest {
        val disk = InMemoryItemStore(Fixtures.library)
        val store = startedStore(disk)
        var done = false
        compose.setContent { ExercisePickerRoute(store, PIECE, onDone = { done = true }) }

        compose.onNodeWithContentDescription("Hanon No. 1").performClick()
        compose.onNodeWithTag("exercisePicker.done").performClick()
        store.settle()
        assertTrue(done)

        val reloaded = startedStore(disk)
        assertEquals(listOf(HANON), reloaded.piece().linkedExercises.map { it.id })
    }

    private fun navigation(onClosed: () -> Unit = {}) =
        PieceNavigation(
            onEdit = {},
            onAddExercises = {},
            onOpenExercise = {},
            onClosed = onClosed,
        )

    private fun Store.piece(): LibraryItemView = libraryRows.value.single { it.id == PIECE }

    private suspend fun Store.link(vararg exerciseIds: String) {
        send(Event.Item(ItemEvent.ChoosePieceExercises(PIECE, exerciseIds.toList(), emptyList())))
        settle()
    }

    private suspend fun TestScope.startedStore(
        disk: InMemoryItemStore = InMemoryItemStore(Fixtures.library)
    ): Store {
        val store =
            Store(
                LiveBridge(),
                disk,
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
