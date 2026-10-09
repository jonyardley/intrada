package com.intrada.android

import android.content.Context
import androidx.compose.ui.test.assertContentDescriptionContains
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import com.intrada.android.core.Settings
import com.intrada.android.ui.PracticeRoute
import com.intrada.android.ui.ProfileRoute
import com.intrada.shared.HighlighterColour
import com.intrada.shared.InstrumentIcon
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ProfileScreenTest {
    @get:Rule val compose = createComposeRule()

    private val prefs =
        RuntimeEnvironment.getApplication()
            .getSharedPreferences(Settings.PREFERENCES, Context.MODE_PRIVATE)
    private val practice =
        RuntimeEnvironment.getApplication()
            .getSharedPreferences(Settings.PRACTICE_PREFERENCES, Context.MODE_PRIVATE)

    @Test
    fun aNameAndInstrumentSavedOnceAreBackAtTheNextLaunch() = runTest {
        val store = openedStore(settings = Settings(prefs, practice))
        compose.setContent { ProfileRoute(store) }

        compose.onNodeWithTag("profile.edit").performClick()
        compose.onNodeWithTag("profileEdit.name").performTextInput("Clara")
        compose.onNodeWithTag("profileEdit.instrument").performTextInput("Cello")
        compose.onNodeWithTag("profileEdit.highlighter.mint").performClick()
        compose.onNodeWithTag("profileEdit.save").performClick()
        store.settle()

        compose.onNodeWithTag("profile.edit").assertExists()
        val next = openedStore(settings = Settings(prefs, practice))
        next.restoreSettings()
        val profile = checkNotNull(next.viewModel.value).profile
        assertEquals("Clara", profile.name)
        assertEquals("Cello", profile.instrument)
        assertEquals(HighlighterColour.MINT, profile.colour)
    }

    @Test
    fun aRefusedSaveStaysOpenWithTheCoresMessageOnTheField() = runTest {
        val store = openedStore()
        compose.setContent { ProfileRoute(store) }

        compose.onNodeWithTag("profile.edit").performClick()
        compose.onNodeWithTag("profileEdit.name").performTextInput("a".repeat(101))
        compose.onNodeWithTag("profileEdit.save").performClick()
        store.settle()

        compose
            .onNodeWithTag("profileEdit.error")
            .assertContentDescriptionContains("Name must be", substring = true)
        compose
            .onNodeWithTag("profileEdit.name")
            .assertContentDescriptionContains("The message at the top", substring = true)
        assertNull(store.viewModel.value?.error)
        assertEquals("", checkNotNull(store.viewModel.value).profile.name)
    }

    @Test
    fun cancelLeavesTheProfileAsItWas() = runTest {
        val store = openedStore()
        compose.setContent { ProfileRoute(store) }

        compose.onNodeWithTag("profile.edit").performClick()
        compose.onNodeWithTag("profileEdit.name").performTextInput("Clara")
        compose.onNodeWithTag("profileEdit.cancel").performClick()

        compose.onNodeWithTag("profile.edit").assertExists()
        assertEquals("", checkNotNull(store.viewModel.value).profile.name)
    }

    @Test
    fun anIconPickedOtherThanTheMatchIsPinnedAndTheMatchUnpinsIt() = runTest {
        val store = openedStore()
        compose.setContent { ProfileRoute(store) }

        compose.onNodeWithTag("profile.edit").performClick()
        compose.onNodeWithTag("profileEdit.instrument").performTextInput("Cello")
        compose.onNodeWithTag("profileEdit.save").performClick()
        compose.onNodeWithTag("profile.edit").performClick()
        compose.onNodeWithTag("profileEdit.changeIcon").performClick()
        compose.onNodeWithContentDescription("Harp").performScrollTo().performClick()
        compose.onNodeWithContentDescription("Harp").assertIsSelected()
        compose.onNodeWithTag("iconPicker.done").performClick()
        compose.onNodeWithTag("profileEdit.save").performClick()
        assertEquals(InstrumentIcon.HARP, checkNotNull(store.viewModel.value).profile.icon)

        compose.onNodeWithTag("profile.edit").performClick()
        compose.onNodeWithTag("profileEdit.changeIcon").performClick()
        compose.onNodeWithContentDescription("Cello and double bass").performClick()
        compose.onNodeWithTag("iconPicker.done").performClick()
        compose.onNodeWithTag("profileEdit.save").performClick()
        val profile = checkNotNull(store.viewModel.value).profile
        assertEquals(InstrumentIcon.CELLO, profile.icon)
        assertEquals(false, profile.iconChosen)
    }

    @Test
    fun anInstrumentSuggestionFillsTheField() = runTest {
        val store = openedStore()
        compose.setContent { ProfileRoute(store) }

        compose.onNodeWithTag("profile.edit").performClick()
        compose.onNodeWithTag("profileEdit.instrument").performTextInput("violi")
        compose.onNodeWithText("Violin").performClick()
        compose.onNodeWithTag("profileEdit.save").performClick()

        assertEquals("Violin", checkNotNull(store.viewModel.value).profile.instrument)
    }

    @Test
    fun aPracticeDefaultChangedHereIsTheCoresNewDefault() = runTest {
        val store = openedStore()
        val before = checkNotNull(store.viewModel.value).practiceDefaults.repTarget
        compose.setContent { ProfileRoute(store) }

        compose.onNodeWithTag("profile.repTarget.decrease").performScrollTo().performClick()

        assertEquals(
            (before - 1u).toUByte(),
            checkNotNull(store.viewModel.value).practiceDefaults.repTarget,
        )
    }

    @Test
    fun theBadgeInThePracticeHeaderOpensTheProfile() = runTest {
        val store = openedStore()
        var opened = false
        compose.setContent { PracticeRoute(store, onBuild = {}, onProfile = { opened = true }) }

        compose.onNodeWithTag("practice.profile").performClick()

        assertTrue(opened)
    }
}
