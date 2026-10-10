package com.intrada.android

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.StateRestorationTester
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipeLeft
import androidx.compose.ui.test.swipeRight
import com.intrada.android.ui.PracticeModel
import com.intrada.android.ui.PracticeScreen
import com.intrada.android.ui.SessionDetailScreen
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.test.TestScope
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
    fun swipingToAnotherWeekShowsItsSessionsAndForgetsTheTappedDay() {
        compose.setContent { PracticeScreen(PracticeFixtures.twoWeeks, onStart = {}, onOpen = {}) }

        compose.onNodeWithTag("practice.day.2026-10-06").performClick()
        compose.onNodeWithTag("practice.weeks").performTouchInput { swipeRight() }
        compose
            .onNodeWithContentDescription("Thu 1 Oct, 30 min, 3 items", substring = true)
            .assertExists()

        compose.onNodeWithTag("practice.weeks").performTouchInput { swipeLeft() }
        compose
            .onNodeWithContentDescription("Today, 24 min, 3 items", substring = true)
            .assertExists()
        compose
            .onNodeWithContentDescription("Tue 6 Oct, 18 min", substring = true)
            .assertDoesNotExist()
    }

    @Test
    fun comingBackToPracticeKeepsTheChosenWeek() {
        val restoration = StateRestorationTester(compose)
        restoration.setContent {
            PracticeScreen(PracticeFixtures.twoWeeks, onStart = {}, onOpen = {})
        }

        compose.onNodeWithTag("practice.weeks").performTouchInput { swipeRight() }
        compose.onNodeWithTag("practice.day.2026-09-29").performClick()
        restoration.emulateSavedInstanceStateRestore()

        compose.onNodeWithTag("practice.day.2026-09-29").assertIsSelected()
        compose.onNodeWithContentDescription("No practice logged").assertExists()
    }

    @Test
    fun aNewWeekIsFollowedWhenTheLastWasShowing() {
        var model by mutableStateOf(PracticeFixtures.twoWeeks.copyWeeks(1))
        compose.setContent { PracticeScreen(model, onStart = {}, onOpen = {}) }
        compose
            .onNodeWithContentDescription("Thu 1 Oct, 30 min, 3 items", substring = true)
            .assertExists()

        model = PracticeFixtures.twoWeeks

        compose
            .onNodeWithContentDescription("Today, 24 min, 3 items", substring = true)
            .assertExists()
    }

    @Test
    fun theDetailNamesEachVariationWithItsMark() = runTest {
        val topMark = topMark()
        compose.setContent { SessionDetailScreen(PracticeFixtures.withVariations, topMark) }

        compose
            .onNodeWithContentDescription(
                "Major scales, Exercise · 8m, C major · 4m · 92 bpm · marked 4",
                substring = true,
            )
            .assertExists()
        compose
            .onNodeWithContentDescription(
                "Arpeggios, Exercise · 8m, E♭ major · 80 bpm, marked 3 out of $topMark"
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
        val day = store.practiceWeeks.value.flatMap { it.days }.single { it.date == "2026-10-07" }
        assertTrue(id in day.sessionIds)
    }

    private suspend fun TestScope.topMark(): Int =
        openedStore().viewModel.value?.limits?.scoreMax?.toInt() ?: error("no view")

    private fun PracticeModel.copyWeeks(count: Int) =
        PracticeModel(weeks.take(count), sessions, lastPractised, colour, greeting)
}
