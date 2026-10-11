package com.intrada.android

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.text.AnnotatedString
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.LibraryAddRoute
import com.intrada.android.ui.LibraryEditRoute
import com.intrada.shared.Event
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ItemFormSuggestionsTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun pickingAComposerFillsTheFieldAndClosesTheList() = runTest {
        val store = startedStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }

        compose.onNodeWithTag("itemForm.composer").performTextInput("Sat")
        compose.onNodeWithText("Erik Satie").performScrollTo().performClick()

        compose.onNodeWithTag("itemForm.composer").assertTextEquals("Erik Satie")
        compose.onNodeWithTag("itemForm.composer").assertIsNotFocused()
    }

    @Test
    fun pickingATagAddsItAndEmptiesTheInput() = runTest {
        val store = startedStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }
        assertEquals(emptyList<String>(), rows())

        compose.onNodeWithTag("itemForm.tag.input").performScrollTo().performTextInput("recital")
        compose.onNodeWithTag("itemForm.tag.add").performScrollTo().performClick()
        compose.onNodeWithTag("itemForm.tag.input").performTextInput("r")

        assertEquals(listOf("romantic"), rows())
        compose.onNodeWithText("romantic").performScrollTo().performClick()

        assertEquals(listOf("recital", "romantic"), chips())
        compose
            .onNodeWithTag("itemForm.tag.input")
            .assert(
                SemanticsMatcher.expectValue(SemanticsProperties.EditableText, AnnotatedString(""))
            )
    }

    @Test
    fun theEditFormSuggestsFromTheLibraryAndLeavesOutTheItemsOwnTags() = runTest {
        val store = startedStore()
        compose.setContent { LibraryEditRoute(store, "01J0000000000000000000SATI", onDone = {}) }

        compose.onNodeWithTag("itemForm.composer").performTextReplacement("Deb")
        assertEquals(listOf("Claude Debussy"), rows())

        compose.onNodeWithTag("itemForm.tag.input").performScrollTo().performClick()
        assertEquals(emptyList<String>(), rows())
    }

    private fun rows() =
        compose.onAllNodesWithTag("suggestion.row").fetchSemanticsNodes().map {
            it.config[SemanticsProperties.Text].joinToString { text -> text.text }
        }

    private fun chips() =
        compose.onAllNodesWithTag("itemForm.tag.chip").fetchSemanticsNodes().map {
            it.config[SemanticsProperties.ContentDescription].single()
        }

    private suspend fun TestScope.startedStore(): Store {
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(library),
                this,
                StandardTestDispatcher(testScheduler),
                log = {},
            )
        store.send(Event.StartApp)
        store.settle()
        return store
    }

    private companion object {
        val library =
            listOf(
                Fixtures.item(tags = listOf("recital")),
                Fixtures.item(
                    id = "01J0000000000000000000SATI",
                    title = "Gymnopédie No. 1",
                    composer = "Erik Satie",
                    tags = listOf("recital", "romantic"),
                ),
            )
    }
}
