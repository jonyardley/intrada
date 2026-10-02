package com.intrada.android

import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.core.withIds
import com.intrada.shared.Event
import com.intrada.shared.ItemKind
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
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
}
