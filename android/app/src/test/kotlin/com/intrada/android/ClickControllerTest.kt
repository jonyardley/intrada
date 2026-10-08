package com.intrada.android

import android.os.Looper
import com.intrada.android.core.ClickOutput
import com.intrada.android.ui.ClickController
import com.intrada.shared.ClickPreset
import com.intrada.shared.Event
import com.intrada.shared.Metre
import com.intrada.shared.SessionEvent
import java.time.Duration
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config

private class CountingClick : ClickOutput {
    val started = mutableListOf<Int>()
    val beats = mutableListOf<Int>()
    val soundings = mutableListOf<Int>()
    var stops = 0

    override var onPulseDied: (() -> Unit)? = null

    override fun start(bpm: Int, beats: Int, sounding: Int): Boolean {
        started += bpm
        this.beats += beats
        soundings += sounding
        return true
    }

    override fun stop() {
        stops++
    }
}

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ClickControllerTest {
    private val output = CountingClick()
    private val click = ClickController { output }

    @Test
    fun theNextItemSilencesTheClickAndTakesItsOwnTempo() = runTest {
        val store = openedStore()
        store.startTwoItems()
        val limits = checkNotNull(store.viewModel.value?.limits)
        click.follow(store.active(), limits)
        click.toggle()
        click.step(click.step)
        val now = "2026-10-07T09:05:00Z"
        store.send(Event.Session(SessionEvent.PrepareReflection(now, click.reading)))
        store.send(Event.Session(SessionEvent.NextItem(now, now, click.reading)))

        click.follow(store.active(), limits)

        assertFalse(click.isRunning)
        assertEquals(store.active().clickSeedBpm.toInt(), click.bpm)
        assertEquals(null, click.reading.click)
    }

    @Test
    fun theSameItemAgainLeavesTheClickAlone() = runTest {
        val store = openedStore()
        store.startTwoItems()
        val limits = checkNotNull(store.viewModel.value?.limits)
        click.follow(store.active(), limits)
        click.toggle()
        click.step(click.step)
        val stepped = click.bpm

        click.follow(store.active(), limits)

        assertTrue(click.isRunning)
        assertEquals(stepped, click.bpm)
    }

    @Test
    fun tenMinutesAwayStopsTheClick() = runTest {
        startOnFirstItem()

        click.enteredBackground()
        shadowOf(Looper.getMainLooper()).idleFor(Duration.ofMinutes(10))

        assertFalse(click.isRunning)
    }

    @Test
    fun comingBackInTimeKeepsTheClick() = runTest {
        startOnFirstItem()

        click.enteredBackground()
        click.enteredForeground()
        shadowOf(Looper.getMainLooper()).idleFor(Duration.ofMinutes(10))

        assertTrue(click.isRunning)
    }

    @Test
    fun aNewBarSoundsEveryBeatAndIsReportedAsChosen() = runTest {
        startOnFirstItem()

        click.bar.choose(sixEight)

        assertEquals(0b111111, click.bar.sounding.toInt())
        assertEquals(sixEight, click.reading.click?.metre)
        assertEquals(6, output.beats.last())
    }

    @Test
    fun groupStartsInSixEightSoundsBeatsOneAndFour() = runTest {
        startOnFirstItem()
        click.bar.choose(sixEight)

        click.bar.apply(ClickPreset.GROUPSTARTS)

        assertEquals(0b001001, click.bar.sounding.toInt())
        assertEquals(ClickPreset.GROUPSTARTS, click.bar.matchingPreset)
        assertEquals(0b001001, output.soundings.last())
    }

    @Test
    fun theLastSoundingBeatCannotBeSilenced() = runTest {
        startOnFirstItem()
        click.bar.apply(ClickPreset.DOWNBEAT)

        click.bar.toggleBeat(0)

        assertEquals(1, click.bar.sounding.toInt())
    }

    @Test
    fun aSilencedBeatMatchesNoPattern() = runTest {
        startOnFirstItem()
        click.bar.choose(Metre(4u, 4u))

        click.bar.toggleBeat(2)

        assertEquals(0b1011, click.bar.sounding.toInt())
        assertEquals(null, click.bar.matchingPreset)
    }

    @Test
    fun aDragPastTheBandStopsAtItsEdge() = runTest {
        startOnFirstItem()

        click.dragTo(10_000)

        assertEquals(click.band.last, click.bpm)
        assertEquals(click.band.last, output.started.last())
    }

    private val sixEight = Metre(6u, 8u, listOf(3u, 3u))

    private suspend fun kotlinx.coroutines.test.TestScope.startOnFirstItem() {
        val store = openedStore()
        store.startTwoItems()
        click.follow(store.active(), checkNotNull(store.viewModel.value?.limits))
        click.toggle()
        assertTrue(click.isRunning)
    }
}
