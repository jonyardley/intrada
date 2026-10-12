package com.intrada.android

import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * A section added on the piece screen survives a relaunch, and one removed in Reorder is gone once
 * Done saves (#2247). Not seeded: seed mode keeps nothing, and the relaunch is the point.
 */
@RunWith(AndroidJUnit4::class)
class SectionsTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun anAddedSectionSurvivesARelaunch() {
        app.launchFresh()
        app.tap("firstRun.skipWelcome")
        app.tapTab(AppTab.LIBRARY.tag)
        app.tap("library.add")
        app.control("itemForm.title").performTextInput("Sections")
        app.tap("itemForm.confirm")

        app.tapRow("library.row", "Sections")
        app.tap("sections.add")
        app.control("sectionSheet.name").performTextInput("A1")
        app.control("sectionSheet.bars").performTextInput("1 to 16")
        app.tap("sectionSheet.save")
        app.row("sections.row", "A1, bars 1 to 16")

        app.relaunch()
        app.tapTab(AppTab.LIBRARY.tag)
        app.tapRow("library.row", "Sections")
        app.row("sections.row", "A1, bars 1 to 16")

        app.tap("sections.header")
        app.await(hasContentDescription("Remove A1")).performClick()
        app.tap("sections.header")
        app.control("sections.add")
        app.assertGone(hasTestTag("sections.row"), "the removal saved on Done")
    }
}
