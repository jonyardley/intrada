package com.intrada.android

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.assertHeightIsAtLeast
import androidx.compose.ui.test.click
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isSelected
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.unit.dp
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * The profile round trip (#1692), the icon button's edge (#2318) and the practice defaults (#1915),
 * each read back after a relaunch. iPhone: ProfileUITests. Unseeded, since seed mode keeps nothing
 * and the relaunch is the point.
 */
@RunWith(AndroidJUnit4::class)
class ProfileTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun editSaveAndRelaunchKeepTheProfile() {
        openProfile()
        app.text("Add a name")
        app.tap("profile.edit")

        app.control("profileEdit.name").performTextInput("Jon")
        app.control("profileEdit.instrument").performTextInput("Cel")
        app.tapRow("suggestion.row", "Cello")
        app.tap("profileEdit.highlighter.coral")
        app.tap("profileEdit.save")

        assertProfileReads("Jon", "Cello", "Coral")

        app.tap("profile.edit")
        app.control("profileEdit.name").performTextInput("x".repeat(101))
        app.tap("profileEdit.save")
        app.await(
            hasTestTag("profileEdit.error") and App.mentions("Name must be 100 characters or fewer")
        )
        app.tap("profileEdit.cancel")
        app.assertGone(hasTestTag("profileEdit.cancel"), "Cancel closes the sheet")
        assertProfileReads("Jon", "Cello", "Coral")

        app.relaunch()
        app.tapTab(AppTab.PRACTICE.tag)
        app.text(", Jon · No sessions yet")
        app.tap("practice.profile")
        assertProfileReads("Jon", "Cello", "Coral")
    }

    @Test
    fun changeIconButtonTakesATapAtItsEdge() {
        openProfile()
        app.tap("profile.edit")

        val change = app.control("profileEdit.changeIcon")
        change.assertHeightIsAtLeast(48.dp)
        change.performTouchInput { click(Offset(width / 2f, height * 0.1f)) }
        app.text("All icons")
    }

    @Test
    fun practiceDefaultsSurviveARelaunch() {
        openProfile()
        app.text("10 per item")
        app.tap("profile.repTarget.decrease")
        app.tap("profile.click.twoAndFour")
        app.await(hasTestTag("profile.click.twoAndFour") and isSelected())
        app.text("9 per item")

        app.relaunch()
        app.tapTab(AppTab.PRACTICE.tag)
        app.tap("practice.profile")
        app.text("9 per item")
        app.await(hasTestTag("profile.click.twoAndFour") and isSelected())
    }

    private fun openProfile() {
        app.launchFresh()
        app.tap("firstRun.skipWelcome")
        app.tapTab(AppTab.PRACTICE.tag)
        app.tap("practice.profile")
    }

    private fun assertProfileReads(name: String, instrument: String, colour: String) {
        app.await(hasText(name) and hasText(instrument))
        app.await(
            hasTestTag("profile.highlighter") and hasContentDescription("Highlighter, $colour")
        )
    }
}
