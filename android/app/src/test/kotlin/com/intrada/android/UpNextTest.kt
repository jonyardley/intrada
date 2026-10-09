package com.intrada.android

import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.ui.PracticeRoute
import com.intrada.android.ui.PracticeScreen
import com.intrada.android.ui.UpNextActions
import com.intrada.android.ui.showsSuggestionRestore
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class UpNextTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun aPieceWithALinkedExerciseIsOfferedAndStartingItStartsTheSession() = runTest {
        val store = libraryStore()
        compose.setContent { PracticeRoute(store, onBuild = {}) }

        compose
            .onNodeWithContentDescription("Up next, Clair de Lune", substring = true)
            .assertExists()
        compose.onNodeWithTag("practice.start").performScrollTo().performClick()
        store.settle()

        assertNotNull(store.viewModel.value?.activeSession)
    }

    @Test
    fun changingThePlanOpensTheBuilderWithThePlanInIt() = runTest {
        val store = libraryStore()
        var opened = false
        compose.setContent { PracticeRoute(store, onBuild = { opened = true }) }

        compose.onNodeWithTag("practice.changePlan").performScrollTo().performClick()

        assertTrue(opened)
        assertTrue("Clair de Lune" in store.setlist().titles())
        assertTrue("Hanon No. 1" in store.setlist().titles())
    }

    @Test
    fun buildingYourOwnLeavesAWayBackToTheSuggestion() = runTest {
        val store = libraryStore()
        compose.setContent { PracticeRoute(store, onBuild = {}) }

        compose.onNodeWithTag("practice.buildOwn").performScrollTo().performClick()
        store.settle()
        compose.onNodeWithTag("practice.showSuggestion").assertDoesNotExist()
        store.send(Event.Session(SessionEvent.CancelBuilding))
        store.settle()
        compose.onNodeWithTag("practice.upNext").assertDoesNotExist()

        compose.onNodeWithTag("practice.showSuggestion").performScrollTo().performClick()
        compose.onNodeWithTag("practice.upNext").assertExists()
    }

    @Test
    fun aLiveSessionHidesTheWayBack() = runTest {
        val store = libraryStore()
        compose.setContent { PracticeRoute(store, onBuild = {}) }
        compose.onNodeWithTag("practice.buildOwn").performScrollTo().performClick()
        store.send(Event.Session(SessionEvent.CancelBuilding))
        store.startTwoItems()
        store.settle()

        compose.onNodeWithTag("practice.showSuggestion").assertDoesNotExist()
    }

    @Test
    fun practisingYourPrioritiesBuildsFromTheStarredPieces() = runTest {
        val library = listOf(Fixtures.item(priority = true)) + Fixtures.library.drop(1)
        val store = openedStore(InMemoryItemStore(library))
        var opened = false
        compose.setContent { PracticeRoute(store, onBuild = { opened = true }) }

        compose.onNodeWithTag("practice.priorities").performScrollTo().performClick()

        assertTrue(opened)
        assertEquals(listOf("Clair de Lune"), store.setlist().titles())
    }

    @Test
    fun theCardStartsAndChangesThroughItsOwnActions() {
        val taps = mutableListOf<String>()
        compose.setContent {
            PracticeScreen(
                PracticeFixtures.suggested,
                onStart = { taps += "last practised" },
                onOpen = {},
                upNext =
                    UpNextActions(onStart = { taps += "start" }, onChange = { taps += "change" }),
            )
        }

        compose.onNodeWithTag("practice.start").performScrollTo().performClick()
        compose.onNodeWithTag("practice.changePlan").performScrollTo().performClick()

        assertEquals(listOf("start", "change"), taps)
    }

    @Test
    fun theRestoreLinkShowsOnlyWhenDismissedWithAPlanAndNothingUnderway() {
        val plan = PracticeFixtures.plan
        val cases =
            listOf(
                Triple(plan, true, true) to true,
                Triple(plan, false, true) to false,
                Triple(plan, true, false) to false,
                Triple(null, true, true) to false,
            )
        cases.forEach { (input, expected) ->
            val (offered, idle, dismissed) = input
            assertEquals(
                "idle $idle, dismissed $dismissed, plan ${offered != null}",
                expected,
                showsSuggestionRestore(offered, idle, dismissed),
            )
        }
    }
}
