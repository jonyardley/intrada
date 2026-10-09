package com.intrada.android

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import com.intrada.android.core.Store
import com.intrada.android.ui.AddToSessionRoute
import com.intrada.android.ui.BuilderNavigation
import com.intrada.android.ui.BuilderRoute
import com.intrada.android.ui.EntrySettingsRoute
import com.intrada.android.ui.PracticeRoute
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class BuilderScreenTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun thePracticeTabStartsBuildingAndOpensTheBuilder() = runTest {
        val store = libraryStore()
        var opened = false
        compose.setContent { PracticeRoute(store, onBuild = { opened = true }) }

        compose.onNodeWithTag("practice.start").performClick()

        assertTrue(opened)
        assertEquals(emptyList<String>(), store.setlist().titles())
    }

    @Test
    fun tickingALibraryRowAddsItAndTickingAgainTakesItOut() = runTest {
        val store = libraryStore()
        store.send(Event.Session(SessionEvent.StartBuilding))
        compose.setContent { AddToSessionRoute(store, onDone = {}) }

        compose.onNodeWithContentDescription("Gymnopédie No. 1", substring = true).performClick()
        assertEquals(listOf("Gymnopédie No. 1"), store.setlist().titles())

        compose.onNodeWithContentDescription("Gymnopédie No. 1", substring = true).performClick()
        assertEquals(emptyList<String>(), store.setlist().titles())
    }

    @Test
    fun aSectionChosenInSettingsIsPlannedOnThePiece() = runTest {
        val store = buildingStore()
        val piece = store.setlist().entries.single { it.itemId == BuilderFixtures.PIECE }
        compose.setContent { EntrySettingsRoute(store, piece.id, onDone = {}) }

        compose.onNodeWithContentDescription("Add Run").performScrollTo().performClick()

        val planned = store.setlist().entries.single { it.id == piece.id }
        assertEquals(listOf("Run"), planned.record.segments.map { it.label })
    }

    @Test
    fun aVariationChosenInSettingsIsPlannedOnTheExercise() = runTest {
        val store = buildingStore()
        store.send(Event.Session(SessionEvent.AddToSetlist(store.scalesId())))
        val scales = store.setlist().entries.single { it.itemId == store.scalesId() }
        compose.setContent { EntrySettingsRoute(store, scales.id, onDone = {}) }

        compose
            .onNodeWithContentDescription("Swung", substring = true)
            .performScrollTo()
            .performClick()

        val planned = store.setlist().entries.single { it.id == scales.id }
        val swung = store.setlist().entryVariations.single { it.entryId == scales.id }
        assertEquals(
            swung.variations.filter { it.label == "Swung" }.map { it.id },
            planned.plannedVariationIds,
        )
    }

    @Test
    fun trackingRepetitionsStartsAtTheDefaultAndStepsDown() = runTest {
        val store = buildingStore()
        val satie = store.setlist().entries.single { it.itemId == BuilderFixtures.SATIE }
        compose.setContent { EntrySettingsRoute(store, satie.id, onDone = {}) }
        val default = store.viewModel.value?.limits?.repTargetDefault

        compose.onNodeWithTag("entrySettings.trackReps").performScrollTo().performClick()
        assertEquals(default, store.repTarget(satie.id))

        compose.onNodeWithContentDescription("Less repetitions").performScrollTo().performClick()
        assertEquals(default?.minus(1u)?.toUByte(), store.repTarget(satie.id))
    }

    @Test
    fun anAimTheCoreRefusesShowsItsErrorOnTheSettingsScreen() = runTest {
        val store = buildingStore()
        val satie = store.setlist().entries.single { it.itemId == BuilderFixtures.SATIE }
        compose.setContent { EntrySettingsRoute(store, satie.id, onDone = {}) }

        compose.onNodeWithTag("entrySettings.aim").performTextInput("a".repeat(TOO_LONG_AIM))
        compose.waitForIdle()

        compose.onNodeWithTag("banner.error").assertIsDisplayed()
    }

    @Test
    fun movingSatieUpInEditModePutsItFirst() = runTest {
        val store = buildingStore()
        compose.setContent { BuilderRoute(store, navigation()) }

        compose.onNodeWithTag("builder.edit").performClick()
        compose.onNodeWithContentDescription("Move Gymnopédie No. 1 up").performClick()

        assertEquals(
            listOf("Gymnopédie No. 1", "Hanon No. 1", "Clair de Lune"),
            store.setlist().titles(),
        )
    }

    @Test
    fun removingTheBlockFromItsMenuLeavesSatie() = runTest {
        val store = buildingStore()
        compose.setContent { BuilderRoute(store, navigation()) }

        compose.onNodeWithTag("builder.blockMenu").performClick()
        compose.onNodeWithTag("builder.menu.remove").performClick()

        assertEquals(listOf("Gymnopédie No. 1"), store.setlist().titles())
        compose.onAllNodesWithTag("builder.header").assertCountEquals(0)
    }

    @Test
    fun cancellingAPlanAsksFirstThenLeavesBuilding() = runTest {
        val store = buildingStore()
        var closed = false
        compose.setContent { BuilderRoute(store, navigation(onClosed = { closed = true })) }

        compose.onNodeWithTag("builder.cancel").performClick()
        assertEquals(3, store.setlist().entries.size)

        compose.onNodeWithTag("builder.discard.confirm").performClick()
        compose.waitForIdle()

        assertNull(store.viewModel.value?.buildingSetlist)
        assertTrue(closed)
    }

    private fun navigation(onClosed: () -> Unit = {}) =
        BuilderNavigation(onAddItems = {}, onAddExercise = {}, onEntry = {}, onClosed = onClosed)

    private companion object {
        const val TOO_LONG_AIM = 501
    }

    private fun Store.repTarget(entryId: String) =
        setlist().entries.single { it.id == entryId }.plannedRepTarget
}
