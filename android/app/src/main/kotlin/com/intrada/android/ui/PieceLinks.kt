package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.bar
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.LibraryItemView
import com.intrada.shared.LinkChange
import com.intrada.shared.LinkedExerciseView

// ── Related exercises ──

@Composable
internal fun RelatedExercisesCard(
    item: LibraryItemView,
    state: PieceScreenState,
    actions: PieceActions,
) {
    val linked = item.linkedExercises
    val editing = state.editingLinks && linked.isNotEmpty()
    Column(Modifier.fillMaxWidth().cardSurface()) {
        SectionHeader(
            "Related exercises",
            Modifier.padding(start = IntradaSpacing.card, end = IntradaSpacing.controlGap)
                .padding(top = IntradaSpacing.controlGap),
            caption = linked.size.takeIf { it > 0 }?.toString(),
            action =
                HeaderAction(
                    if (editing) "Done" else "Edit",
                    if (editing) "Done editing related exercises" else "Edit related exercises",
                    "relatedExercises.header",
                    enabled = linked.isNotEmpty(),
                ) {
                    state.editingLinks = !editing
                },
        )
        if (linked.isEmpty()) {
            BasicText(
                "Scales, arpeggios, and anything else you practise alongside this piece.",
                Modifier.padding(horizontal = IntradaSpacing.card),
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        }
        linked.forEachIndexed { index, exercise ->
            if (index > 0) HairlineDivider()
            if (editing) {
                LinkedExerciseEditRow(item, exercise, index, linked.size, actions)
            } else {
                LinkedExerciseRow(exercise) { actions.navigation.onOpenExercise(exercise.id) }
            }
        }
        AddRow(
            "Add exercise",
            "Add an exercise for this piece",
            "relatedExercises.add",
            actions.navigation.onAddExercises,
        )
    }
}

@Composable
private fun LinkedExerciseRow(exercise: LinkedExerciseView, onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth()
            .height(IntrinsicSize.Min)
            .clickable(onClickLabel = "edit", role = Role.Button, onClick = onClick)
            .clearAndSetSemantics {
                contentDescription = exercise.spoken
                role = Role.Button
                testTag = "relatedExercises.row"
                onClick(label = "edit") {
                    onClick()
                    true
                }
            }
    ) {
        Box(Modifier.width(4.dp).fillMaxHeight().background(ItemKind.EXERCISE.bar))
        LinkedExerciseTitle(
            exercise,
            Modifier.weight(1f)
                .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.card),
        )
    }
}

@Composable
private fun LinkedExerciseEditRow(
    piece: LibraryItemView,
    exercise: LinkedExerciseView,
    index: Int,
    count: Int,
    actions: PieceActions,
) {
    fun change(change: LinkChange) =
        actions.send(Event.Item(ItemEvent.ChangePieceLink(piece.id, change)))
    Column(
        Modifier.fillMaxWidth()
            .padding(start = IntradaSpacing.card, end = IntradaSpacing.controlGap)
            .padding(vertical = IntradaSpacing.controlGap)
    ) {
        LinkedExerciseTitle(exercise)
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (piece.sections.isNotEmpty()) {
                TextAction(
                    "Choose sections",
                    "relatedExercises.sections",
                    { actions.navigation.onChooseSections(exercise.id) },
                    Modifier.semantics {
                        contentDescription = "Choose the sections ${exercise.title} is for"
                    },
                )
            }
            Spacer(Modifier.weight(1f))
            IconAction(
                R.drawable.ic_chevron_up,
                "Move ${exercise.title} up",
                onClick = { change(LinkChange.Move(exercise.id, (index - 1).toULong())) },
                enabled = index > 0,
            )
            IconAction(
                R.drawable.ic_chevron_down,
                "Move ${exercise.title} down",
                onClick = { change(LinkChange.Move(exercise.id, (index + 1).toULong())) },
                enabled = index < count - 1,
            )
            IconAction(
                R.drawable.ic_minus_circle,
                "Remove ${exercise.title} from related exercises",
                onClick = { change(LinkChange.Unlink(exercise.id)) },
                tint = IntradaColor.danger,
            )
        }
    }
}

@Composable
private fun LinkedExerciseTitle(exercise: LinkedExerciseView, modifier: Modifier = Modifier) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(3.dp)) {
        BasicText(exercise.title, style = IntradaFont.cardTitle.copy(color = IntradaColor.ink))
        listOfNotNull(exercise.metaLine, exercise.sectionsCaption).forEach {
            BasicText(it, style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary))
        }
    }
}
