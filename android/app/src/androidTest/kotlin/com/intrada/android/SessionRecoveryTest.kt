package com.intrada.android

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isSelected
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * A practice the app was closed on comes back on the next launch (#962), with the item-complete
 * sheet's answers (#2137). iPhone: SessionRecoveryUITests. Each test starts from an empty store and
 * no saved settings, so a session left by an earlier run can neither pass nor fail it.
 */
@RunWith(AndroidJUnit4::class)
class SessionRecoveryTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun aClosedSessionOffersResumeOnRelaunch() {
        startClairDeLune()
        app.control("player.skip")

        app.resumeAfterRelaunch()

        app.control("player.skip")
        app.endSessionEarly()
        app.discardSummary()
    }

    @Test
    fun aClosedSheetComesBackWithItsAnswers() {
        startClairDeLune()
        app.tap("player.advance")
        app.tap("reflection.mark.7")
        app.await(markSeven)

        // Each answer goes through a close on its own: every send carries the whole sheet, so a
        // later one would keep an earlier one that never sent.
        app.resumeAfterRelaunch()
        app.await(markSeven)
        val prefilled = checkNotNull(tempo()) { "the sheet shows a tempo" }
        app.tap("reflection.tempo.increase")
        val setTempo = checkNotNull(tempo()) { "the sheet still shows a tempo" }
        assertNotEquals("the step moved the tempo", prefilled, setTempo)

        app.resumeAfterRelaunch()
        assertEquals("the tempo set by hand survived", setTempo, tempo())
        app.await(markSeven)

        app.control("reflection.note").performTextInput("Pedal clearer in bar 12")
        app.resumeAfterRelaunch()
        app.await(hasTestTag("reflection.note") and hasText("Pedal clearer in bar 12"))
        app.await(markSeven)

        app.tap("reflection.skip")
        app.discardSummary()
    }

    private val markSeven = hasTestTag("reflection.mark.7") and isSelected()

    private fun startClairDeLune() {
        app.launchWithOnePiece("Clair de Lune")
        app.tap("practice.startHere.next")
        app.startWith("Clair de Lune")
    }

    private fun tempo(): String? =
        app.control("reflection.tempo")
            .fetchSemanticsNode()
            .config
            .getOrElseNullable(SemanticsProperties.Text) { null }
            ?.joinToString()
}
