package com.intrada.android

import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Choosing every major key in the keys sheet lays the twelve rings out on the piece screen (#2568),
 * against the seeded "Clair de Lune".
 */
@RunWith(AndroidJUnit4::class)
class ItemKeysTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun allMajorKeysShowAsRings() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tapRow("library.row", "Clair de Lune")

        app.tap("keys.choose")
        app.tap("keysSheet.allMajor")
        app.tap("sheet.done")

        for (key in listOf("C major", "G major", "F major")) {
            app.await(hasContentDescription("$key,", substring = true))
        }
    }
}
