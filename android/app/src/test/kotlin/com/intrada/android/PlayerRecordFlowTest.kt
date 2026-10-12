package com.intrada.android

import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextReplacement
import com.intrada.android.core.Store
import com.intrada.android.ui.PlayerModel
import com.intrada.android.ui.PlayerScreen
import com.intrada.android.ui.ScreenAlerts
import com.intrada.android.ui.SummaryModel
import com.intrada.android.ui.SummaryScreen
import com.intrada.android.ui.offersMoveOn
import com.intrada.android.ui.secondsLeft
import com.intrada.android.ui.sendAccepted
import com.intrada.shared.ActiveSessionView
import com.intrada.shared.BarRange
import com.intrada.shared.EntryStatus
import com.intrada.shared.Event
import com.intrada.shared.LastTimeView
import com.intrada.shared.ReflectionAnswers
import com.intrada.shared.SegmentClockView
import com.intrada.shared.SessionEvent
import java.time.Instant
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class PlayerRecordFlowTest {
    @get:Rule val compose = createComposeRule()

    private val sent = mutableListOf<SessionEvent>()
    private val held = Instant.parse("2026-10-07T09:03:25Z")

    private inline fun <reified T : SessionEvent> sentOf(): List<T> = sent.filterIsInstance<T>()

    private fun record(store: Store): (Event) -> Boolean = { event ->
        (event as? Event.Session)?.let { sent += it.value }
        store.sendAccepted(event)
    }

    /** The live player over the real core; [shape] adds what the fixtures cannot reach. */
    private fun showPlayer(store: Store, shape: (ActiveSessionView) -> ActiveSessionView = { it }) {
        compose.setContent {
            val view by store.viewModel.collectAsState()
            val active = view?.activeSession
            val limits = view?.limits
            if (active != null && limits != null) {
                PlayerScreen(
                    PlayerModel(shape(active), limits, held = held) {
                        ScreenAlerts(store.viewModel.value?.error, store.halted.value)
                    },
                    send = record(store),
                )
            }
        }
    }

    private suspend fun TestScope.scalesSession(): Store {
        val store = libraryStore()
        store.send(Event.Session(SessionEvent.StartBuilding))
        store.send(Event.Session(SessionEvent.AddToSetlist(store.scalesId())))
        store.send(Event.Session(SessionEvent.StartSession(PlayerFixtures.STARTED)))
        return store
    }

    @Test
    fun theSectionClockRoundsUpAndOffersOnlyOncePastItsEnd() {
        val end = Instant.parse("2026-10-07T09:05:00Z")
        val segment = SegmentClockView("A1", end.toString(), "Move on to Run", "Stay on A1")

        assertEquals(1, secondsLeft(end, end.minusMillis(400)))
        assertEquals(0, secondsLeft(end, end))
        assertFalse(offersMoveOn(segment, end.minusMillis(400)))
        assertTrue(offersMoveOn(segment, end))
        assertFalse(offersMoveOn(segment.copy(moveLabel = null), end.plusSeconds(60)))
    }

    @Test
    fun aSectionPastItsTimeOffersToMoveOnOrStay() = runTest {
        val store = openedStore()
        store.startTwoItems()
        val ended = SegmentClockView("A1", "2026-10-07T09:03:00Z", "Move on to Run", "Stay on A1")
        showPlayer(store) { it.copy(record = it.record.copy(segment = ended)) }

        compose.onAllNodesWithTag("player.troubleSpot").assertCountEquals(0)
        compose.onNodeWithTag("player.moveOn").performScrollTo().performClick()
        compose.onNodeWithTag("player.stay").performScrollTo().performClick()

        assertEquals(1, sentOf<SessionEvent.MoveToNextSegment>().size)
        assertEquals(1, sent.count { it == SessionEvent.StayOnSegment })
    }

    @Test
    fun theLastTimeOfferPlaysItThatWay() = runTest {
        val store = scalesSession()
        val entry = store.active().entries.first()
        val variation = store.active().currentVariations.last().id
        val offer = LastTimeView(entry.id, null, listOf(variation), "Last time: Swung", null)
        showPlayer(store) { it.copy(record = it.record.copy(lastTime = offer)) }

        compose.onNodeWithTag("player.lastTime").performScrollTo().performClick()

        val switched = sentOf<SessionEvent.SwitchPlay>().single()
        assertEquals(entry.id, switched.entryId)
        assertEquals(listOf(variation), switched.variationIds)
        assertEquals(listOf(variation), store.active().currentVariationIds)
    }

    @Test
    fun aVariationPickedMidItemIsWhatThePlayRecords() = runTest {
        val store = scalesSession()
        showPlayer(store)
        val swung = store.active().currentVariations.single { it.label == "Swung" }.id

        compose.onNodeWithTag("player.variation").performScrollTo().performClick()
        compose.onAllNodesWithTag("variationPicker.row")[1].performClick()

        assertEquals(listOf(swung), sentOf<SessionEvent.SwitchPlay>().single().variationIds)
        assertEquals(listOf(swung), store.active().currentVariationIds)
        compose.onAllNodesWithTag("variationPicker.row").assertCountEquals(0)
    }

    @Test
    fun aTroubleSpotTakesTheBarsTyped() = runTest {
        val store = openedStore()
        store.startTwoItems()
        showPlayer(store)

        compose.onNodeWithTag("player.troubleSpot").performScrollTo().performClick()
        compose.onNodeWithTag("troubleSpot.first").performTextReplacement("12")
        compose.onNodeWithTag("troubleSpot.last").performTextReplacement("16")
        compose.onNodeWithTag("troubleSpot.add").performClick()

        val spot = sentOf<SessionEvent.AddTroubleSpot>().single()
        assertEquals(store.active().entries.first().itemId, spot.itemId)
        assertEquals(BarRange(12u, 16u), spot.bars)
        compose.onAllNodesWithTag("troubleSpot.add").assertCountEquals(0)
    }

    @Test
    fun aRefusedSpotKeepsThePageOpenWithTheReason() = runTest {
        val store = openedStore()
        store.startTwoItems()
        showPlayer(store)

        compose.onNodeWithTag("player.troubleSpot").performScrollTo().performClick()
        compose.onNodeWithTag("troubleSpot.first").performTextReplacement("16")
        compose.onNodeWithTag("troubleSpot.last").performTextReplacement("12")
        compose.onNodeWithTag("troubleSpot.add").performClick()

        compose.onNodeWithTag("troubleSpot.add").assertExists()
        compose
            .onNodeWithText("Bars run from the lower bar to the higher", useUnmergedTree = true)
            .assertExists()
        assertEquals(null, store.viewModel.value?.error)
    }

    @Test
    fun aPointKeptFromTheNoteGoesInTheDraft() = runTest {
        val store = openedStore()
        store.startTwoItems()
        val reading = PlayerFixtures.silent
        store.send(Event.Session(SessionEvent.PrepareReflection("2026-10-07T09:04:00Z", reading)))
        store.send(
            Event.Session(
                SessionEvent.UpdateReflectionDraft(
                    ReflectionAnswers(
                        marks = emptyList(),
                        note = "left hand rushed in bar 12, got it at 84",
                        tempos = emptyList(),
                        gotInTheWay = emptyList(),
                        notePoints = emptyList(),
                        ways = emptyList(),
                    )
                )
            )
        )
        showPlayer(store)

        compose.onAllNodesWithTag("reflection.noteOffer")[0].performScrollTo().performClick()

        val draft = sentOf<SessionEvent.UpdateReflectionDraft>().last().answers
        assertEquals(1, draft.notePoints.size)
        assertEquals(
            1,
            store.active().record.finish?.noteOffers?.count { it.confirmed },
        )
    }

    @Test
    fun aChangedWayGoesInTheDraft() = runTest {
        val store = scalesSession()
        val active = store.active()
        val slow = active.currentVariations.first().id
        store.send(
            Event.Session(
                SessionEvent.SwitchPlay(
                    active.entries.first().id,
                    null,
                    null,
                    listOf(slow),
                    "2026-10-07T09:02:00Z",
                    PlayerFixtures.silent,
                )
            )
        )
        store.send(
            Event.Session(
                SessionEvent.PrepareReflection("2026-10-07T09:04:00Z", PlayerFixtures.silent)
            )
        )
        val swung = active.currentVariations.last().id
        showPlayer(store)

        compose.onAllNodesWithTag("reflection.changeWay")[1].performScrollTo().performClick()
        compose.onNodeWithTag("reflection.wayVariation.$swung").performScrollTo().performClick()

        val way = sentOf<SessionEvent.UpdateReflectionDraft>().last().answers.ways.single()
        assertEquals(listOf(slow, swung), way.variationIds)
    }

    @Test
    fun aNoteOnAnItemReachesTheSummary() = runTest {
        val store = openedStore()
        store.startTwoItems()
        store.send(
            Event.Session(
                SessionEvent.EndSessionEarly("2026-10-07T09:06:00Z", PlayerFixtures.silent)
            )
        )
        compose.setContent {
            val view by store.viewModel.collectAsState()
            val summary = view?.summary
            val limits = view?.limits
            if (summary != null && limits != null) {
                SummaryScreen(SummaryModel(summary, limits), send = record(store))
            }
        }
        val entry =
            store.viewModel.value?.summary?.entries?.first { it.status != EntryStatus.NOTATTEMPTED }
                ?: error("nothing practised")

        compose.onAllNodesWithTag("summary.addNote")[0].performScrollTo().performClick()
        compose.onNodeWithTag("summary.entryNote").performTextReplacement("Left hand rushed")

        val note = sentOf<SessionEvent.UpdateEntryNotes>().last()
        assertEquals(entry.id, note.entryId)
        assertEquals(
            "Left hand rushed",
            store.viewModel.value?.summary?.entries?.first { it.id == entry.id }?.notes,
        )
    }
}
