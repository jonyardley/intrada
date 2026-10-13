package com.intrada.android

import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * The Up next suggestion: Start plays today's plan in one tap (#57), "Change it first" opens it in
 * the builder, "Build my own instead" opens a blank one (#1617), and the suggestion comes back
 * afterwards rather than being lost for the app run (#1618).
 */
@RunWith(AndroidJUnit4::class)
class PracticeSuggestionTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun startPlaysThePlanInOneTap() {
        app.launch(seed = true)
        app.tapTab(AppTab.PRACTICE.tag)

        app.tap("practice.start")

        app.control("player.skip")
    }

    @Test
    fun changeItFirstOpensThePlanInTheBuilder() {
        app.launch(seed = true)
        app.tapTab(AppTab.PRACTICE.tag)

        app.tap("practice.changePlan")

        app.control("builder.row")
    }

    @Test
    fun buildOwnInsteadOpensTheBuilderInOneTap() {
        app.launch(seed = true)

        app.openEmptyBuilder()

        app.control("builder.addItems")
    }

    @Test
    fun theSuggestionCanBeRestoredWithoutRelaunching() {
        app.launch(seed = true)
        app.openEmptyBuilder()
        app.control("builder.addItems")

        app.tap("builder.cancel")
        app.tap("practice.showSuggestion")

        app.control("practice.buildOwn")
    }
}
