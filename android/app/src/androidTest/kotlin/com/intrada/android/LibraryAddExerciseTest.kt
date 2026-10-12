package com.intrada.android

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
 * The add form's one "Add exercise" button: creating and choosing both land in the form (#1616).
 */
@RunWith(AndroidJUnit4::class)
class LibraryAddExerciseTest {
    @get:Rule val compose = createEmptyComposeRule()
    private val app = App(compose)

    @After fun close() = app.close()

    @Test
    fun creatingAndChoosingInThePickerBothLandInTheForm() {
        app.launch(seed = true)
        app.tapTab(AppTab.LIBRARY.tag)
        app.tap("library.add")
        app.control("itemForm.title").performTextInput("Nocturne in E flat")

        app.tap("itemForm.addExercise")
        app.tap("exercisePicker.create")
        app.control("writtenExercise.title").performTextInput("Chromatic run")
        app.tap("writtenExercise.done")
        app.row("exercisePicker.written", "Chromatic run")

        app.tapRow("exercisePicker.row", "Hanon No. 1")
        app.tap("exercisePicker.done")

        app.assertGone(hasTestTag("exercisePicker.done"), "the picker closed")
        app.row("itemForm.exercise", "Chromatic run")
        app.row("itemForm.exercise", "Hanon No. 1")
    }
}
