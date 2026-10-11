package com.intrada.android

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Modifier
import com.github.takahirom.roborazzi.RobolectricDeviceQualifiers
import com.github.takahirom.roborazzi.captureRoboImage
import com.intrada.android.core.ClickOutput
import com.intrada.android.core.Settings
import com.intrada.android.core.Store
import com.intrada.android.ui.ClickActions
import com.intrada.android.ui.ClickBarLine
import com.intrada.android.ui.ClickController
import com.intrada.android.ui.ClickRow
import com.intrada.android.ui.ClickRowState
import com.intrada.android.ui.ClickSheetPage
import com.intrada.android.ui.ClickStatus
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaSpacing
import com.intrada.android.ui.PlayWayChoices
import com.intrada.android.ui.PlayerModel
import com.intrada.android.ui.PlayerScreen
import com.intrada.android.ui.RecoveryCard
import com.intrada.android.ui.ReflectionActions
import com.intrada.android.ui.ReflectionModel
import com.intrada.android.ui.ReflectionSheet
import com.intrada.android.ui.ReflectionState
import com.intrada.android.ui.SheetPage
import com.intrada.android.ui.SummaryModel
import com.intrada.android.ui.SummaryScreen
import com.intrada.android.ui.TroubleSpotContent
import com.intrada.android.ui.VariationChoice
import com.intrada.android.ui.VariationPickerContent
import com.intrada.shared.ClickPreset
import com.intrada.shared.Event
import com.intrada.shared.LastTimeView
import com.intrada.shared.Metre
import com.intrada.shared.ReflectionAnswers
import com.intrada.shared.SegmentClockView
import com.intrada.shared.SessionEvent
import java.time.Instant
import java.time.ZoneOffset
import kotlinx.coroutines.test.TestScope
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
        val active = store.active()
        // Seeded before composition, or the row reads whatever frame the looper reached (#2503).
        val click = ClickController {
            error("a snapshot never sounds")
        }
            .apply { follow(active, view.limits) }
        captureRoboImage("src/test/snapshots/player.png") {
            PlayerScreen(
                PlayerModel(active, view.limits, held = held, click = click),
                send = { true },
            )
        }
    }

    @Test
    fun clickRow() {
        fun row(status: ClickStatus, bpm: Int = 66, seeded: Boolean = true, unit: UByte = 4u) =
            ClickRowState(
                bpm,
                unit,
                status,
                seeded,
                "Andante · ♩ = 66",
                "Andante, 66 beats per minute",
            )
        captureRoboImage("src/test/snapshots/player-click.png") {
            Column(
                Modifier.background(IntradaColor.paperTop).padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
            ) {
                ClickRow(
                    row(ClickStatus.STOPPED),
                    ClickActions({}, {}, step = 2, band = 40..208, onDrag = {}),
                )
                ClickRow(
                    row(ClickStatus.RUNNING, bpm = 72, seeded = false),
                    ClickActions({}, {}, step = 2, band = 40..208, onDrag = {}),
                )
                ClickRow(
                    row(ClickStatus.RUNNING, bpm = 168, unit = 8u),
                    ClickActions({}, {}, step = 2, band = 40..208, onDrag = {}),
                )
                ClickRow(
                    row(ClickStatus.UNAVAILABLE),
                    ClickActions({}, {}, step = 2, band = 40..208, onDrag = {}),
                )
            }
        }
    }

    @Test
    fun clickBarLine() {
        captureRoboImage("src/test/snapshots/player-click-bar.png") {
            Column(
                Modifier.background(IntradaColor.paperTop).padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
            ) {
                ClickBarLine(Metre(4u, 4u), 0b1111u, 1, {})
                ClickBarLine(Metre(4u, 4u), 0b1000u, 3, {})
                ClickBarLine(Metre(7u, 8u, listOf(3u, 2u, 2u)), 0b0101001u, 0, {})
                ClickBarLine(Metre(12u, 8u, List(6) { 2u }), 0b0101_0101_0101u, 0, {})
            }
        }
    }

    @Test
    fun clickSheet() = runTest {
        val store = openedStore()
        store.startTwoItems()
        val limits = store.viewModel.value?.limits ?: error("no limits")
        val click = ClickController { SilentClick }
        click.follow(store.active(), limits)
        click.bar.choose(Metre(7u, 8u, listOf(3u, 2u, 2u)))
        click.bar.apply(ClickPreset.GROUPSTARTS)
        captureRoboImage("src/test/snapshots/player-click-sheet.png") {
            ClickSheetPage(click, limits, onClose = {})
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
    fun playerRecord() = runTest {
        val store = libraryStore()
        store.send(Event.Session(SessionEvent.StartBuilding))
        store.send(Event.Session(SessionEvent.AddToSetlist(store.scalesId())))
        store.send(Event.Session(SessionEvent.StartSession(PlayerFixtures.STARTED)))
        val view = store.viewModel.value ?: error("no view")
        val active = store.active()
        val record =
            active.record.copy(
                segment =
                    SegmentClockView("A1", "2026-10-07T09:03:00Z", "Move on to Run", "Stay on A1"),
                lastTime =
                    LastTimeView(
                        active.entries.first().id,
                        null,
                        listOf(active.currentVariations.last().id),
                        "Last time: Swung",
                        null,
                    ),
            )
        val click = ClickController {
            error("a snapshot never sounds")
        }
            .apply { follow(active, view.limits) }
        captureRoboImage("src/test/snapshots/player-record.png") {
            PlayerScreen(
                PlayerModel(active.copy(record = record), view.limits, held = held, click = click),
                send = { true },
            )
        }
    }

    @Test
    fun recordSheets() = runTest {
        val store = libraryStore()
        store.send(Event.Session(SessionEvent.StartBuilding))
        store.send(Event.Session(SessionEvent.AddToSetlist(store.scalesId())))
        store.send(Event.Session(SessionEvent.StartSession(PlayerFixtures.STARTED)))
        val active = store.active()
        captureRoboImage("src/test/snapshots/player-variation-picker.png") {
            SheetPage("Switch variation", {}, "variationPicker.cancel") {
                VariationPickerContent(
                    VariationChoice(
                        active.currentVariations,
                        active.currentVariations.first().id,
                    )
                ) {
                    true
                }
            }
        }
        captureRoboImage("src/test/snapshots/player-trouble-spot.png") {
            SheetPage("Trouble spot", {}, "troubleSpot.cancel") {
                TroubleSpotContent("Clair de Lune · in A1", null) { _, _ -> }
            }
        }
    }

    @Test
    fun reflectionRecord() = runTest {
        val store = scalesReflection()
        val view = store.viewModel.value ?: error("no view")
        val active = store.active()
        val reflection = active.reflection ?: error("no reflection")
        val entry = active.entries.first()
        val finish = active.record.finish
        val model =
            ReflectionModel(
                active.currentItemTitle,
                reflection,
                entry.plays,
                finish,
                null,
                view.limits,
                finish?.let {
                    PlayWayChoices(
                        it.sections,
                        it.keys,
                        active.currentVariations,
                        entry.plannedLabel,
                    )
                },
            )
        val state =
            ReflectionState(reflection.answers, reflection.tempos).apply {
                changingPlayId = entry.plays.last().id
            }
        captureRoboImage("src/test/snapshots/player-reflection-record.png") {
            ReflectionSheet(model, state, ReflectionActions({}, {}, {}))
        }
    }

    @Test
    fun summaryWithANote() = runTest {
        val store = openedStore()
        store.startTwoItems()
        val at = "2026-10-07T09:06:00Z"
        store.send(Event.Session(SessionEvent.EndSessionEarly(at, PlayerFixtures.silent)))
        val first = store.viewModel.value?.summary?.entries?.first() ?: error("no summary")
        store.send(
            Event.Session(SessionEvent.UpdateEntryNotes(first.id, "Left hand rushed in bar 12"))
        )
        val view = store.viewModel.value ?: error("no view")
        val summary = view.summary ?: error("no summary")
        captureRoboImage("src/test/snapshots/player-summary-note.png") {
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

private suspend fun TestScope.scalesReflection(): Store {
    val store = libraryStore()
    store.send(Event.Session(SessionEvent.StartBuilding))
    store.send(Event.Session(SessionEvent.AddToSetlist(store.scalesId())))
    store.send(Event.Session(SessionEvent.StartSession(PlayerFixtures.STARTED)))
    val started = store.active()
    store.send(
        Event.Session(
            SessionEvent.SwitchPlay(
                started.entries.first().id,
                null,
                null,
                listOf(started.currentVariations.first().id),
                "2026-10-07T09:02:00Z",
                PlayerFixtures.silent,
            )
        )
    )
    store.send(
        Event.Session(SessionEvent.PrepareReflection("2026-10-07T09:04:10Z", PlayerFixtures.silent))
    )
    val draft =
        ReflectionAnswers(
            marks = emptyList(),
            note = "left hand rushed in bar 12, got it at 84",
            tempos = emptyList(),
            gotInTheWay = emptyList(),
            notePoints = emptyList(),
            ways = emptyList(),
        )
    store.send(Event.Session(SessionEvent.UpdateReflectionDraft(draft)))
    return store
}

private object SilentClick : ClickOutput {
    override var onPulseDied: (() -> Unit)? = null

    override fun start(bpm: Int, beats: Int, sounding: Int) = true

    override fun stop() = Unit
}
