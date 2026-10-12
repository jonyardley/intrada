package com.intrada.android

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
 * does one created in the picker (#2569).
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
}
