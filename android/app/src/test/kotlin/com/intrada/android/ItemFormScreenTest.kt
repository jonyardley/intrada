package com.intrada.android

import androidx.compose.ui.test.assertContentDescriptionContains
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.LibraryAddRoute
import com.intrada.android.ui.LibraryEditRoute
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ItemFormScreenTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun addingAPieceClosesTheFormAndLandsInTheLibrary() = runTest {
        val store = startedStore()
        var done = false
        compose.setContent { LibraryAddRoute(store, onDone = { done = true }) }

        compose.onNodeWithTag("itemForm.title").performTextInput("Nocturne in E flat")
        compose.onNodeWithTag("itemForm.composer").performTextInput("Chopin")
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertTrue(done)
        val added = store.libraryRows.value.single()
        assertEquals("Nocturne in E flat", added.title)
        assertEquals("Chopin", added.subtitle)
        assertEquals(ItemKind.PIECE, added.itemType)
    }

    @Test
    fun saveWaitsForATitle() = runTest {
        val store = startedStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }

        compose.onNodeWithTag("itemForm.confirm").assertIsNotEnabled()
        compose.onNodeWithTag("itemForm.title").performTextInput("   ")
        compose.onNodeWithTag("itemForm.confirm").assertIsNotEnabled()
        compose.onNodeWithTag("itemForm.title").performTextInput("Arpeggios")
        compose.onNodeWithTag("itemForm.confirm").assertIsEnabled()
    }

    @Test
    fun aRefusedAddStaysOpenWithTheCoresMessageInline() = runTest {
        val store = startedStore()
        var done = false
        compose.setContent { LibraryAddRoute(store, onDone = { done = true }) }

        compose.onNodeWithTag("itemForm.title").performTextInput("Arpeggios")
        compose.onNodeWithTag("itemForm.bpm").performTextInput("999")
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertFalse(done)
        compose
            .onNodeWithTag("itemForm.error")
            .assertContentDescriptionContains(BPM_REFUSAL, substring = true)
        assertNull(store.viewModel.value?.error)
        assertTrue(store.libraryRows.value.isEmpty())
    }

    @Test
    fun anExerciseAddsWithItsTypedVariations() = runTest {
        val store = startedStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }

        compose.onNodeWithTag("itemForm.kind.exercise").performClick()
        compose.onNodeWithTag("itemForm.title").performTextInput("Arpeggios")
        compose.onNodeWithTag("itemForm.variation.input").performTextInput("Legato")
        compose.onNodeWithTag("itemForm.variation.add").performClick()
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        val added = store.libraryRows.value.single()
        assertEquals(ItemKind.EXERCISE, added.itemType)
        assertEquals(listOf("Legato"), added.variations.map { it.label })
    }

    @Test
    fun anEditLoadsTheItemAndSavesFieldsAndVariationsTogether() = runTest {
        val store = startedStore()
        store.send(Event.Item(ItemEvent.Add(Fixtures.scales)))
        store.settle()
        val id = store.libraryRows.value.single().id
        var done = false
        compose.setContent { LibraryEditRoute(store, id, onDone = { done = true }) }

        compose.onAllNodesWithTag("itemForm.variation.remove").fetchSemanticsNodes().let {
            assertEquals(2, it.size)
        }
        compose.onAllNodesWithTag("itemForm.variation.remove")[0].performScrollTo().performClick()
        compose.onNodeWithTag("itemForm.variation.input").performTextInput("Staccato")
        compose.onNodeWithTag("itemForm.variation.add").performClick()
        compose.onNodeWithTag("itemForm.title").performTextClearance()
        compose.onNodeWithTag("itemForm.title").performTextInput("Scales in sixths")
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertTrue(done)
        val saved = store.libraryRows.value.single()
        assertEquals("Scales in sixths", saved.title)
        assertEquals(listOf("Swung", "Staccato"), saved.variations.map { it.label })
        assertEquals(listOf("warm-up"), saved.tags)
    }

    @Test
    fun aRefusedEditKeepsTheItemAsItWas() = runTest {
        val store = startedStore()
        store.send(Event.Item(ItemEvent.Add(Fixtures.scales)))
        store.settle()
        val id = store.libraryRows.value.single().id
        var done = false
        compose.setContent { LibraryEditRoute(store, id, onDone = { done = true }) }

        compose.onNodeWithTag("itemForm.title").performTextClearance()
        compose.onNodeWithTag("itemForm.title").performTextInput("Renamed")
        compose.onNodeWithTag("itemForm.bpm").performTextClearance()
        compose.onNodeWithTag("itemForm.bpm").performTextInput("999")
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertFalse(done)
        compose
            .onNodeWithTag("itemForm.error")
            .assertContentDescriptionContains(BPM_REFUSAL, substring = true)
        assertEquals("Scales in thirds", store.libraryRows.value.single().title)
    }

    @Test
    fun switchingAnExerciseToAPieceAndBackKeepsItsVariations() = runTest {
        val store = startedStore()
        store.send(Event.Item(ItemEvent.Add(Fixtures.scales)))
        store.settle()
        val id = store.libraryRows.value.single().id
        compose.setContent { LibraryEditRoute(store, id, onDone = {}) }

        compose.onNodeWithTag("itemForm.kind.piece").performClick()
        compose.onNodeWithTag("itemForm.kind.exercise").performClick()
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertEquals(
            listOf("Slow", "Swung"),
            store.libraryRows.value.single().variations.map { it.label },
        )
    }

    private suspend fun TestScope.startedStore(): Store {
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(emptyList()),
                this,
                StandardTestDispatcher(testScheduler),
                log = {},
            )
        store.send(Event.StartApp)
        store.settle()
        return store
    }

    private companion object {
        const val BPM_REFUSAL = "BPM must be a whole number between"
    }
}
