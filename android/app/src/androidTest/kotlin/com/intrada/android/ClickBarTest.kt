package com.intrada.android

import androidx.compose.ui.test.hasStateDescription
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.test.espresso.Espresso
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * The bar line appears only while the click sounds, and the sheet it opens changes which beats
 * sound. The reflection naming the beat unit (#1499) is not on Android yet: #2571.
 */
@RunWith(AndroidJUnit4::class)
class ClickBarTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun theBarFollowsTheSheet() {
        app.launch(seed = true)
        app.startOneItemSession()

        app.control("click.toggle")
        app.assertNeverShows(
            hasTestTag("click.bar"),
            withinMillis = 1_000,
            "a bar line while silent",
        )
        app.tap("click.toggle")
        bar("4 crotchet beats, metronome on every beat")

        app.tap("click.bar")
        app.tap("clickSheet.pattern.downbeat")
        app.tap("clickSheet.done")
        bar("4 crotchet beats, metronome on beat 1")

        app.tap("click.bar")
        app.tap("clickSheet.metre.6-8")
        app.tap("clickSheet.done")
        bar("6 quaver beats, metronome on every beat")

        app.tap("click.toggle")
        app.assertGone(hasTestTag("click.bar"), "the bar line leaves with the click")

        app.tap("player.advance")
        app.control("reflection.tempo.increase")

        // A stray back must not throw the marks and the note away (#1934).
        Espresso.pressBack()
        app.assertNeverShows(
            hasTestTag("click.toggle"),
            withinMillis = 1_000,
            "back left the item-complete page",
        )
        app.control("reflection.skip")
    }

    private fun bar(spoken: String) =
        app.await(hasTestTag("click.bar") and hasStateDescription(spoken))
}
