package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.toMutableStateList
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.intrada.android.R
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.bar
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.FormErrorField
import com.intrada.shared.FormErrorTarget
import com.intrada.shared.ItemKind
import com.intrada.shared.Key
import com.intrada.shared.LibraryItemView

@Composable
internal fun ItemFormExercises(
    form: ItemFormState,
    library: List<LibraryItemView>,
    modifier: Modifier = Modifier,
) {
    var choosing by remember { mutableStateOf(false) }
    Column(modifier.cardSurface()) {
        FieldLabel(
            "Related exercises",
            Modifier.padding(horizontal = IntradaSpacing.card)
                .padding(top = IntradaSpacing.cardCompact, bottom = IntradaSpacing.controlGap),
        )
        if (form.exercises.isEmpty()) {
            BasicText(
                "Scales, arpeggios, and anything else you practise alongside this piece.",
                Modifier.padding(horizontal = IntradaSpacing.card)
                    .padding(bottom = IntradaSpacing.cardCompact),
                style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
            )
        }
        form.exercises.forEachIndexed { index, staged ->
            HairlineDivider()
            val fault = form.faultedExercise?.takeIf { it.index.toInt() == index }
            StagedExerciseRow(staged, fault, onRemove = { form.removeExercise(staged.id) })
        }
        HairlineDivider()
        AddRow(
            "Add exercise",
            "Add an exercise for this piece",
            "itemForm.addExercise",
            { choosing = true },
        )
    }
    if (choosing) {
        StagedExercisePicker(
            library,
            form.exercises.toList(),
            onCancel = { choosing = false },
            onDone = {
                form.chooseExercises(it)
                choosing = false
            },
        )
    }
}

@Composable
private fun StagedExerciseRow(
    staged: StagedExercise,
    fault: FormErrorTarget.Exercise?,
    onRemove: () -> Unit,
    tag: String = "itemForm.exercise",
) {
    Row(
        Modifier.fillMaxWidth()
            .height(IntrinsicSize.Min)
            .heightIn(min = 48.dp)
            .background(if (fault != null) IntradaColor.dangerWash else IntradaColor.cardFill)
            .testTag(tag),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier.width(4.dp)
                .fillMaxHeight()
                .then(
                    if (fault != null) Modifier.background(IntradaColor.danger)
                    else Modifier.background(ItemKind.EXERCISE.bar)
                )
        )
        Column(
            Modifier.weight(1f)
                .padding(start = IntradaSpacing.card, top = IntradaSpacing.controlGap)
                .padding(bottom = IntradaSpacing.controlGap)
                .semantics(mergeDescendants = true) {
                    if (fault != null) stateDescription = faultSpoken(fault.field)
                },
            verticalArrangement = Arrangement.spacedBy(3.dp),
        ) {
            BasicText(staged.title, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
            staged.meta?.let {
                BasicText(it, style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary))
            }
        }
        IconAction(
            R.drawable.ic_minus_circle,
            "Remove ${staged.title}",
            onRemove,
            Modifier.padding(end = IntradaSpacing.controlGap).testTag("$tag.remove"),
            tint = IntradaColor.danger,
        )
    }
}

private const val FAULT_HINT = "The message at the top of the form is about this"

// A staged row shows only its title, so naming the field gives TalkBack the pointer the wash gives
// everyone else.
private fun faultSpoken(field: FormErrorField?): String {
    val name =
        when (field) {
            null -> return FAULT_HINT
            FormErrorField.TITLE -> "Title"
            FormErrorField.COMPOSER -> "Composer"
            FormErrorField.TEMPO -> "Tempo"
            FormErrorField.NOTES -> "Notes"
            FormErrorField.TAGS -> "Tags"
            FormErrorField.VARIATIONS -> "Variations"
            FormErrorField.SECTIONS -> "Sections"
        }
    return "$name. $FAULT_HINT"
}

@Composable
private fun StagedExercisePicker(
    library: List<LibraryItemView>,
    staged: List<StagedExercise>,
    onCancel: () -> Unit,
    onDone: (List<StagedExercise>) -> Unit,
) {
    val state = remember {
        ExercisePickerState(staged.filterIsInstance<StagedExercise.Chosen>().map { it.id }.toSet())
    }
    val written = remember {
        staged.filterIsInstance<StagedExercise.Written>().toMutableStateList()
    }
    var writing by remember { mutableStateOf(false) }
    FullScreenDialog(onCancel) {
        ExercisePickerScreen(
            library,
            state,
            onCancel,
            onDone = {
                val chosen =
                    library
                        .filter { it.id in state.chosen }
                        .map {
                            StagedExercise.Chosen(it.id, it.title, it.subtitle.ifEmpty { null })
                        }
                onDone(written + chosen)
            },
        ) {
            Column(Modifier.cardSurface()) {
                written.forEach { draft ->
                    StagedExerciseRow(
                        draft,
                        null,
                        onRemove = { written.remove(draft) },
                        tag = "exercisePicker.written",
                    )
                    HairlineDivider()
                }
                AddRow(
                    "Create an exercise",
                    "Create an exercise",
                    "exercisePicker.create",
                    { writing = true },
                )
            }
        }
    }
    if (writing) {
        WrittenExerciseDialog(
            onCancel = { writing = false },
            onDone = {
                written.add(it)
                writing = false
            },
        )
    }
}

@Composable
private fun WrittenExerciseDialog(onCancel: () -> Unit, onDone: (StagedExercise.Written) -> Unit) {
    var title by remember { mutableStateOf("") }
    var key by remember { mutableStateOf<Key?>(null) }
    var bpm by remember { mutableStateOf("") }
    FullScreenDialog(onCancel) {
        ScreenScaffold(
            "New exercise",
            actions = {
                TextAction("Cancel", "writtenExercise.cancel", onCancel)
                TextAction(
                    "Done",
                    "writtenExercise.done",
                    { onDone(StagedExercise.Written(title, key, bpm)) },
                    emphasised = true,
                    enabled = title.isNotBlank(),
                )
            },
        ) {
            Column(
                Modifier.fillMaxSize()
                    .verticalScroll(rememberScrollState())
                    .padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
            ) {
                Column(Modifier.cardSurface()) {
                    FormField(
                        "Title",
                        title,
                        { title = it },
                        "writtenExercise.title",
                        placeholder = "Required",
                    )
                    HairlineDivider()
                    KeyPicker(key, { key = it })
                    HairlineDivider()
                    FormField(
                        "Beats per minute",
                        bpm,
                        { bpm = it },
                        "writtenExercise.bpm",
                        keyboard = KeyboardType.Number,
                    )
                }
                BasicText(
                    "It joins the list here.",
                    style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
    }
}

@Composable
private fun FullScreenDialog(onDismiss: () -> Unit, content: @Composable () -> Unit) {
    Dialog(onDismiss, DialogProperties(usePlatformDefaultWidth = false)) { content() }
}
