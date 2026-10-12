package com.intrada.android

import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * A create the core refuses keeps the form open with the core's sentence on it (#1595). The iPhone
 * drives this through a chord chart, which Android leaves out until #2025.
 */
@RunWith(AndroidJUnit4::class)
class CreateFaultTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun aRefusedCreateKeepsTheFormOpenAndNamesTheFault() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tap("library.add")
        app.control("itemForm.title").performTextInput("Kettle of fish")
        app.control("itemForm.composer").performTextInput("x".repeat(201))
        app.tap("itemForm.confirm")

        app.row("itemForm.error", "Composer must be between 1 and 200 characters")
        app.control("itemForm.confirm")

        app.tap("itemForm.cancel")
        app.control("library.add")
        app.assertNeverShows(
            hasTestTag("library.row") and App.mentions("Kettle of fish"),
            withinMillis = 3_000,
            "nothing was written, so the library is as it was",
        )
    }
}
