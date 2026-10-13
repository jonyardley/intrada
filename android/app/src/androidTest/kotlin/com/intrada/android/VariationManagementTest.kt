package com.intrada.android

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.getBoundsInRoot
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.App.Companion.scrolledTo
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * An exercise opens on its form, where a drag on a variation's handle moves its row (#1783),
 * against the seeded "Major Scales", whose one variation is "Hands separately". A piece's
 * Variations sheet renames a library variation by long press (#2567); Clair de Lune has none of its
 * own, so the renamed one is ticked to read back on the piece.
 */
@RunWith(AndroidJUnit4::class)
class VariationManagementTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun draggingAHandleMovesItsRow() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tapRow("library.row", "Major Scales")
        app.control("itemForm.variation.input").scrolledTo().performTextInput("Slow")
        app.tap("itemForm.variation.add")

        val slow = label("Slow")
        val saved = label("Hands separately")
        assertTrue("the typed row starts below the saved one", top(slow) > top(saved))

        app.await(hasTestTag("variationRow.reorder") and App.mentions("Reorder Slow"))
            .scrolledTo()
            .performTouchInput {
                down(center)
                advanceEventTime(viewConfiguration.longPressTimeoutMillis + 100)
                repeat(10) { moveBy(Offset(0f, -height * 0.15f)) }
                up()
            }

        assertTrue("the typed row dragged above the saved one", top(slow) < top(saved))
    }

    @Test
    fun renamingAVariationFromTheSheetShowsTheNewNameOnThePiece() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tapRow("library.row", "Clair de Lune")

        app.tap("variations.choose")
        app.row("variationsSheet.row", "Hands separately").scrolledTo().performTouchInput {
            longClick()
        }
        app.control("variationRename.name").performTextClearance()
        app.control("variationRename.name").performTextInput("Hands apart")
        app.tap("variationRename.confirm")
        app.tapRow("variationsSheet.row", "Hands apart")
        app.tap("sheet.done")

        app.row("variations.row", "Hands apart")
        app.assertGone(
            hasTestTag("variations.row") and App.mentions("Hands separately"),
            "the old name is gone",
        )
    }

    private fun label(text: String) = app.await(hasTestTag("variationRow.label") and hasText(text))

    private fun top(node: SemanticsNodeInteraction) = node.getBoundsInRoot().top
}
