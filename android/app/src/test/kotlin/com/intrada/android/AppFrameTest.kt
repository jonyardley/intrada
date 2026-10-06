package com.intrada.android

import androidx.activity.ComponentActivity
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.AppFrame
import com.intrada.android.ui.AppTab
import com.intrada.shared.Event
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

    private fun pressBack() {
        compose.runOnUiThread { compose.activity.onBackPressedDispatcher.onBackPressed() }
        compose.waitForIdle()
    }

    private suspend fun TestScope.show(items: InMemoryItemStore = InMemoryItemStore()) {
        val store =
            Store(LiveBridge(), items, this, StandardTestDispatcher(testScheduler), log = {})
        store.send(Event.StartApp)
        store.settle()
        compose.setContent { AppFrame(store) }
    }
}
