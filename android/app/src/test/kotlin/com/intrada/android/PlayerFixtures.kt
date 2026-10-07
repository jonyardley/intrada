package com.intrada.android

import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.ItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Settings
import com.intrada.android.core.Store
import com.intrada.shared.ActiveSessionView
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import com.intrada.shared.TempoReading
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope

object PlayerFixtures {
    const val STARTED = "2026-10-07T09:00:00Z"
    val silent = TempoReading(bpm = 72.toUShort(), clickSounding = false)
}

suspend fun TestScope.openedStore(
    items: ItemStore = InMemoryItemStore(Fixtures.library),
    settings: Settings? = null,
): Store {
    val store =
        Store(
            LiveBridge(),
            items,
            this,
            StandardTestDispatcher(testScheduler),
            log = {},
            settings = settings,
        )
    store.send(Event.StartApp)
    store.settle()
    return store
}

/** Satie then Hanon, started at [PlayerFixtures.STARTED]. */
fun Store.startTwoItems() {
    send(Event.Session(SessionEvent.StartBuilding))
    send(Event.Session(SessionEvent.AddToSetlist(BuilderFixtures.SATIE)))
    send(Event.Session(SessionEvent.AddToSetlist(BuilderFixtures.HANON)))
    send(Event.Session(SessionEvent.StartSession(PlayerFixtures.STARTED)))
}

fun Store.active(): ActiveSessionView =
    viewModel.value?.activeSession ?: error("no session is running")
