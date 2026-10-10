package com.intrada.android

import com.intrada.android.core.CoreBridge
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.ItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Reporter
import com.intrada.android.core.SharedItemStore
import com.intrada.android.core.Store
import com.intrada.android.core.StoreFailure
import com.intrada.android.core.stepName
import com.intrada.android.core.withIds
import com.intrada.ffi.CoreException
import com.intrada.ffi.InternalException
import com.intrada.ffi.StoreFfi
import com.intrada.shared.CreateItem
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.ItemSection
import com.intrada.shared.PersistenceOperation
import com.intrada.shared.PersistenceOutput
import com.intrada.shared.RecognitionOutput
import com.intrada.shared.RecordKey
import com.intrada.shared.RecordKind
import com.intrada.shared.Request
import com.intrada.shared.SectionKind
import com.intrada.shared.ViewModel
import java.io.File
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
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

    @Test
    fun aCorePanicIsReportedAsOne() = runTest {
        val reporter = RecordingReporter()
        val panic = InternalException("panicked at app.rs")
        val store = store(ScriptedBridge(listOf(panic)), reporter = reporter)

        store.send(Event.ClearError)

        assertEquals(listOf(panic to "core-panic"), reporter.reports)
    }

    @Test
    fun aBridgeFailureIsReportedAsOne() = runTest {
        val reporter = RecordingReporter()
        val failure = CoreException.Bridge("decode")
        val store = store(ScriptedBridge(listOf(failure)), reporter = reporter)

        store.send(Event.ClearError)

        assertEquals(listOf(failure to "bridge"), reporter.reports)
    }

    @Test
    fun eachSendLeavesAStep() = runTest {
        val reporter = RecordingReporter()
        val store = store(ScriptedBridge(emptyList()), reporter = reporter)

        store.send(Event.ClearError)
        store.send(Event.ClearNotice)

        assertEquals(listOf("ClearError", "ClearNotice"), reporter.steps)
    }

    @Test
    fun aSavedPieceIsThereWhenTheAppOpensAgain() = runTest {
        val file = File.createTempFile("intrada", ".sqlite").also { it.delete() }
        try {
            val first = store(LiveBridge(), SharedItemStore.open(file.path, log = {}).store)
            first.send(Event.StartApp)
            first.settle()
            first.send(
                Event.Item(
                    ItemEvent.Add(
                        CreateItem(
                            title = "Clair de Lune",
                            kind = ItemKind.PIECE,
                            tags = emptyList(),
                            variationLabels = emptyList(),
                        )
                    )
                )
            )
            first.settle()

            val reopened = store(LiveBridge(), SharedItemStore.open(file.path, log = {}).store)
            reopened.send(Event.StartApp)
            reopened.settle()

            assertEquals(listOf("Clair de Lune"), reopened.libraryRows.value.map { it.title })
        } finally {
            file.delete()
        }
    }

    @Test
    fun aDatabaseThatWillNotOpenFallsBackToMemoryAndSaysSo() = runTest {
        val reporter = RecordingReporter()
        val opened =
            SharedItemStore.open("/nonexistent-dir/intrada.sqlite", log = {}, reporter = reporter)
        assertTrue(opened.degraded)
        assertEquals(listOf("store-open"), reporter.reports.map { it.second })

        val store = store(LiveBridge(), opened.store)
        store.send(Event.StartApp)
        store.settle()

        assertNull(store.viewModel.value?.error)
    }

    @Test
    fun aFailedDiskJobIsReportedAsPersistence() = runTest {
        val reporter = RecordingReporter()
        val store = store(LiveBridge(), Fixtures.FailingItemStore, reporter = reporter)

        store.send(Event.StartApp)
        store.settle()

        assertTrue(reporter.reports.isNotEmpty())
        assertTrue(reporter.reports.all { it.second == "persistence" })
    }

    @Test
    fun aWriteTheStoreRefusesIsThrownForTheStoreToReport() {
        val items = SharedItemStore(StoreFfi.inMemory(), log = {})
        val unstorable =
            ItemSection(
                id = "s1",
                name = "A",
                bars = null,
                kind = SectionKind.FORM,
                targetBpm = null,
                position = ULong.MAX_VALUE,
                updatedAt = "2026-09-01T09:00:00Z",
                deletedAt = null,
            )
        val broken = Fixtures.item().copy(sections = listOf(unstorable))

        assertThrows(StoreFailure::class.java) { items.run(PersistenceOperation.SaveItem(broken)) }
    }

    @Test
    fun theInMemoryStoreKeepsTombstonesAndLoadsOnlyTheKeysAskedFor() {
        val store = InMemoryItemStore(Fixtures.library)
        val gone = Fixtures.library[0].id
        store.run(PersistenceOperation.DeleteItem(gone, "2026-09-02T09:00:00Z"))

        val records =
            store.run(
                PersistenceOperation.LoadRecords(
                    listOf(RecordKey(RecordKind.ITEM, gone), RecordKey(RecordKind.ITEM, "absent"))
                )
            ) as PersistenceOutput.Records
        assertEquals(listOf(gone), records.value.items.map { it.item.id })
        assertEquals(listOf("2026-09-02T09:00:00Z"), records.value.items.map { it.deletedAt })
        val live = store.run(PersistenceOperation.LoadItems) as PersistenceOutput.Items
        assertFalse(live.value.any { it.id == gone })
    }

    private fun TestScope.store(
        bridge: CoreBridge,
        items: ItemStore = InMemoryItemStore(),
        reporter: Reporter = RecordingReporter(),
    ) =
        Store(
            bridge,
            items,
            scope = this,
            io = StandardTestDispatcher(testScheduler),
            log = {},
            reporter = reporter,
        )
}

private class RecordingReporter : Reporter {
    val reports = mutableListOf<Pair<Throwable, String>>()
    val steps = mutableListOf<String>()

    override fun report(error: Throwable, context: String) {
        reports += error to context
    }

    override fun step(event: Event) {
        steps += stepName(event)
    }
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
