package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.LibraryItemView

@Composable
fun ExercisePickerRoute(
    store: Store,
    pieceId: String,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val rows by store.libraryRows.collectAsState()
    val piece = rows.firstOrNull { it.id == pieceId }
    if (piece == null) {
        MissingItem("Add exercises", NO_LONGER_THERE, modifier)
        return
    }
    val exercises = rows.filter { it.itemType == ItemKind.EXERCISE }
    val state =
        remember(pieceId) { ExercisePickerState(piece.linkedExercises.map { it.id }.toSet()) }
    var closing by remember { mutableStateOf(false) }
    ExercisePickerScreen(
        exercises,
        state,
        onCancel = onDone,
        onDone = {
            if (!closing) {
                if (state.chosen != state.linked) {
                    val ids = exercises.map { it.id }.filter { it in state.chosen }
                    state.formError =
                        store.sendFromForm(
                            Event.Item(ItemEvent.ChoosePieceExercises(pieceId, ids, emptyList()))
                        )
                }
                closing = state.formError == null
                if (closing) onDone()
            }
        },
        modifier,
    )
}

@Composable
fun ExercisePickerScreen(
    exercises: List<LibraryItemView>,
    state: ExercisePickerState,
    onCancel: () -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    written: @Composable () -> Unit = {},
) {
    ScreenScaffold(
        "Add exercises",
        modifier,
        actions = {
            TextAction("Cancel", "exercisePicker.cancel", onCancel)
            TextAction("Done", "exercisePicker.done", onDone, emphasised = true)
        },
    ) {
        Column(
            Modifier.fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
        ) {
            state.formError?.let { FormErrorBanner(it, Modifier.testTag("exercisePicker.error")) }
            written()
            if (exercises.isEmpty()) {
                BasicText(
                    "No exercises in your library yet.",
                    style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
                )
            } else {
                Column(Modifier.cardSurface()) {
                    exercises.forEachIndexed { index, exercise ->
                        if (index > 0) HairlineDivider()
                        TickRow(
                            exercise.title,
                            exercise.subtitle.takeIf { it.isNotEmpty() },
                            chosen = exercise.id in state.chosen,
                            tag = "exercisePicker.row",
                            onToggle = { state.toggle(exercise.id) },
                        )
                    }
                }
            }
        }
    }
}
