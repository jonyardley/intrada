package com.intrada.android

import androidx.activity.ComponentActivity
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.assertContentDescriptionEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.click
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTouchInput
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.ItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.AppFrame
import com.intrada.android.ui.AppTab
import com.intrada.shared.Event
import com.intrada.shared.FirstRunEvent
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class AppFrameTest {
    @get:Rule val compose = createAndroidComposeRule<ComponentActivity>()

    @Test
    fun eachTabShowsItsTitle() = runTest {
        show()
        compose.onNodeWithText("Library").assertIsDisplayed()
        compose.onNodeWithTag(AppTab.LIBRARY.tag).assertIsSelected()

        AppTab.entries.reversed().forEach { tab ->
            compose.onNodeWithTag(tab.tag).performClick()
            compose.onNodeWithText(tab.label).assertIsDisplayed()
            compose.onNodeWithTag(tab.tag).assertIsSelected()
        }
    }

    @Test
    fun backFromPracticeLandsOnLibrary() = runTest {
        show()
        compose.onNodeWithTag(AppTab.PRACTICE.tag).performClick()
        compose.onNodeWithText("Practice").assertIsDisplayed()

        pressBack()

        compose.onNodeWithText("Library").assertIsDisplayed()
        compose.onNodeWithTag(AppTab.LIBRARY.tag).assertIsSelected()
        assertFalse(compose.activity.isFinishing)
    }

    @Test
    fun backAfterTwoTabsLandsOnLibrary() = runTest {
        show()
        compose.onNodeWithTag(AppTab.PRACTICE.tag).performClick()
        compose.onNodeWithTag(AppTab.ROUTINES.tag).performClick()
        compose.onNodeWithText("Routines").assertIsDisplayed()

        pressBack()

        compose.onNodeWithText("Library").assertIsDisplayed()
        compose.onNodeWithTag(AppTab.LIBRARY.tag).assertIsSelected()
    }

    @Test
    fun backFromLibraryLeavesTheApp() = runTest {
        show()

        pressBack()

        assertTrue(compose.activity.isFinishing)
    }

    @Test
    fun aPieceOpensItsPageAndAnExerciseOpensTheForm() = runTest {
        show(InMemoryItemStore(Fixtures.library))

        compose
            .onNodeWithContentDescription("Piece, Clair de Lune", substring = true)
            .performClick()
        compose.onNodeWithTag("piece.edit").assertIsDisplayed()

        pressBack()
        compose
            .onNodeWithContentDescription("Exercise, Hanon No. 1", substring = true)
            .performClick()
        compose.onNodeWithTag("itemForm.confirm").assertIsDisplayed()
    }

    @Test
    fun startingTheBuiltSessionOpensThePlayerOverTheTabs() = runTest {
        val store = openedStore()
        compose.setContent { AppFrame(store) }
        compose.onNodeWithTag(AppTab.PRACTICE.tag).performClick()
        compose.onNodeWithTag("practice.start").performClick()
        store.send(Event.Session(SessionEvent.AddToSetlist(BuilderFixtures.SATIE)))

        compose.onNodeWithTag("builder.start").performClick()

        compose.onNodeWithTag("player.title").assertTextEquals("Gymnopédie No. 1")
        compose.onNodeWithTag(AppTab.PRACTICE.tag).assertDoesNotExist()
    }

    @Test
    fun aTapWhereATabSitsUnderThePlayerLeavesTheTabsAlone() = runTest {
        val store = openedStore()
        compose.setContent { AppFrame(store) }
        compose.onNodeWithTag(AppTab.PRACTICE.tag).performClick()
        val library = compose.onNodeWithTag(AppTab.LIBRARY.tag).fetchSemanticsNode().boundsInRoot
        compose.onNodeWithTag("practice.start").performClick()
        store.send(Event.Session(SessionEvent.AddToSetlist(BuilderFixtures.SATIE)))
        compose.onNodeWithTag("builder.start").performClick()

        compose.onRoot().performTouchInput { click(library.center) }

        compose.onNodeWithTag("player.title").assertIsDisplayed()
        store.send(
            Event.Session(
                SessionEvent.EndSessionEarly(PlayerFixtures.STARTED, PlayerFixtures.silent)
            )
        )
        store.send(Event.Session(SessionEvent.DiscardSession))
        compose.onNodeWithTag(AppTab.PRACTICE.tag).assertIsSelected()
    }

    @Test
    fun theUndoOfADeleteStaysUpOnEveryTab() = runTest {
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(),
                this,
                StandardTestDispatcher(testScheduler),
                log = {},
            )
        store.send(Event.LoadSampleData)
        store.send(Event.FirstRun(FirstRunEvent.SkipWelcome))
        store.settle()
        compose.setContent { AppFrame(store) }
        compose.onNodeWithTag(AppTab.PRACTICE.tag).performClick()
        store.send(
            Event.Session(SessionEvent.DeleteSaved("sample-session-3d", PlayerFixtures.STARTED))
        )
        store.settle()

        compose.onNodeWithTag(AppTab.LIBRARY.tag).performClick()

        val banner = compose.onNodeWithTag("banner.undoDelete").assertIsDisplayed()
        val title = compose.onNodeWithText("Library").fetchSemanticsNode().boundsInRoot
        assertTrue(banner.fetchSemanticsNode().boundsInRoot.bottom <= title.top)
        val undo =
            banner.fetchSemanticsNode().config[SemanticsActions.CustomActions].single {
                it.label == "Undo"
            }
        compose.runOnIdle { undo.action() }
        store.settle()
        compose.onNodeWithTag("banner.undoDelete").assertDoesNotExist()
        assertTrue(store.sessionHistory.value.any { it.id == "sample-session-3d" })
    }

    @Test
    fun aCoreFailureStaysAboveEachScreenUntilDismissed() = runTest {
        val store = sampleStore(FlakyItemStore(failing = true))
        store.send(Event.StartApp)
        store.settle()
        val message = checkNotNull(store.viewModel.value?.error)
        compose.setContent { AppFrame(store) }

        compose.onNodeWithTag("banner.error").assertContentDescriptionEquals(message)
        assertAbove("banner.error", compose.onNodeWithText("Library"))
        compose.onNodeWithTag("library.add").performClick()
        compose.onNodeWithTag("banner.error").assertIsDisplayed()

        compose.runOnIdle { action("banner.error", "Dismiss").action() }
        compose.onNodeWithTag("banner.error").assertDoesNotExist()
    }

    @Test
    fun aFailureSitsAboveTheUndo() = runTest {
        val items = FlakyItemStore()
        val store = sampleStore(items)
        store.send(Event.Session(SessionEvent.DeleteSaved(SAMPLE_SESSION, PlayerFixtures.STARTED)))
        items.failing = true
        store.send(Event.StartApp)
        store.settle()
        compose.setContent { AppFrame(store) }

        assertAbove("banner.error", compose.onNodeWithTag("banner.undoDelete"))
    }

    @Test
    fun theBannersStayUpOverThePlayer() = runTest {
        val items = FlakyItemStore()
        val store = sampleStore(items)
        store.send(Event.Session(SessionEvent.DeleteSaved(SAMPLE_SESSION, PlayerFixtures.STARTED)))
        store.send(Event.Session(SessionEvent.StartFromSuggestion(PlayerFixtures.STARTED)))
        store.settle()
        items.failing = true
        store.send(Event.StartApp)
        store.settle()
        compose.setContent { AppFrame(store) }

        assertAbove("banner.error", compose.onNodeWithTag("player.title"))
        assertAbove("banner.undoDelete", compose.onNodeWithTag("player.title"))
        compose.onNodeWithTag(AppTab.PRACTICE.tag).assertDoesNotExist()
    }

    @Test
    fun theWelcomeHidesTheBannersFromTalkBack() = runTest {
        val items = FlakyItemStore()
        val store =
            Store(LiveBridge(), items, this, StandardTestDispatcher(testScheduler), log = {})
        store.send(Event.StartApp)
        store.settle()
        compose.setContent { AppFrame(store) }
        compose.onNodeWithTag("firstRun.skipWelcome").assertIsDisplayed()

        items.failing = true
        store.send(Event.StartApp)
        store.settle()

        checkNotNull(store.viewModel.value?.error)
        compose.onNodeWithTag("firstRun.skipWelcome").assertIsDisplayed()
        compose.onNodeWithTag("banner.error").assertDoesNotExist()
    }

    private fun assertAbove(tag: String, below: SemanticsNodeInteraction) {
        val banner = compose.onNodeWithTag(tag).assertIsDisplayed().fetchSemanticsNode()
        assertTrue(banner.boundsInRoot.bottom <= below.fetchSemanticsNode().boundsInRoot.top)
    }

    private fun action(tag: String, label: String) =
        compose
            .onNodeWithTag(tag)
            .fetchSemanticsNode()
            .config[SemanticsActions.CustomActions]
            .single { it.label == label }

    private suspend fun TestScope.sampleStore(items: ItemStore = InMemoryItemStore()): Store {
        val store =
            Store(LiveBridge(), items, this, StandardTestDispatcher(testScheduler), log = {})
        store.send(Event.LoadSampleData)
        store.send(Event.FirstRun(FirstRunEvent.SkipWelcome))
        store.settle()
        return store
    }

    private fun pressBack() {
        compose.runOnUiThread { compose.activity.onBackPressedDispatcher.onBackPressed() }
        compose.waitForIdle()
    }

    private suspend fun TestScope.show(items: InMemoryItemStore = InMemoryItemStore()) {
        val store =
            Store(LiveBridge(), items, this, StandardTestDispatcher(testScheduler), log = {})
        store.send(Event.StartApp)
        store.send(Event.FirstRun(FirstRunEvent.SkipWelcome))
        store.settle()
        compose.setContent { AppFrame(store) }
    }
}

private const val SAMPLE_SESSION = "sample-session-3d"

/** Writes land until [failing] is set, then every operation fails as a full disk would. */
private class FlakyItemStore(var failing: Boolean = false) : ItemStore {
    private val inner = InMemoryItemStore()

    override fun run(operation: PersistenceOperation): PersistenceOutput =
        if (failing) error("disk unavailable") else inner.run(operation)
}
