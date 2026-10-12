package com.intrada.android

import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.App.Companion.scrolledTo
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * A BPM the core cannot read is refused on the form, in the core's own sentence, rather than
 * dropped on save (#2224). The iPhone's two refusals inside a piece's exercise sheet wait on #2569.
 */
@RunWith(AndroidJUnit4::class)
class BpmRefusalTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun anUnreadableBpmKeepsTheAddFormOpenWithTheCoresSentence() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tap("library.add")
        app.control("itemForm.bpm").scrolledTo().performTextInput("12a")
        app.control("itemForm.title").scrolledTo().performTextInput("Kettle of fish")
        app.tap("itemForm.confirm")

        app.row("itemForm.error", "BPM must be a whole number between 1 and 400")
        app.control("itemForm.confirm")
        app.await(hasTestTag("itemForm.bpm") and hasText("12a"))
    }
}
