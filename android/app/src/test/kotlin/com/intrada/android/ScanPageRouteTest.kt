package com.intrada.android

import android.app.Activity
import androidx.activity.compose.LocalActivity
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.junit4.StateRestorationTester
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextClearance
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.PageReader
import com.intrada.android.core.Store
import com.intrada.android.core.Ulid
import com.intrada.android.ui.ItemFormState
import com.intrada.android.ui.LibraryAddRoute
import com.intrada.shared.DraftSource
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.PageReading
import com.intrada.shared.PhotoDraft
import com.intrada.shared.RecognisedLine
import com.intrada.shared.RecognitionOutput
import com.intrada.shared.TextDraftField
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
class ScanPageRouteTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun aReadPageFillsTheFormAndThePhotoGoesOnThePiece() = runTest {
        val store = readingStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }
        val photoId = Ulid.generate()

        store.send(Event.Item(ItemEvent.ReadPhoto(photoId)))
        testScheduler.advanceUntilIdle()
        compose.waitForIdle()

        assertEquals("Clair de Lune", titleShown())
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()
        assertEquals(photoId, store.libraryRows.value.single().photoId)
    }

    @Test
    fun aClearedFieldStaysClearAfterTheScreenTurns() = runTest {
        val store = readingStore()
        val restoration = StateRestorationTester(compose)
        val turning =
            object : Activity() {
                override fun isChangingConfigurations() = true
            }
        restoration.setContent {
            CompositionLocalProvider(LocalActivity provides turning) {
                LibraryAddRoute(store, onDone = {})
            }
        }
        store.send(Event.Item(ItemEvent.ReadPhoto(Ulid.generate())))
        testScheduler.advanceUntilIdle()
        compose.waitForIdle()
        compose.onNodeWithTag("itemForm.title").performTextClearance()

        restoration.emulateSavedInstanceStateRestore()

        assertEquals("", titleShown())
    }

    @Test
    fun aRescanReplacesWhatThePreviousPageFilled() {
        val form = ItemFormState()
        form.fill(draft("Clair de Lune"))
        form.fill(draft("Reverie"))

        assertEquals("Reverie", form.title)
    }

    private fun titleShown(): String? =
        compose
            .onNodeWithTag("itemForm.title")
            .fetchSemanticsNode()
            .config
            .getOrNull(SemanticsProperties.EditableText)
            ?.text

    private fun draft(title: String) =
        PhotoDraft(title = TextDraftField(title, DraftSource.RECOGNISED, 0.9f, false))

    private suspend fun TestScope.readingStore(): Store {
        val line = RecognisedLine("Clair de Lune", 0.2f, 0.1f, 0.6f, 0.05f, 0.95f)
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(emptyList()),
                this,
                StandardTestDispatcher(testScheduler),
                log = {},
                pageReader = PageReader { RecognitionOutput.Page(PageReading(listOf(line), null)) },
            )
        store.send(Event.StartApp)
        store.settle()
        return store
    }
}
