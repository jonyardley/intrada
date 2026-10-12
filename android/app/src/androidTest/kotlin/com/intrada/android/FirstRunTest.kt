package com.intrada.android

import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/** A new install meets the welcome once: through either way out, a relaunch goes straight in. */
@RunWith(AndroidJUnit4::class)
class FirstRunTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun skippingTheProfileIsNotAskedAgain() {
        app.launchFresh()
        app.tap("firstRun.setUpProfile")
        app.tap("firstRun.skipProfile")
        app.tap("firstRun.skipFirstPiece")
        app.control(AppTab.LIBRARY.tag)

        assertRelaunchSkipsTheWelcome()
    }

    @Test
    fun skippingTheWelcomeIsNotAskedAgain() {
        app.launchFresh()
        app.tap("firstRun.skipWelcome")
        app.control(AppTab.LIBRARY.tag)

        assertRelaunchSkipsTheWelcome()
    }

    @Test
    fun savingTheProfileStillOffersTheFirstPiece() {
        app.launchFresh()
        app.tap("firstRun.setUpProfile")
        app.tap("profileEdit.highlighter.mint")
        app.control("profileEdit.name").performTextInput("Jon")
        app.tap("firstRun.saveProfile")
        app.tap("firstRun.skipFirstPiece")

        assertRelaunchSkipsTheWelcome()
    }

    @Test
    fun addingAFirstPieceLandsOnPractice() {
        app.launchFresh()
        app.tap("firstRun.setUpProfile")
        app.tap("firstRun.skipProfile")
        app.tap("firstRun.typePiece")
        app.control("itemForm.title").performTextInput("Clair de Lune")
        app.tap("itemForm.confirm")

        app.control("practice.startHere.next")
        app.control(AppTab.PRACTICE.tag).assertIsSelected()
    }

    private fun assertRelaunchSkipsTheWelcome() {
        app.relaunch()
        app.tapTab(AppTab.PRACTICE.tag)
        app.control("practice.profile")
        // The loads land after the tabs draw, so give the welcome its chance.
        app.assertNeverShows(
            hasTestTag("firstRun.setUpProfile"),
            withinMillis = 3_000,
            "the welcome stays dismissed",
        )
    }
}
