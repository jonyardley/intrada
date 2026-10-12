package com.intrada.android

import androidx.compose.ui.test.assertIsOff
import androidx.compose.ui.test.assertIsOn
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.intrada.android.ui.AppTab
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * The piece screen's "Add exercise": a chosen exercise links to the piece on Done (#1431), and so
 * does one created in the picker (#2569). The whole set is one event (#2248), so one refused draft
 * saves nothing and leaves every choice in the picker for a second Done.
 */
@RunWith(AndroidJUnit4::class)
class LibraryDetailAddExerciseTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun aChosenExerciseLinksToThePiece() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tapRow("library.row", "Clair de Lune")

        app.tap("relatedExercises.add")
        app.tapRow("exercisePicker.row", "Hanon No. 1")
        app.tap("exercisePicker.done")

        app.row("relatedExercises.row", "Hanon No. 1")
    }

    @Test
    fun creatingAndChoosingInThePickerBothLinkToThePiece() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tapRow("library.row", "Clair de Lune")

        app.tap("relatedExercises.add")
        app.tap("exercisePicker.create")
        app.control("writtenExercise.title").performTextInput("Chromatic run")
        app.tap("writtenExercise.done")
        app.row("exercisePicker.written", "Chromatic run")
        app.tapRow("exercisePicker.row", "Hanon No. 1")
        app.tap("exercisePicker.done")

        app.row("relatedExercises.row", "Chromatic run")
        app.row("relatedExercises.row", "Hanon No. 1")
    }

    @Test
    fun aRefusedNewExerciseKeepsThePickerOpenWithEveryDraft() {
        openPicker("Clair de Lune")
        draft("Bad tempo", bpm = "12a")
        draft("Good tempo", bpm = "90")
        app.tap("exercisePicker.done")

        app.await(hasTestTag("exercisePicker.error") and App.mentions(BPM_REFUSAL))
        app.control("exercisePicker.done")
        app.row("exercisePicker.written", "Bad tempo")
        app.row("exercisePicker.written", "Good tempo")
        app.assertNeverShows(
            hasTestTag("exercisePicker.row") and App.mentions("Good tempo"),
            withinMillis = 1_000,
            "nothing was created, so it is not in the library list",
        )
    }

    @Test
    fun aRefusedSetKeepsAnUntickedExerciseUnticked() {
        openPicker("Nocturne")
        app.row("exercisePicker.row", "Major Scales").assertIsOn()
        app.tapRow("exercisePicker.row", "Major Scales")
        draft("Bad tempo", bpm = "12a")
        app.tap("exercisePicker.done")

        app.await(hasTestTag("exercisePicker.error") and App.mentions(BPM_REFUSAL))
        app.row("exercisePicker.row", "Major Scales").assertIsOff()

        app.tap("exercisePicker.cancel")
        app.row("relatedExercises.row", "Major Scales")
    }

    private fun openPicker(piece: String) {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tapRow("library.row", piece)
        app.tap("relatedExercises.add")
    }

    private fun draft(title: String, bpm: String) {
        app.tap("exercisePicker.create")
        app.control("writtenExercise.title").performTextInput(title)
        app.control("writtenExercise.bpm").performTextInput(bpm)
        app.tap("writtenExercise.done")
        app.row("exercisePicker.written", title)
    }

    private companion object {
        const val BPM_REFUSAL = "BPM must be a whole number between 1 and 400"
    }
}
