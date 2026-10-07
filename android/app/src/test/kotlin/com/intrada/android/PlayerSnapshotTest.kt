package com.intrada.android

import android.content.Context
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.Settings
import com.intrada.android.ui.PlayerModel
import com.intrada.android.ui.PlayerScreen
import com.intrada.android.ui.RecoveryCard
import com.intrada.android.ui.SummaryModel
import com.intrada.android.ui.SummaryScreen
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import java.time.Instant
import java.time.ZoneOffset
import kotlinx.coroutines.test.runTest
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = RobolectricDeviceQualifiers.Pixel7)
class PlayerSnapshotTest {
    private val held = Instant.parse("2026-10-07T09:03:25Z")

    @Test
    fun player() = runTest {
        val store = openedStore()
        store.startTwoItems()
        store.send(
            Event.Session(SessionEvent.RepGotIt("2026-10-07T09:01:00Z", PlayerFixtures.silent))
        )
        val view = store.viewModel.value ?: error("no view")
        captureRoboImage("src/test/snapshots/player.png") {
            PlayerScreen(PlayerModel(store.active(), view.limits, held = held), send = { true })
        }
    }

    @Test
    fun reflection() = runTest {
        val store = openedStore()
        store.startTwoItems()
        store.send(
            Event.Session(
                SessionEvent.PrepareReflection("2026-10-07T09:04:10Z", PlayerFixtures.silent)
            )
        )
        val view = store.viewModel.value ?: error("no view")
        captureRoboImage("src/test/snapshots/player-reflection.png") {
            PlayerScreen(PlayerModel(store.active(), view.limits, held = held), send = { true })
        }
    }

    @Test
    fun summary() = runTest {
        val store = openedStore()
        store.startTwoItems()
        val at = "2026-10-07T09:06:00Z"
        store.send(Event.Session(SessionEvent.EndSessionEarly(at, PlayerFixtures.silent)))
        val view = store.viewModel.value ?: error("no view")
        val summary = view.summary ?: error("no summary")
        captureRoboImage("src/test/snapshots/player-summary.png") {
            SummaryScreen(SummaryModel(summary, view.limits), send = { true })
        }
    }

    @Test
    fun recoveryCard() = runTest {
        val app = RuntimeEnvironment.getApplication()
        val settings =
            Settings(
                app.getSharedPreferences(Settings.PREFERENCES, Context.MODE_PRIVATE),
                app.getSharedPreferences(Settings.PRACTICE_PREFERENCES, Context.MODE_PRIVATE),
            )
        val store = openedStore(settings = settings)
        store.startTwoItems()
        val session = checkNotNull(store.pendingSessionInProgress())
        captureRoboImage("src/test/snapshots/player-recovery.png") {
            RecoveryCard(
                session,
                onResume = {},
                onDiscard = {},
                today = held,
                zone = ZoneOffset.UTC,
            )
        }
    }
}
