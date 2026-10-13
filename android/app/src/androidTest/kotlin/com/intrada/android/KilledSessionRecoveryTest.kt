package com.intrada.android

import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assume.assumeTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * A practice comes back after Android kills the app (#2576). Kept out of the Gradle run:
 * `scripts/android-kill-recovery.sh` runs each step on its own, with `am force-stop` between.
 */
@RunWith(AndroidJUnit4::class)
class KilledSessionRecoveryTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @Test
    fun answerTheSheetBeforeTheKill() {
        assumeTrue(step() == "answer")
        app.startClairDeLune()
        app.tap("player.advance")
        app.tap("reflection.mark.7")
        app.await(markSeven)
        val prefilled = checkNotNull(app.reflectionTempo()) { "the sheet shows a tempo" }
        app.tap("reflection.tempo.increase")
        val setTempo = checkNotNull(app.reflectionTempo()) { "the sheet still shows a tempo" }
        assertNotEquals("the step moved the tempo", prefilled, setTempo)
        app.control("reflection.note").performTextInput("Pedal clearer in bar 12")
        app.await(hasTestTag("reflection.note") and hasText("Pedal clearer in bar 12"))
        expectedTempo().writeText(setTempo)
        // The note sends on a typing pause or when the screen stops: stop it before the kill.
        app.close()
    }

    @Test
    fun resumeWithTheAnswersAfterTheKill() {
        assumeTrue(step() == "resume")
        val setTempo = expectedTempo().readText()
        app.resumeAfterRelaunch()
        app.await(markSeven)
        assertEquals("the tempo set by hand survived the kill", setTempo, app.reflectionTempo())
        app.await(hasTestTag("reflection.note") and hasText("Pedal clearer in bar 12"))

        app.tap("reflection.skip")
        app.discardSummary()
        app.close()
        expectedTempo().delete()
    }

    private fun step() = InstrumentationRegistry.getArguments().getString("killRecovery")

    private fun expectedTempo() =
        File(
            InstrumentationRegistry.getInstrumentation().targetContext.noBackupFilesDir,
            "kill-recovery-tempo",
        )
}
