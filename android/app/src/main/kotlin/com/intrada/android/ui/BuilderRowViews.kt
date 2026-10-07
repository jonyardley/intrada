package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import com.intrada.android.R
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.label
import com.intrada.android.ui.components.scaled
import com.intrada.shared.BuildingSetlistView
import com.intrada.shared.ItemKind
import com.intrada.shared.SessionEvent

private const val LIFT_ELEVATION = 8f

internal class DragState {
    var row by mutableStateOf<String?>(null)
    var offset by mutableFloatStateOf(0f)
    val heights = mutableStateMapOf<String, Int>()

    fun reset() {
        row = null
        offset = 0f
    }
}

@Composable
internal fun BuilderList(
    setlist: BuildingSetlistView,
    state: BuilderScreenState,
    actions: BuilderActions,
    editing: Boolean,
) {
    val rows = BuilderRow.rows(setlist.blocks, state.collapsed, editing)
    val drag = remember { DragState() }
    val context = RowContext(setlist, state, actions, editing)
    Column {
        rows.forEachIndexed { index, row ->
            val lifted = row.id == drag.row
            Box(
                Modifier.onSizeChanged { drag.heights[row.id] = it.height }
                    .zIndex(if (lifted) 1f else 0f)
                    .graphicsLayer {
                        translationY = if (lifted) drag.offset else 0f
                        shadowElevation = if (lifted) LIFT_ELEVATION.dp.toPx() else 0f
                    }
                    .then(
                        if (editing || row is BuilderRow.AddRelated) Modifier
                        else Modifier.liftable(row, rows, drag, actions)
                    )
                    .padding(
                        top = if (row.startsUnit && index > 0) IntradaSpacing.controlGap else 0.dp
                    )
            ) {
                when (row) {
                    is BuilderRow.Standalone -> StandaloneRow(row, context)
                    is BuilderRow.Header -> HeaderRow(row, context)
                    is BuilderRow.Nested -> NestedRow(row, context)
                    is BuilderRow.AddRelated -> AddRelatedRow(row, actions)
                }
            }
        }
    }
}

private fun Modifier.liftable(
    row: BuilderRow,
    rows: List<BuilderRow>,
    drag: DragState,
    actions: BuilderActions,
): Modifier =
    pointerInput(row.id, rows) {
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
                BuilderRow.drop(rows, from, destination)?.let(actions.send)
                drag.reset()
            },
            onDragCancel = { drag.reset() },
        )
    }

@Composable
private fun StandaloneRow(row: BuilderRow.Standalone, context: RowContext) {
    val entry = row.entry
    val index = context.unitIndex(row.block)
    val canUp = index > 0
    val canDown = index < context.setlist.blocks.lastIndex
    val settings = { context.actions.navigation.onEntry(entry.id) }
    SegmentRow(SegmentPosition.SINGLE) {
        RowLeading(context.editing, "Remove ${entry.itemTitle}") { context.removeUnit(row.block) }
        RowBody(
            RowSpec(
                RowCopy(
                    entry.itemTitle,
                    entry.itemType.label + durationSuffix(row.block.durationDisplay),
                    entry.itemType,
                ),
                entry,
                "builder.row",
                tap = ("settings" to settings).takeIf { !context.editing },
                move =
                    MoveActions(
                        up = { context.moveUnit(row.block, -1) }.takeIf { canUp },
                        down = { context.moveUnit(row.block, 1) }.takeIf { canDown },
                    ),
            ),
            context,
            Modifier.weight(1f),
        )
        if (context.editing) {
            MoveButtons(entry.itemTitle, canUp, canDown) { context.moveUnit(row.block, it) }
        } else {
            IconAction(
                R.drawable.ic_close,
                "Remove ${entry.itemTitle}",
                { context.removeUnit(row.block) },
                Modifier.testTag("builder.remove"),
                tint = IntradaColor.inkFaintIcon,
            )
        }
    }
}

@Composable
private fun HeaderRow(row: BuilderRow.Header, context: RowContext) {
    val block = row.block
    val state = context.state
    val title = block.pieceTitle ?: "Related exercises"
    val index = context.unitIndex(block)
    val canUp = index > 0
    val canDown = index < context.setlist.blocks.lastIndex
    val toggle = {
        state.collapsed =
            if (row.collapsed) state.collapsed - row.groupId else state.collapsed + row.groupId
    }
    SegmentRow(row.position, sunken = !row.collapsed) {
        RowLeading(context.editing, "Remove $title") { context.removeUnit(block) }
        RowBody(
            RowSpec(
                RowCopy(
                    title,
                    headerSubtitle(block, row.collapsed) ?: relatedLabel(block),
                    ItemKind.PIECE,
                ),
                block.piece,
                "builder.header",
                tap = (if (row.collapsed) "expand" else "collapse") to toggle,
                move =
                    MoveActions(
                        up = { context.moveUnit(block, -1) }.takeIf { canUp },
                        down = { context.moveUnit(block, 1) }.takeIf { canDown },
                    ),
            ),
            context,
            Modifier.weight(1f),
        )
        if (context.editing) {
            MoveButtons(title, canUp, canDown) { context.moveUnit(block, it) }
        } else {
            IconAction(
                R.drawable.ic_ellipsis,
                "Actions for $title",
                { state.menuFor = block },
                Modifier.testTag("builder.blockMenu"),
            )
            Image(
                painterResource(
                    if (row.collapsed) R.drawable.ic_chevron_down else R.drawable.ic_chevron_up
                ),
                contentDescription = null,
                modifier = Modifier.size(IntradaIconSize.inline.scaled()),
                colorFilter = ColorFilter.tint(IntradaColor.inkFaintIcon),
            )
        }
    }
}

@Composable
private fun NestedRow(row: BuilderRow.Nested, context: RowContext) {
    val entry = row.entry
    val settings = { context.actions.navigation.onEntry(entry.id) }
    val move = { to: Int -> context.moveRelated(entry, row.block, to) }
    val canUp = row.localIndex > 0
    val canDown = row.localIndex < row.block.related.lastIndex
    Column(Modifier.fillMaxWidth()) {
        HairlineDivider(Modifier.padding(start = IntradaSpacing.card))
        SegmentRow(row.position) {
            RowLeading(context.editing, "Remove ${entry.itemTitle}") {
                context.send(SessionEvent.RemoveFromSetlist(entry.id))
            }
            RowBody(
                RowSpec(
                    RowCopy(
                        entry.itemTitle,
                        "Related" + durationSuffix(entry.durationDisplay),
                        ItemKind.EXERCISE,
                        nested = true,
                    ),
                    entry,
                    "builder.row",
                    tap = ("settings" to settings).takeIf { !context.editing },
                    move =
                        MoveActions(
                            up = { move(row.localIndex - 1) }.takeIf { canUp },
                            down = { move(row.localIndex + 1) }.takeIf { canDown },
                        ),
                ),
                context,
                Modifier.weight(1f),
            )
            if (context.editing) {
                MoveButtons(entry.itemTitle, canUp, canDown) { move(row.localIndex + it) }
            }
        }
    }
}

@Composable
private fun AddRelatedRow(row: BuilderRow.AddRelated, actions: BuilderActions) {
    Column(Modifier.fillMaxWidth()) {
        HairlineDivider(Modifier.padding(start = IntradaSpacing.card))
        SegmentRow(SegmentPosition.BOTTOM) {
            AddRow(
                "Add a related exercise",
                "Add a related exercise to ${row.block.pieceTitle ?: "this block"}",
                "builder.addRelated",
                { actions.navigation.onAddExercise(row.groupId) },
                Modifier.weight(1f),
            )
        }
    }
}
