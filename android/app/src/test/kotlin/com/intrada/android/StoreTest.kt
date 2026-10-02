package com.intrada.android

import com.intrada.android.core.CoreBridge
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.ItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.core.withIds
import com.intrada.ffi.CoreException
import com.intrada.ffi.InternalException
import com.intrada.shared.Event
import com.intrada.shared.ItemKind
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.RecognitionOutput
import com.intrada.shared.Request
import com.intrada.shared.ViewModel
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class StoreTest {
    @Test
    fun startAppShowsTheStoredLibrary() = runTest {
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(Fixtures.library),
                scope = this,
                io = StandardTestDispatcher(testScheduler),
                log = {},
            )
        assertTrue(store.libraryRows.value.isEmpty())

        store.send(Event.StartApp)
        store.settle()

        val view = checkNotNull(store.viewModel.value)
        val shown = store.libraryRows.value.withIds(view.visibleIds)
        assertEquals(
            setOf("Clair de Lune", "Gymnopédie No. 1", "Hanon No. 1"),
            shown.map { it.title }.toSet(),
        )
        assertEquals(ItemKind.EXERCISE, shown.single { it.title == "Hanon No. 1" }.itemType)
    }

    @Test
    fun aFailedLoadSurfacesTheCoresMessage() = runTest {
        val store = store(LiveBridge(), Fixtures.FailingItemStore)
        assertNull(store.viewModel.value?.error)

        store.send(Event.StartApp)
        store.settle()

        assertNotNull(store.viewModel.value?.error)
        assertFalse(store.halted.value)
    }

    @Test
    fun clearErrorTakesTheMessageAway() = runTest {
        val store = store(LiveBridge(), Fixtures.FailingItemStore)
        store.send(Event.StartApp)
        store.settle()
        assertNotNull(store.viewModel.value?.error)

        store.send(Event.ClearError)

        assertNull(store.viewModel.value?.error)
    }

    @Test
    fun oneBridgeFailureDoesNotHalt() = runTest {
        val bridge = ScriptedBridge(listOf(CoreException.Bridge("decode")))
        val store = store(bridge)

        store.send(Event.ClearError)
        store.send(Event.ClearError)

        assertFalse(store.halted.value)
        assertEquals(2, bridge.updates)
    }

    @Test
    fun twoBridgeFailuresRunningHaltAndRefuseSends() = runTest {
        val bridge =
            ScriptedBridge(listOf(CoreException.Bridge("decode"), CoreException.Bridge("decode")))
        val store = store(bridge)

        store.send(Event.ClearError)
        assertFalse(store.halted.value)
        store.send(Event.ClearError)
        assertTrue(store.halted.value)

        store.send(Event.ClearError)
        assertEquals(2, bridge.updates)
    }

    @Test
    fun aSuccessBetweenFailuresKeepsTheStoreRunning() = runTest {
        val failure = CoreException.Bridge("decode")
        val bridge = ScriptedBridge(listOf(failure, null, failure, failure))
        val store = store(bridge)

        store.send(Event.ClearError)
        store.send(Event.ClearError)
        store.send(Event.ClearError)
        assertFalse(store.halted.value)

        store.send(Event.ClearError)
        assertTrue(store.halted.value)
    }

    @Test
    fun aCorePanicHaltsAtOnce() = runTest {
        val store = store(ScriptedBridge(listOf(InternalException("panicked at app.rs"))))

        store.send(Event.ClearError)

        assertTrue(store.halted.value)
    }

    private fun TestScope.store(bridge: CoreBridge, items: ItemStore = InMemoryItemStore()) =
        Store(
            bridge,
            items,
            scope = this,
            io = StandardTestDispatcher(testScheduler),
            log = {},
        )
}

private class ScriptedBridge(failures: List<Exception?>) : CoreBridge {
    private val pending = ArrayDeque(failures)
    private val live = LiveBridge()
    var updates = 0
        private set

    override fun update(event: Event): List<Request> {
        updates += 1
        pending.removeFirstOrNull()?.let { throw it }
        return emptyList()
    }

    override fun resolve(id: UInt, output: PersistenceOutput): List<Request> = emptyList()

    override fun resolve(id: UInt, output: RecognitionOutput): List<Request> = emptyList()

    override fun view(): ViewModel = live.view()
}
