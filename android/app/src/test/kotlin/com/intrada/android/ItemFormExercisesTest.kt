package com.intrada.android

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertContentDescriptionContains
import androidx.compose.ui.test.junit4.StateRestorationTester
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onLast
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import com.intrada.android.core.InMemoryItemStore
import com.intrada.android.core.LiveBridge
import com.intrada.android.core.Store
import com.intrada.android.ui.LibraryAddRoute
import com.intrada.shared.Event
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ItemFormExercisesTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun aPieceSavesWithAWrittenAndAChosenExerciseInOneGo() = runTest {
        val store = startedStore()
        var done = false
        compose.setContent { LibraryAddRoute(store, onDone = { done = true }) }

        compose.onNodeWithTag("itemForm.title").performTextInput("Autumn Leaves")
        openPicker()
        compose.onNodeWithTag("exercisePicker.row").performClick()
        write("Shell voicings")
        compose.onNodeWithTag("exercisePicker.done").performClick()
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertTrue(done)
        val piece = store.libraryRows.value.single { it.title == "Autumn Leaves" }
        assertEquals(
            listOf("Shell voicings", "Hanon No. 1"),
            piece.linkedExercises.map { it.title },
        )
    }

    @Test
    fun aRefusedWrittenRowIsTheOneMarked() = runTest {
        val store = startedStore()
        var done = false
        compose.setContent { LibraryAddRoute(store, onDone = { done = true }) }

        compose.onNodeWithTag("itemForm.title").performTextInput("Autumn Leaves")
        openPicker()
        write("Shell voicings")
        write("Guide tones", bpm = "999")
        compose.onNodeWithTag("exercisePicker.done").performClick()
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()

        assertFalse(done)
        compose
            .onNodeWithTag("itemForm.error")
            .assertContentDescriptionContains(BPM_REFUSAL, substring = true)
        assertEquals(listOf(null, "Tempo. $FAULT_HINT"), rowStates())
        assertNull(store.libraryRows.value.firstOrNull { it.title == "Autumn Leaves" })
    }

    @Test
    fun removingTheMarkedRowClearsTheMark() = runTest {
        val store = startedStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }

        compose.onNodeWithTag("itemForm.title").performTextInput("Autumn Leaves")
        openPicker()
        write("Guide tones", bpm = "999")
        write("Shell voicings")
        compose.onNodeWithTag("exercisePicker.done").performClick()
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()
        assertEquals(listOf("Tempo. $FAULT_HINT", null), rowStates())
        compose.onAllNodesWithTag("itemForm.exercise.remove")[0].performScrollTo().performClick()

        assertEquals(listOf<String?>(null), rowStates())
    }

    @Test
    fun switchingToAnExerciseDropsTheStagedRows() = runTest {
        val store = startedStore()
        compose.setContent { LibraryAddRoute(store, onDone = {}) }

        openPicker()
        compose.onNodeWithTag("exercisePicker.row").performClick()
        compose.onNodeWithTag("exercisePicker.done").performClick()
        assertEquals(1, rowStates().size)

        compose.onNodeWithTag("itemForm.kind.exercise").performScrollTo().performClick()
        compose.onNodeWithTag("itemForm.kind.piece").performScrollTo().performClick()
        assertTrue(rowStates().isEmpty())
    }

    @Test
    fun stagedRowsSurviveTheActivityBeingRecreated() = runTest {
        val store = startedStore()
        val restoration = StateRestorationTester(compose)
        restoration.setContent { LibraryAddRoute(store, onDone = {}) }

        openPicker()
        compose.onNodeWithTag("exercisePicker.row").performClick()
        write("Shell voicings", bpm = "80", key = true)
        compose.onNodeWithTag("exercisePicker.done").performClick()
        val before = rowTexts()
        restoration.emulateSavedInstanceStateRestore()

        assertEquals(2, before.size)
        assertTrue("the written row shows its key and tempo", " · " in before[0][1])
        assertEquals(before, rowTexts())
    }

    @Test
    fun theMarkSurvivesTheActivityBeingRecreated() = runTest {
        val store = startedStore()
        val restoration = StateRestorationTester(compose)
        restoration.setContent { LibraryAddRoute(store, onDone = {}) }

        compose.onNodeWithTag("itemForm.title").performTextInput("Autumn Leaves")
        openPicker()
        write("Shell voicings")
        write("Guide tones", bpm = "999")
        compose.onNodeWithTag("exercisePicker.done").performClick()
        compose.onNodeWithTag("itemForm.confirm").performClick()
        store.settle()
        restoration.emulateSavedInstanceStateRestore()

        assertEquals(listOf(null, "Tempo. $FAULT_HINT"), rowStates())
    }

    private fun openPicker() {
        compose.onNodeWithTag("itemForm.addExercise").performScrollTo().performClick()
    }

    private fun write(title: String, bpm: String = "", key: Boolean = false) {
        compose.onNodeWithTag("exercisePicker.create").performScrollTo().performClick()
        compose.onNodeWithTag("writtenExercise.title").performTextInput(title)
        if (key) {
            // The form's own key picker sits under the pop-up, so the pop-up's is the last.
            compose.onAllNodesWithTag("itemForm.key").onLast().performClick()
            compose.onNodeWithTag("itemForm.key.major.1").performScrollTo().performClick()
        }
        if (bpm.isNotEmpty()) compose.onNodeWithTag("writtenExercise.bpm").performTextInput(bpm)
        compose.onNodeWithTag("writtenExercise.done").performClick()
    }

    private fun rowTexts(): List<List<String>> =
        compose.onAllNodesWithTag("itemForm.exercise").fetchSemanticsNodes().map { node ->
            node.children.flatMap { child ->
                child.config.getOrNull(SemanticsProperties.Text).orEmpty().map { it.text }
            }
        }

    private fun rowStates(): List<String?> =
        compose.onAllNodesWithTag("itemForm.exercise").fetchSemanticsNodes().map { node ->
            node.children.firstNotNullOfOrNull {
                it.config.getOrNull(SemanticsProperties.StateDescription)
            }
        }

    private suspend fun TestScope.startedStore(): Store {
        val store =
            Store(
                LiveBridge(),
                InMemoryItemStore(Fixtures.library),
                this,
                StandardTestDispatcher(testScheduler),
                log = {},
            )
        store.send(Event.StartApp)
        store.settle()
        return store
    }

    private companion object {
        const val BPM_REFUSAL = "BPM must be a whole number between"
        const val FAULT_HINT = "The message at the top of the form is about this"
    }
}
