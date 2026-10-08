package com.intrada.android

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.junit4.StateRestorationTester
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.intrada.android.core.ClickOutput
import com.intrada.android.ui.ClickController
import com.intrada.android.ui.PlayerModel
import com.intrada.android.ui.PlayerScreen
import com.intrada.shared.Event
import com.intrada.shared.Metre
import com.intrada.shared.SessionEvent
import com.intrada.shared.TempoReading
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

private class FakeClick(private val starts: Boolean = true) : ClickOutput {
    val started = mutableListOf<Int>()
    var stopped = 0

    override var onPulseDied: (() -> Unit)? = null

    override fun start(bpm: Int, beats: Int, sounding: Int): Boolean {
        if (starts) started += bpm
        return starts
    }

    override fun stop() {
        stopped++
    }
}

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ClickFlowTest {
    @get:Rule val compose = createComposeRule()

    private val sent = mutableListOf<Event>()

    private val readings: List<TempoReading>
        get() = sent.mapNotNull {
            when (val event = (it as? Event.Session)?.value) {
                is SessionEvent.TempoChanged -> event.reading
                is SessionEvent.PrepareReflection -> event.reading
                else -> null
            }
        }

    private suspend fun TestScope.showPlayer(fake: FakeClick) {
        val store = openedStore()
        store.startTwoItems()
        val limits = checkNotNull(store.viewModel.value?.limits)
        val model = PlayerModel(store.active(), limits, click = ClickController { fake })
        compose.setContent {
            PlayerScreen(
                model,
                send = {
                    sent += it
                    true
                },
            )
        }
    }

    @Test
    fun theClickStartsAtTheItemsTempoAndTheSteppedTempoReachesTheReflection() = runTest {
        val fake = FakeClick()
        showPlayer(fake)

        compose.onNodeWithTag("click.toggle").performScrollTo().performClick()
        compose.onNodeWithContentDescription("Faster").performClick()
        compose.onNodeWithTag("player.advance").performClick()

        val step = fake.started[1] - fake.started[0]
        assertEquals(70, fake.started.first())
        assertTrue(step > 0)
        val last = readings.last()
        assertEquals(70 + step, last.bpm.toInt())
        assertTrue(last.clickSounding)
        assertNotNull(last.click)
    }

    // Turning the phone rebuilds the screen; the click must keep sounding at the stepped tempo.
    @Test
    fun aRebuiltScreenKeepsTheClickAndItsTempo() = runTest {
        val fake = FakeClick()
        val store = openedStore()
        store.startTwoItems()
        val limits = checkNotNull(store.viewModel.value?.limits)
        val model = PlayerModel(store.active(), limits, click = ClickController { fake })
        val restoration = StateRestorationTester(compose)
        restoration.setContent {
            PlayerScreen(
                model,
                send = {
                    sent += it
                    true
                },
            )
        }
        compose.onNodeWithTag("click.toggle").performScrollTo().performClick()
        compose.onNodeWithContentDescription("Faster").performClick()

        restoration.emulateSavedInstanceStateRestore()
        compose.onNodeWithTag("player.advance").performClick()

        compose.onNodeWithContentDescription("Stop the metronome").assertExists()
        assertEquals(fake.started.last(), readings.last().bpm.toInt())
        assertTrue(readings.last().clickSounding)
    }

    @Test
    fun stoppingTheClickIsSentAsSilent() = runTest {
        val fake = FakeClick()
        showPlayer(fake)

        compose.onNodeWithTag("click.toggle").performScrollTo().performClick()
        compose.onNodeWithTag("click.toggle").performClick()

        assertEquals(1, fake.stopped)
        assertFalse(readings.last().clickSounding)
    }

    @Test
    fun anUntouchedClickSendsNoBarOfItsOwn() = runTest {
        showPlayer(FakeClick())

        compose.onNodeWithTag("player.advance").performClick()

        val reading = readings.single()
        assertEquals(70, reading.bpm.toInt())
        assertFalse(reading.clickSounding)
        assertEquals(null, reading.click)
    }

    @Test
    fun audioThatWillNotStartSaysSo() = runTest {
        showPlayer(FakeClick(starts = false))

        compose.onNodeWithTag("click.toggle").performScrollTo().performClick()

        compose
            .onNodeWithTag("click.toggle")
            .assert(
                SemanticsMatcher.expectValue(SemanticsProperties.StateDescription, "unavailable")
            )
        assertFalse(readings.last().clickSounding)
    }

    @Test
    fun aPulseAnotherAppTookShowsTheClickStopped() = runTest {
        val fake = FakeClick()
        showPlayer(fake)
        compose.onNodeWithTag("click.toggle").performScrollTo().performClick()

        fake.onPulseDied?.invoke()

        compose.onNodeWithContentDescription("Start the metronome").assertExists()
    }

    @Test
    fun aBarAndPatternChosenInTheSheetReachTheReflection() = runTest {
        showPlayer(FakeClick())

        compose.onNodeWithTag("click.toggle").performScrollTo().performClick()
        compose.onNodeWithTag("click.bar").performScrollTo().performClick()
        compose.onNodeWithTag("clickSheet.metre.6-8").performClick()
        compose.onNodeWithTag("clickSheet.pattern.groupstarts").performClick()
        compose.onNodeWithTag("clickSheet.beat.4").performScrollTo().performClick()
        compose.onNodeWithContentDescription("Done").performClick()
        compose.onNodeWithTag("player.advance").performClick()

        val chosen = readings.last().click
        assertEquals(Metre(6u, 8u, listOf(3u, 3u)), chosen?.metre)
        assertEquals(0b000001, chosen?.sounding?.toInt())
    }
}
