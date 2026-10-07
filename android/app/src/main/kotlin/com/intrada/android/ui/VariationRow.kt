package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import com.intrada.android.R
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.scaled
import java.util.UUID

/**
 * One row of the Variations card. [variantId] is the saved variation it was loaded from, sent by id
 * only; null for a row typed on the form.
 */
class VariationRow(
    val variantId: String?,
    val label: String,
    val hasMarks: Boolean = false,
    val id: String = UUID.randomUUID().toString(),
)

internal fun SnapshotStateList<VariationRow>.move(from: Int, to: Int) {
    if (from !in indices || to !in indices || from == to) return
    add(to, removeAt(from))
}

private const val LIFT_ELEVATION = 8f

/** An exercise's variations: long-press the grip to drag, removed with a warning if marked. */
@Composable
internal fun VariationRowsCard(
    rows: SnapshotStateList<VariationRow>,
    modifier: Modifier = Modifier,
) {
    val drag = remember { DragState() }
    var confirming by rememberSaveable { mutableStateOf<String?>(null) }
    Column(modifier.cardSurface()) {
        FieldLabel(
            "Variations",
            Modifier.padding(horizontal = IntradaSpacing.card)
                .padding(top = IntradaSpacing.cardCompact),
        )
        rows.forEach { row ->
            val lifted = row.id == drag.row
            Box(
                Modifier.onSizeChanged { drag.heights[row.id] = it.height }
                    .zIndex(if (lifted) 1f else 0f)
                    .graphicsLayer {
                        translationY = if (lifted) drag.offset else 0f
                        shadowElevation = if (lifted) LIFT_ELEVATION.dp.toPx() else 0f
                    }
            ) {
                VariationRowView(
                    row,
                    rows,
                    drag,
                    onRemove = {
                        if (row.hasMarks) confirming = row.id
                        else rows.removeAll { it.id == row.id }
                    },
                )
            }
        }
        AddInputRow("Add a variation", "itemForm.variation", { rows.add(VariationRow(null, it)) })
    }
    val pending = rows.firstOrNull { it.id == confirming }
    if (pending != null) {
        ConfirmDialog(
            ConfirmCopy(
                "Remove ${pending.label}?",
                "Its marks go with it.",
                "Remove",
                "itemForm.variationRemoval",
            ),
            onConfirm = {
                rows.removeAll { it.id == pending.id }
                confirming = null
            },
            onDismiss = { confirming = null },
        )
    }
}

// TalkBack gets move up and down, since a drag alone is not screen-reader-operable.
@Composable
private fun VariationRowView(
    row: VariationRow,
    rows: SnapshotStateList<VariationRow>,
    drag: DragState,
    onRemove: () -> Unit,
) {
    val index = rows.indexOfFirst { it.id == row.id }
    Column {
        Row(
            Modifier.fillMaxWidth()
                .heightIn(min = 48.dp)
                .padding(start = IntradaSpacing.controlGap, end = IntradaSpacing.controlGap),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                Modifier.size(48.dp)
                    .reorderable(row, rows, drag)
                    .semantics {
                        contentDescription = "Reorder ${row.label}"
                        customActions =
                            listOfNotNull(
                                CustomAccessibilityAction("Move up") {
                                        rows.move(index, index - 1)
                                        true
                                    }
                                    .takeIf { index > 0 },
                                CustomAccessibilityAction("Move down") {
                                        rows.move(index, index + 1)
                                        true
                                    }
                                    .takeIf { index < rows.lastIndex },
                            )
                    }
                    .testTag("itemForm.variation.reorder"),
                contentAlignment = Alignment.Center,
            ) {
                Image(
                    painterResource(R.drawable.ic_grip),
                    contentDescription = null,
                    modifier = Modifier.size(IntradaIconSize.inline.scaled()),
                    colorFilter =
                        ColorFilter.tint(
                            if (row.id == drag.row) IntradaColor.ink else IntradaColor.inkFaintIcon
                        ),
                )
            }
            BasicText(
                row.label,
                Modifier.weight(1f).testTag("itemForm.variation.label"),
                style = IntradaFont.body.copy(color = IntradaColor.ink),
            )
            IconAction(
                R.drawable.ic_minus_circle,
                "Remove ${row.label}",
                onRemove,
                Modifier.testTag("itemForm.variation.remove"),
                tint = IntradaColor.danger,
            )
        }
        HairlineDivider()
    }
}

private fun Modifier.reorderable(
    row: VariationRow,
    rows: SnapshotStateList<VariationRow>,
    drag: DragState,
): Modifier =
    pointerInput(row.id) {
        detectDragGesturesAfterLongPress(
            onDragStart = {
                drag.row = row.id
                drag.offset = 0f
            },
            onDrag = { change, amount ->
                change.consume()
                drag.offset += amount.y
            },
            onDragEnd = {
                val from = rows.indexOfFirst { it.id == row.id }
                val heights = rows.map { drag.heights[it.id] ?: 0 }
                val destination = BuilderRow.dropDestination(heights, from, drag.offset)
                rows.move(from, if (destination > from) destination - 1 else destination)
                drag.reset()
            },
            onDragCancel = { drag.reset() },
        )
    }
