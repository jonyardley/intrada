package com.intrada.android

import android.content.Context
import androidx.activity.ComponentActivity
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.StateRestorationTester
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.Settings
import com.intrada.android.ui.AppFrame
import com.intrada.android.ui.AppTab
import com.intrada.shared.ItemKind
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class FirstRunTest {
    @get:Rule val compose = createAndroidComposeRule<ComponentActivity>()

    private val settings
        get() =
            Settings(
                RuntimeEnvironment.getApplication()
                    .getSharedPreferences(Settings.PREFERENCES, Context.MODE_PRIVATE),
                RuntimeEnvironment.getApplication()
                    .getSharedPreferences(Settings.PRACTICE_PREFERENCES, Context.MODE_PRIVATE),
            )

    @Test
    fun skippingTheWelcomeKeepsItAwayAfterARelaunch() = runTest {
        val store = openedStore(InMemoryItemStore(), settings)
        compose.setContent { AppFrame(store) }

        compose.onNodeWithTag("firstRun.skipWelcome").performClick()
        store.settle()

        compose.onNodeWithTag(AppTab.LIBRARY.tag).assertIsSelected()
        val next = openedStore(InMemoryItemStore(), settings)
        assertTrue(checkNotNull(next.viewModel.value).firstRun.showsWelcome)
        next.restoreSettings()
        assertFalse(checkNotNull(next.viewModel.value).firstRun.showsWelcome)
    }

    @Test
    fun aSavedProfileMovesOnToTheFirstPiece() = runTest {
        val store = openedStore(InMemoryItemStore())
        compose.setContent { AppFrame(store) }

        compose.onNodeWithTag("firstRun.setUpProfile").performClick()
        compose.onNodeWithTag("profileEdit.name").performTextInput("Clara")
        compose.onNodeWithTag("firstRun.saveProfile").performClick()
        store.settle()

        assertEquals("Clara", checkNotNull(store.viewModel.value).profile.name)
        compose.onNodeWithTag("firstRun.typePiece").assertExists()
    }

    @Test
    fun aRefusedProfileStaysOnTheProfileStep() = runTest {
        val store = openedStore(InMemoryItemStore())
        compose.setContent { AppFrame(store) }

        compose.onNodeWithTag("firstRun.setUpProfile").performClick()
        compose.onNodeWithTag("profileEdit.name").performTextInput("a".repeat(101))
        compose.onNodeWithTag("firstRun.saveProfile").performClick()
        store.settle()

        compose.onNodeWithTag("firstRun.profileError").assertExists()
        compose.onNodeWithTag("firstRun.typePiece").assertDoesNotExist()
    }

    @Test
    fun backFromTheProfileReturnsToTheWelcome() = runTest {
        val store = openedStore(InMemoryItemStore())
        compose.setContent { AppFrame(store) }

        compose.onNodeWithTag("firstRun.setUpProfile").performClick()
        compose.onNodeWithTag("firstRun.saveProfile").assertExists()
        pressBack()

        compose.onNodeWithTag("firstRun.skipWelcome").assertExists()
        assertFalse(compose.activity.isFinishing)
    }

    @Test
    fun addingAFirstExerciseOpensPractice() = runTest {
        val store = openedStore(InMemoryItemStore())
        compose.setContent { AppFrame(store) }

        compose.onNodeWithTag("firstRun.setUpProfile").performClick()
        compose.onNodeWithTag("firstRun.skipProfile").performClick()
        compose.onNodeWithTag("firstRun.addExercise").performClick()
        compose.onNodeWithTag("itemForm.title").performTextInput("Arpeggios")
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        compose.onNodeWithTag(AppTab.PRACTICE.tag).assertIsSelected()
        assertEquals("Arpeggios", store.libraryRows.value.single().title)
        assertEquals(ItemKind.EXERCISE, store.libraryRows.value.single().itemType)
    }

    @Test
    fun skippingTheFirstPieceLeavesTheLibraryEmpty() = runTest {
        val store = openedStore(InMemoryItemStore())
        compose.setContent { AppFrame(store) }

        compose.onNodeWithTag("firstRun.setUpProfile").performClick()
        compose.onNodeWithTag("firstRun.skipProfile").performClick()
        compose.onNodeWithTag("firstRun.skipFirstPiece").performClick()
        store.settle()

        compose.onNodeWithTag(AppTab.LIBRARY.tag).assertIsSelected()
        compose.onNodeWithTag("firstRun.typePiece").assertDoesNotExist()
        assertTrue(store.libraryRows.value.isEmpty())
    }

    @Test
    fun backFromTheAddFormReturnsToTheFirstPiece() = runTest {
        val store = openedStore(InMemoryItemStore())
        compose.setContent { AppFrame(store) }

        compose.onNodeWithTag("firstRun.setUpProfile").performClick()
        compose.onNodeWithTag("firstRun.skipProfile").performClick()
        compose.onNodeWithTag("firstRun.addExercise").performClick()
        compose.onNodeWithTag("itemForm.title").assertExists()
        pressBack()

        compose.onNodeWithTag("firstRun.addExercise").assertExists()
        assertFalse(compose.activity.isFinishing)
    }

    @Test
    fun theFirstPieceStepSurvivesARestoreAfterTheProfileSaves() = runTest {
        val store = openedStore(InMemoryItemStore())
        val restorer = StateRestorationTester(compose)
        restorer.setContent { AppFrame(store) }

        compose.onNodeWithTag("firstRun.setUpProfile").performClick()
        compose.onNodeWithTag("profileEdit.name").performTextInput("Clara")
        compose.onNodeWithTag("firstRun.saveProfile").performClick()
        store.settle()
        assertFalse(checkNotNull(store.viewModel.value).firstRun.showsWelcome)
        restorer.emulateSavedInstanceStateRestore()

        compose.onNodeWithTag("firstRun.typePiece").assertExists()
    }

    private fun pressBack() {
        compose.runOnUiThread { compose.activity.onBackPressedDispatcher.onBackPressed() }
        compose.waitForIdle()
    }
}
