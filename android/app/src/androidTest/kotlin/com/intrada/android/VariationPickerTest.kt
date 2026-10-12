package com.intrada.android

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasStateDescription
import androidx.compose.ui.test.hasTestTag
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
        app.tap("player.gotIt")
        app.tap("player.variation")
        app.tapRow("variationPicker.row", "Hands separately")
        app.tap("player.gotIt")
        app.tap("player.advance")

        // Two different marks, so one write cannot satisfy both.
        pill("Mark for No variation, 7").performClick()
        pill("Mark for Hands separately, 3").performClick()
        app.tap("reflection.save")
        app.control("summary.save")

        pill("Mark for Major Scales, 7").assertIsSelected()
        pill("Mark for Hands separately, 3").assertIsSelected()
    }

    @Test
    fun gotItCountsPastTheTargetAndUndoTakesItBack() {
        startScalesSession()
        val reps = app.control("player.reps")
        val target =
            reps
                .fetchSemanticsNode()
                .config
                .getOrElseNullable(SemanticsProperties.StateDescription) { null }
                ?.substringAfterLast(" of ")
                ?.takeWhile { it.isDigit() }
                ?.toIntOrNull()
        checkNotNull(target) { "the counter names its target" }
        app.assertNeverShows(
            hasTestTag("player.undo"),
            withinMillis = 1_000,
            "nothing to undo before a tap",
        )

        repeat(target + 1) { app.tap("player.gotIt") }
        app.await(
            hasTestTag("player.reps") and hasStateDescription("${target + 1} of $target, 1 extra")
        )

        app.tap("player.undo")
        app.await(hasTestTag("player.reps") and hasStateDescription("$target of $target"))
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

    private fun pill(spoken: String) = app.await(hasContentDescription(spoken)).scrolledTo()
}
