package com.intrada.android

import androidx.compose.foundation.layout.Box
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.ui.PlayerHost
import com.intrada.android.ui.PracticeRoute
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class StartHereTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun onAnEmptyLibraryTheFirstStepOpensTheAddForm() = runTest {
        val store = openedStore(InMemoryItemStore())
        var adds = 0
        compose.setContent { PracticeRoute(store, onBuild = {}, onAdd = { adds++ }) }

        compose.onNodeWithTag("practice.startHere.next").performScrollTo().performClick()

        assertEquals(1, adds)
    }

    @Test
    fun theCardGoesOnceASessionIsMarkedAndStaysGoneAfterARestart() = runTest {
        val items = InMemoryItemStore(Fixtures.library)
        val store = openedStore(items)
        compose.setContent {
            Box {
                PracticeRoute(store, onBuild = {})
                PlayerHost(store)
            }
        }
        compose.onNodeWithTag("practice.startHere").assertExists()

        store.startTwoItems()
        compose.onNodeWithTag("player.advance").performClick()
        compose.onNodeWithTag("reflection.mark.4").performScrollTo().performClick()
        compose.onNodeWithTag("reflection.save").performScrollTo().performClick()
        compose.onNodeWithTag("player.advance").performClick()
        compose.onNodeWithTag("reflection.skip").performScrollTo().performClick()
        compose.onNodeWithTag("summary.save").performScrollTo().performClick()
        store.settle()

        compose.onNodeWithTag("practice.startHere").assertDoesNotExist()
        val restarted = openedStore(items)
        assertTrue(checkNotNull(restarted.viewModel.value).firstRun.marked)
        assertFalse(checkNotNull(restarted.viewModel.value).firstRun.showsStartHere)
    }
}
