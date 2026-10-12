package com.intrada.android

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTouchInput
import androidx.test.espresso.Espresso
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.App.Companion.scrolledTo
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/** The session builder against the real core (#935): queue, remove, cancel, reorder, settings. */
@RunWith(AndroidJUnit4::class)
class SessionBuilderTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun buildAddRemoveThenCancel() {
        app.launch(seed = true)
        app.openEmptyBuilder()
        queueHanonAndScales()

        compose.awaitCount(hasTestTag("builder.remove"), 2, "items queued")
        app.tap("builder.remove")
        compose.awaitCount(hasTestTag("builder.remove"), 1, "items left after one is removed")

        app.tap("builder.cancel")
        app.tap("builder.discard.cancel")
        app.assertGone(hasTestTag("builder.discard.confirm"), "Cancel in the dialog closes it")
        app.control("builder.addItems")

        app.tap("builder.cancel")
        app.tap("builder.discard.confirm")
        app.control("practice.start")
    }

    @Test
    fun aLongPressDragReordersWithoutEditMode() {
        app.launch(seed = true)
        app.openEmptyBuilder()
        queueHanonAndScales()

        val hanon = builderRow("Hanon No. 1", "Exercise")
        val scales = builderRow("Major Scales", "Exercise")
        assertTrue("Hanon starts above Scales", top(hanon) < top(scales))

        val from = app.await(hanon).fetchSemanticsNode().boundsInRoot
        val to = app.await(scales).fetchSemanticsNode().boundsInRoot
        // Past the middle of the row below, which is where the drop counts as a move.
        val travel = to.center.y - from.center.y + to.height / 2
        app.await(hanon).performTouchInput {
            down(center)
            advanceEventTime(viewConfiguration.longPressTimeoutMillis + LIFT_PAUSE)
            repeat(DRAG_STEPS) { moveBy(Offset(0f, travel / DRAG_STEPS)) }
            up()
        }

        compose.waitUntil(App.TIMEOUT) { top(scales) < top(hanon) }
    }

    @Test
    fun aNestedRowTapOpensItsSettings() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tapRow("library.row", "Clair de Lune")
        app.tap("relatedExercises.add")
        app.tapRow("exercisePicker.row", "Hanon No. 1")
        app.tapRow("exercisePicker.row", "Major Scales")
        app.tap("exercisePicker.done")
        app.row("relatedExercises.row", "Major Scales")
        Espresso.pressBack()

        app.openEmptyBuilder()
        app.tap("builder.addItems")
        app.tapRow("addToSession.row", "Clair de Lune")
        app.tap("addToSession.done")

        app.await(builderRow("Major Scales", "Related"))
        app.await(builderRow("Hanon No. 1", "Related")).scrolledTo().performClick()
        app.control("entrySettings.trackReps")
    }

    private fun queueHanonAndScales() {
        app.tap("builder.addItems")
        app.tapRow("addToSession.row", "Hanon No. 1")
        app.tapRow("addToSession.row", "Major Scales")
        app.tap("addToSession.done")
    }

    private fun top(row: SemanticsMatcher) =
        compose.onAllNodes(row).fetchSemanticsNodes().firstOrNull()?.boundsInRoot?.top
            ?: Float.MAX_VALUE

    private companion object {
        const val LIFT_PAUSE = 200L
        const val DRAG_STEPS = 10
    }
}
