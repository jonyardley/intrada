package com.intrada.android

import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.test.espresso.Espresso
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/** System back from a piece returns to the Library, as the iPhone's edge swipe does (#1822). */
@RunWith(AndroidJUnit4::class)
class NativeBackNavigationTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun backReturnsFromAPieceToTheLibrary() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tapRow("library.row", "Clair de Lune")
        app.text("Claude Debussy")

        Espresso.pressBack()

        app.text("3 pieces")
    }
}
