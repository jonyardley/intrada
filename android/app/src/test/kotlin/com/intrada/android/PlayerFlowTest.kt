package com.intrada.android

import android.content.Context
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.Settings
import com.intrada.android.core.resumeRecoverableSession
import com.intrada.android.ui.PlayerHost
import com.intrada.android.ui.PlayerModel
import com.intrada.android.ui.PlayerScreen
import com.intrada.android.ui.PracticeRoute
import com.intrada.android.ui.ScreenAlerts
import com.intrada.android.ui.SessionClock
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class PlayerFlowTest {
    @get:Rule val compose = createComposeRule()

    private val prefs =
        RuntimeEnvironment.getApplication()
            .getSharedPreferences(Settings.PREFERENCES, Context.MODE_PRIVATE)
    private val practice =
        RuntimeEnvironment.getApplication()
            .getSharedPreferences(Settings.PRACTICE_PREFERENCES, Context.MODE_PRIVATE)

    @Test
    fun aSessionPlayedThroughIsThereAfterARestart() = runTest {
        val items = InMemoryItemStore(Fixtures.library)
        val store = openedStore(items)
        store.startTwoItems()
        compose.setContent { PlayerHost(store) }

        compose.onNodeWithTag("player.title").assertIsDisplayed()
        compose.onNodeWithText("Gymnopédie No. 1").assertIsDisplayed()
        compose.onNodeWithTag("player.gotIt").performScrollTo().performClick()
        assertEquals(1, store.active().currentRepCount?.toInt())
        compose.onNodeWithTag("player.advance").performClick()
        compose.onNodeWithTag("reflection.mark.4").performScrollTo().performClick()
        compose.onNodeWithTag("reflection.save").performScrollTo().performClick()

        compose.onNodeWithText("Hanon No. 1").assertIsDisplayed()
        compose.onNodeWithTag("player.advance").performClick()
        compose.onNodeWithTag("reflection.skip").performScrollTo().performClick()

        compose.onNodeWithTag("summary.save").performScrollTo().performClick()
        store.settle()
        assertNull(store.viewModel.value?.activeSession)
        assertNull(store.viewModel.value?.summary)

        val restarted = openedStore(items)
        assertTrue(
            BuilderFixtures.SATIE in restarted.viewModel.value?.recentlyPractisedIds.orEmpty()
        )
    }

    @Test
    fun anErrorFromBeforeTheSheetDoesNotShowAsItsRefusal() = runTest {
        val store = openedStore()
        store.startTwoItems()
        store.send(Event.Session(SessionEvent.StartSession(PlayerFixtures.STARTED)))
        val stale = checkNotNull(store.viewModel.value?.error)
        compose.setContent { PlayerHost(store) }

        compose.onNodeWithTag("player.advance").performClick()

        compose.onNodeWithTag("reflection.save").assertExists()
        compose.onNodeWithContentDescription("Error: $stale").assertDoesNotExist()
    }

    @Test
    fun aRefusedSaveIsShownOnTheSheetAndAnnounced() = runTest {
        val store = openedStore()
        store.startTwoItems()
        store.send(
            Event.Session(
                SessionEvent.PrepareReflection("2026-10-07T09:04:10Z", PlayerFixtures.silent)
            )
        )
        val limits = checkNotNull(store.viewModel.value?.limits)
        val model = PlayerModel(store.active(), limits, alertsNow = { ScreenAlerts("Nope") })
        compose.setContent { PlayerScreen(model, send = { false }) }

        compose.onNodeWithTag("reflection.save").performScrollTo().performClick()

        compose
            .onNodeWithContentDescription("Error: Nope")
            .assert(
                SemanticsMatcher.expectValue(SemanticsProperties.LiveRegion, LiveRegionMode.Polite)
            )
    }

    @Test
    fun aMarkIsKeptAsADraftSoAResumeReopensTheSheetWithIt() = runTest {
        val store = openedStore(settings = Settings(prefs, practice))
        store.startTwoItems()
        compose.setContent { PlayerHost(store) }

        compose.onNodeWithTag("player.advance").performClick()
        compose.onNodeWithTag("reflection.mark.3").performScrollTo().performClick()

        val draft = store.active().reflection?.answers?.marks
        assertEquals(listOf(3), draft?.map { it.score.toInt() })
        val saved = Settings(prefs, practice).pendingSessionInProgress()
        assertEquals(listOf(3), saved?.reflection?.answers?.marks?.map { it.score.toInt() })
    }

    // The app is killed with the sheet up: a fresh store reads the slot, offers the practice and
    // resumes it at the same item with the same start, through the real core both ways (#846,
    // #1345).
    @Test
    fun aPracticeKilledAtTheSheetResumesAtTheSameItemWithItsTime() = runTest {
        val items = InMemoryItemStore(Fixtures.library)
        val first = openedStore(items, Settings(prefs, practice))
        first.startTwoItems()
        val now = "2026-10-07T09:05:00Z"
        first.send(Event.Session(SessionEvent.PrepareReflection(now, PlayerFixtures.silent)))
        first.send(Event.Session(SessionEvent.NextItem(now, now, PlayerFixtures.silent)))
        first.send(
            Event.Session(SessionEvent.RepGotIt("2026-10-07T09:06:00Z", PlayerFixtures.silent))
        )
        // The sheet stamps the open play's seconds, so the slot holds three minutes of Hanon
        // (#2061).
        first.send(
            Event.Session(
                SessionEvent.PrepareReflection("2026-10-07T09:08:00Z", PlayerFixtures.silent)
            )
        )
        val before = first.active()

        val next = openedStore(items, Settings(prefs, practice))
        next.loadRecoverableSession()
        assertNotNull(next.recoverableSession.value)
        val resumedAt = "2026-10-07T09:20:00Z"
        next.resumeRecoverableSession(resumedAt)

        val after = next.active()
        assertEquals(1uL, after.currentPosition)
        assertEquals(before.currentItemTitle, after.currentItemTitle)
        assertEquals(before.startedAt, after.startedAt)
        assertEquals(1, after.currentRepCount?.toInt())
        val itemStart = checkNotNull(SessionClock.parse(after.currentItemStartedAt))
        val resumed = checkNotNull(SessionClock.parse(resumedAt))
        assertEquals(180L, SessionClock.secondsBetween(itemStart, resumed))
        assertEquals(180uL, after.reflection?.elapsedSecs)
        assertNull(next.recoverableSession.value)
    }

    @Test
    fun thePracticeTabOffersTheInterruptedSessionAndResumesIt() = runTest {
        val first = openedStore(settings = Settings(prefs, practice))
        first.startTwoItems()
        val next = openedStore(settings = Settings(prefs, practice))
        next.loadRecoverableSession()
        compose.setContent { PracticeRoute(next, onBuild = {}) }

        compose.onNodeWithText("Pick up where you left off?").assertIsDisplayed()
        compose.onNodeWithTag("practice.resume").performClick()

        assertEquals("Gymnopédie No. 1", next.active().currentItemTitle)
        compose.onNodeWithText("Pick up where you left off?").assertDoesNotExist()
    }

    @Test
    fun discardingTheOfferEmptiesTheSlot() = runTest {
        val first = openedStore(settings = Settings(prefs, practice))
        first.startTwoItems()
        val next = openedStore(settings = Settings(prefs, practice))
        next.loadRecoverableSession()
        compose.setContent { PracticeRoute(next, onBuild = {}) }

        compose.onNodeWithTag("practice.discardResume").performClick()

        assertNull(Settings(prefs, practice).sessionInProgress.read())
        compose.onNodeWithText("Pick up where you left off?").assertDoesNotExist()
    }
}
