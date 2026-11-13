package com.intrada.android

import android.os.SystemClock
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.App.Companion.scrolledTo
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * The player's variation picker against the real core (#1739): the reflection offers a mark per
 * variation, and each keeps its own. "Major Scales" is the seeded exercise whose one variation is
 * "Hands separately"; a play opens plain (#2246).
 */
@RunWith(AndroidJUnit4::class)
class VariationPickerTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun theReflectionOffersAMarkPerVariation() {
        startScalesSession()
        playLongEnoughToMark()
        app.tap("player.variation")
        app.tapRow("variationPicker.row", "Hands separately")
        playLongEnoughToMark()
        app.tap("player.advance")

        // Two different marks, so one write cannot satisfy both.
        pill("Mark for No variation, 7").performClick()
        pill("Mark for Hands separately, 3").performClick()
        app.tap("reflection.save")
        app.control("summary.save")

        pill("Mark for Major Scales, 7").assertIsSelected()
        pill("Mark for Hands separately, 3").assertIsSelected()
    }

    private fun startScalesSession() {
        app.launch(seed = true)
        app.tapTab(AppTab.PRACTICE.tag)
        app.tap("practice.buildOwn")
        app.tap("builder.addItems")
        app.tapRow("addToSession.row", "Major Scales")
        app.tap("addToSession.done")
        app.tap("builder.start")
    }

    // The sheet offers no mark for a play under five seconds: it reads as a stray tap (#1758).
    private fun playLongEnoughToMark() = SystemClock.sleep(PLAY_MILLIS)

    private fun pill(spoken: String) = app.await(hasContentDescription(spoken)).scrolledTo()

    private companion object {
        const val PLAY_MILLIS = 5_500L
    }
}
