package com.intrada.android

import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * A saved session's card in Practice opens the record of what was played (#1371). iPhone:
 * SessionHistoryUITests.
 */
@RunWith(AndroidJUnit4::class)
class SessionHistoryTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun tappingASessionCardOpensTheRecordOfWhatWasPlayed() {
        app.launch(seed = true)
        app.tapTab(AppTab.PRACTICE.tag)
        app.tap("practice.buildOwn")
        app.startWith("Clair de Lune")
        app.endSessionEarly()
        app.tap("summary.save")

        app.tapRow("practice.sessionCard", "1 piece")

        // The detail screen's own content: a card that merely vanished would also pass.
        app.control("sessionDetail.played")
        app.await(App.mentions("Clair de Lune"))
    }
}
