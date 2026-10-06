package com.intrada.android

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.ExercisePickerRoute
import com.intrada.android.ui.LinkSectionsRoute
import com.intrada.android.ui.PieceNavigation
import com.intrada.android.ui.PieceRoute
import com.intrada.android.ui.SectionRoute
import com.intrada.shared.BarsInput
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.LibraryItemView
import com.intrada.shared.SectionChange
import com.intrada.shared.SectionEdit
import com.intrada.shared.SectionKind
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
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
    fun aSectionSavedFromItsScreenLandsOnThePiece() = runTest {
        val store = startedStore()
        var done = false
        compose.setContent { SectionRoute(store, PIECE, null, onDone = { done = true }) }

        compose.onNodeWithTag("sectionSheet.name").performTextInput("A1")
        compose.onNodeWithTag("sectionSheet.bars").performTextInput("1 to 12")
        compose.onNodeWithTag("sectionSheet.kind.spot").performClick()
        compose.onNodeWithTag("sectionSheet.save").performClick()
        store.settle()

        assertTrue(done)
        val section = store.piece().sections.single()
        assertEquals("A1", section.name)
        assertEquals(1, section.firstBar?.toInt())
        assertEquals(12, section.lastBar?.toInt())
        assertEquals(SectionKind.TROUBLESPOT, section.kind)
    }

    @Test
    fun aRefusedSectionStaysOpenWithTheCoresReason() = runTest {
        val store = startedStore()
        var done = false
        compose.setContent { SectionRoute(store, PIECE, null, onDone = { done = true }) }

        compose.onNodeWithTag("sectionSheet.name").performTextInput("A1")
        compose.onNodeWithTag("sectionSheet.bars").performTextInput("twelve")
        compose.onNodeWithTag("sectionSheet.save").performClick()
        store.settle()

        assertFalse(done)
        compose.onNodeWithTag("sectionSheet.error").assertExists()
        assertTrue(store.piece().sections.isEmpty())
    }

    @Test
    fun editingASectionRenamesItWhereItStands() = runTest {
        val store = startedStore()
        store.addSections("A", "B")
        val first = store.piece().sections.first().id
        compose.setContent { SectionRoute(store, PIECE, first, onDone = {}) }

        compose.onNodeWithTag("sectionSheet.name").performTextClearance()
        compose.onNodeWithTag("sectionSheet.name").performTextInput("Intro")
        compose.onNodeWithTag("sectionSheet.save").performClick()
        store.settle()

        assertEquals(listOf("Intro", "B"), store.piece().sections.map { it.name })
    }

    @Test
    fun removingASectionAsksFirst() = runTest {
        val store = startedStore()
        store.addSections("A", "B")
        val first = store.piece().sections.first().id
        var done = false
        compose.setContent { SectionRoute(store, PIECE, first, onDone = { done = true }) }

        compose.onNodeWithTag("sectionSheet.remove").performScrollTo().performClick()
        assertEquals(2, store.piece().sections.size)
        compose.onNodeWithTag("sectionSheet.removal.confirm").performClick()
        store.settle()

        assertTrue(done)
        assertEquals(listOf("B"), store.piece().sections.map { it.name })
    }

    @Test
    fun aReorderSavesOnDoneAndNotBefore() = runTest {
        val store = startedStore()
        store.addSections("A", "B", "C")
        compose.setContent { PieceRoute(store, PIECE, navigation()) }

        compose.onNodeWithTag("sections.header").performClick()
        compose.onNodeWithContentDescription("Move C up").performClick()
        compose.onNodeWithContentDescription("Remove A").performClick()
        assertEquals(listOf("A", "B", "C"), store.piece().sections.map { it.name })
        compose.onNodeWithTag("sections.header").performClick()
        store.settle()

        assertEquals(listOf("C", "B"), store.piece().sections.map { it.name })
        compose.onAllNodesWithTag("sections.row").assertCountEquals(2)
    }

    @Test
    fun tickingAnExerciseLinksItToThePiece() = runTest {
        val store = startedStore()
        var done = false
        compose.setContent { ExercisePickerRoute(store, PIECE, onDone = { done = true }) }

        compose.onNodeWithContentDescription("Hanon No. 1").performClick()
        compose.onNodeWithTag("exercisePicker.done").performClick()
        store.settle()

        assertTrue(done)
        assertEquals(listOf(HANON), store.piece().linkedExercises.map { it.id })
    }

    @Test
    fun choosingSectionsReplacesTheWholePieceLink() = runTest {
        val store = startedStore()
        store.addSections("A", "B")
        store.link(HANON)
        val sectionA = store.piece().sections.first().id
        compose.setContent { LinkSectionsRoute(store, PIECE, HANON, onDone = {}) }

        compose.onNodeWithContentDescription("The whole piece").performClick()
        compose.onNodeWithContentDescription("A").performClick()
        compose.onNodeWithTag("linkSectionsSheet.done").performClick()
        store.settle()

        val linked = store.piece().linkedExercises.single()
        assertFalse(linked.wholePiece)
        assertEquals(listOf(sectionA), linked.sections.map { it.id })
    }

    @Test
    fun editingRelatedExercisesMovesAndUnlinksOneAtATime() = runTest {
        val store = startedStore()
        store.send(Event.Item(ItemEvent.Add(Fixtures.scales)))
        store.settle()
        val scales = store.libraryRows.value.single { it.title == Fixtures.scales.title }.id
        store.link(HANON, scales)
        compose.setContent { PieceRoute(store, PIECE, navigation()) }

        compose.onNodeWithTag("relatedExercises.header").performClick()
        compose.onNodeWithContentDescription("Move Hanon No. 1 down").performClick()
        store.settle()
        assertEquals(listOf(scales, HANON), store.piece().linkedExercises.map { it.id })

        compose
            .onNodeWithContentDescription("Remove Scales in thirds from related exercises")
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

    private fun navigation(onClosed: () -> Unit = {}) =
        PieceNavigation(
            onEdit = {},
            onSection = {},
            onAddExercises = {},
            onChooseSections = {},
            onOpenExercise = {},
            onClosed = onClosed,
        )

    private fun Store.piece(): LibraryItemView = libraryRows.value.single { it.id == PIECE }

    private suspend fun Store.addSections(vararg names: String) {
        names.forEach { name ->
            send(
                Event.Item(
                    ItemEvent.ChangeSection(
                        PIECE,
                        SectionChange.Save(
                            SectionEdit(null, name, BarsInput.Typed(""), SectionKind.FORM, "")
                        ),
                    )
                )
            )
        }
        settle()
    }

    private suspend fun Store.link(vararg exerciseIds: String) {
        send(Event.Item(ItemEvent.ChoosePieceExercises(PIECE, exerciseIds.toList(), emptyList())))
        settle()
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
