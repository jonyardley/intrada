package com.intrada.android

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Modifier
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.Settings
import com.intrada.android.ui.ClickRow
import com.intrada.android.ui.ClickRowState
import com.intrada.android.ui.ClickStatus
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaSpacing
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
    fun clickRow() {
        fun row(status: ClickStatus, bpm: Int = 66, seeded: Boolean = true, unit: UByte = 4u) =
            ClickRowState(bpm, unit, status, seeded, "Andante · ♩ = 66", "Andante, 66 bpm")
        captureRoboImage("src/test/snapshots/player-click.png") {
            Column(
                Modifier.background(IntradaColor.paperTop).padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
            ) {
                ClickRow(row(ClickStatus.STOPPED), {}, {}, step = 2)
                ClickRow(row(ClickStatus.RUNNING, bpm = 72, seeded = false), {}, {}, step = 2)
                ClickRow(row(ClickStatus.RUNNING, bpm = 168, unit = 8u), {}, {}, step = 2)
                ClickRow(row(ClickStatus.UNAVAILABLE), {}, {}, step = 2)
            }
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
