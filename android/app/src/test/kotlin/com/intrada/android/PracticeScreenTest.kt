package com.intrada.android

import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.intrada.android.ui.PracticeScreen
import com.intrada.android.ui.SessionDetailScreen
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class PracticeScreenTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun theWeekOpensOnItsOpeningDayAndATappedDaySwapsTheCards() {
        val opened = mutableListOf<String>()
        compose.setContent {
            PracticeScreen(PracticeFixtures.filled, onStart = {}, onOpen = { opened += it })
        }

        compose
            .onNodeWithContentDescription("Today, 24 min, 3 items", substring = true)
            .assertExists()
        compose.onNodeWithTag("practice.day.2026-10-06").performClick()
        compose
            .onNodeWithContentDescription("Tue 6 Oct, 18 min, 3 items", substring = true)
            .performScrollTo()
            .performClick()

        assertEquals(listOf(PracticeFixtures.withVariations.id), opened)
    }

    @Test
    fun aRestDaySaysSo() {
        compose.setContent { PracticeScreen(PracticeFixtures.filled, onStart = {}, onOpen = {}) }

        compose.onNodeWithTag("practice.day.2026-10-07").performClick()

        compose.onNodeWithContentDescription("No practice logged").assertExists()
    }

    @Test
    fun theDetailNamesEachVariationWithItsMark() {
        compose.setContent { SessionDetailScreen(PracticeFixtures.withVariations, topMark = 10) }

        compose
            .onNodeWithContentDescription(
                "Major scales, Exercise · 8m, C major · 4m · 92 bpm · marked 4",
                substring = true,
            )
            .assertExists()
        compose
            .onNodeWithContentDescription(
                "Arpeggios, Exercise · 8m, E♭ major · 80 bpm, marked 3 out of 10"
            )
            .assertExists()
    }

    @Test
    fun aSavedSessionReachesTheHistoryAndItsWeek() = runTest {
        val store = openedStore()
        assertTrue(store.practiceWeeks.value.isNotEmpty())

        store.startTwoItems()
        store.send(
            Event.Session(
                SessionEvent.EndSessionEarly("2026-10-07T09:05:00Z", PlayerFixtures.silent)
            )
        )
        store.send(Event.Session(SessionEvent.SaveSession("2026-10-07T09:06:00Z")))
        store.settle()

        val id = store.sessionHistory.value.single().id
        assertTrue(store.practiceWeeks.value.any { week -> week.days.any { id in it.sessionIds } })
    }
}
