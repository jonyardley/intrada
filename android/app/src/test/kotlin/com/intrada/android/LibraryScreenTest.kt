package com.intrada.android

import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertContentDescriptionEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.ItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.LibraryRoute
import com.intrada.shared.Event
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class LibraryScreenTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun aCoreFailureShowsTheCoresMessageUntilDismissed() = runTest {
        val store = store(Fixtures.FailingItemStore)
        store.send(Event.StartApp)
        store.settle()
        val message = checkNotNull(store.viewModel.value?.error)
        show(store)

        val banner = compose.onNodeWithTag("banner.error")
        banner.assertContentDescriptionEquals(message)
        val dismiss =
            banner.fetchSemanticsNode().config[SemanticsActions.CustomActions].single {
                it.label == "Dismiss"
            }
        compose.runOnIdle { dismiss.action() }

        compose.onNodeWithTag("banner.error").assertDoesNotExist()
    }

    @Test
    fun eachRowReadsItsKindTitleAndSubtitle() = runTest {
        val store = store(InMemoryItemStore(Fixtures.library))
        store.send(Event.StartApp)
        store.settle()
        show(store)

        val spoken =
            compose.onAllNodesWithTag("library.row").fetchSemanticsNodes().map {
                it.config.getOrNull(SemanticsProperties.ContentDescription)?.single()
            }
        assertEquals(
            setOf(
                "Piece, Clair de Lune, Claude Debussy",
                "Piece, Gymnopédie No. 1, Erik Satie",
                "Exercise, Hanon No. 1",
            ),
            spoken.toSet(),
        )
        compose.onNodeWithTag("banner.error").assertDoesNotExist()
    }

    @Test
    fun tappingARowOpensThatItemAndPlusOpensTheAddForm() = runTest {
        val store = store(InMemoryItemStore(Fixtures.library))
        store.send(Event.StartApp)
        store.settle()
        val opened = mutableListOf<String>()
        var adds = 0
        compose.setContent {
            LibraryRoute(store, onAdd = { adds += 1 }, onOpen = { opened += it.id })
        }

        compose.onAllNodesWithTag("library.row")[0].performClick()
        compose.onNodeWithTag("library.add").performClick()

        assertEquals(listOf(store.viewModel.value?.visibleIds?.first()), opened)
        assertEquals(1, adds)
    }

    private fun show(store: Store) {
        compose.setContent { LibraryRoute(store, onAdd = {}, onOpen = {}) }
    }

    private fun TestScope.store(items: ItemStore) =
        Store(LiveBridge(), items, this, StandardTestDispatcher(testScheduler), log = {})
}
