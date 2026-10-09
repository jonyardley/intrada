package com.intrada.android

import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import com.intrada.android.ui.ProgressScreen
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ProgressScreenTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun barsSpeakTheCoreWordsAndTheDialItsTopMark() {
        compose.setContent { ProgressScreen(ProgressFixtures.analytics, topMark = 5, onBuild = {}) }

        compose.onNodeWithContentDescription("This week: 82 minutes").assertExists()
        compose.onNodeWithContentDescription("Last week: 95 minutes").assertExists()
        compose.onNodeWithContentDescription("Overall mastery 3.4 of 5.0").assertExists()
        compose
            .onNodeWithContentDescription("Gymnopédie No. 1, first time marked, mastery 3")
            .assertExists()
    }

    @Test
    fun emptyProgressOffersToBuildASession() {
        var built = 0
        compose.setContent { ProgressScreen(null, topMark = 5, onBuild = { built++ }) }

        compose.onNodeWithTag("progress.empty.build").performClick()

        assertEquals(1, built)
    }
}
