package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.TagChip
import com.intrada.android.ui.components.bar
import com.intrada.android.ui.components.scaled
import com.intrada.shared.BuildingSetlistView
import com.intrada.shared.Event
import com.intrada.shared.ItemKind
import com.intrada.shared.SessionEvent
import com.intrada.shared.SetlistBlockView
import com.intrada.shared.SetlistEntryView

internal class RowContext(
    val setlist: BuildingSetlistView,
    val state: BuilderScreenState,
    val actions: BuilderActions,
    val editing: Boolean,
) {
    fun send(event: SessionEvent) = actions.send(Event.Session(event))

    fun unitIndex(block: SetlistBlockView): Int =
        setlist.blocks.indexOfFirst {
            it.entries.firstOrNull()?.id == block.entries.firstOrNull()?.id
        }

    fun moveUnit(block: SetlistBlockView, delta: Int) {
        val to = unitIndex(block) + delta
        val entryId = block.entries.firstOrNull()?.id ?: return
        if (to in setlist.blocks.indices) send(SessionEvent.MoveUnit(entryId, to.toULong()))
    }

    fun moveRelated(entry: SetlistEntryView, block: SetlistBlockView, to: Int) {
        if (to in block.related.indices) send(SessionEvent.MoveRelated(entry.id, to.toULong()))
    }

    fun removeUnit(block: SetlistBlockView) {
        val groupId = block.groupId
        if (groupId != null) send(SessionEvent.RemoveBlock(groupId))
        else block.entries.firstOrNull()?.let { send(SessionEvent.RemoveFromSetlist(it.id)) }
    }
}

/** What a row says and does; the shared body draws it and gives TalkBack the same. */
internal class RowSpec(
    val copy: RowCopy,
    val plan: SetlistEntryView?,
    val tag: String,
    val tap: Pair<String, () -> Unit>?,
    val move: MoveActions,
)

internal class MoveActions(val up: (() -> Unit)?, val down: (() -> Unit)?)

internal class RowCopy(
    val title: String,
    val caption: String?,
    val kind: ItemKind,
    val nested: Boolean = false,
)

@Composable
internal fun SegmentRow(
    position: SegmentPosition,
    modifier: Modifier = Modifier,
    sunken: Boolean = false,
    content: @Composable RowScope.() -> Unit,
) {
    val r = IntradaRadius.card
    val shape =
        when (position) {
            SegmentPosition.SINGLE -> RoundedCornerShape(r)
            SegmentPosition.TOP -> RoundedCornerShape(topStart = r, topEnd = r)
            SegmentPosition.MIDDLE -> RoundedCornerShape(0.dp)
            SegmentPosition.BOTTOM -> RoundedCornerShape(bottomStart = r, bottomEnd = r)
        }
    Row(
        modifier
            .fillMaxWidth()
            .clip(shape)
            .background(if (sunken) IntradaColor.surfaceSunken else IntradaColor.cardFill)
            .padding(IntradaSpacing.cardCompact),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        content = content,
    )
}

@Composable
internal fun RowLeading(editing: Boolean, removeLabel: String, onRemove: () -> Unit) {
    if (editing) {
        IconAction(
            R.drawable.ic_minus_circle,
            removeLabel,
            onRemove,
            Modifier.testTag("builder.editRemove"),
            tint = IntradaColor.danger,
        )
    } else {
        Image(
            painterResource(R.drawable.ic_grip),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.inline.scaled()),
            colorFilter = ColorFilter.tint(IntradaColor.inkFaintIcon),
        )
    }
}

@Composable
internal fun RowBody(spec: RowSpec, context: RowContext, modifier: Modifier = Modifier) {
    val copy = spec.copy
    val tap = spec.tap
    Row(
        modifier
            .heightIn(min = 48.dp)
            .then(
                if (tap == null) Modifier
                else Modifier.clickable(onClickLabel = tap.first, onClick = tap.second)
            )
            .rowSemantics(spec, context),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        val width = if (copy.nested) 3.dp else 4.dp
        Box(
            Modifier.width(width)
                .height(if (copy.nested) 26.dp else 34.dp)
                .clip(RoundedCornerShape(width))
                .background(copy.kind.bar)
        )
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            val title = if (copy.nested) IntradaFont.bodyMedium else IntradaFont.cardTitle
            BasicText(copy.title, style = title.copy(color = IntradaColor.ink))
            copy.caption?.let {
                BasicText(it, style = IntradaFont.small.copy(color = IntradaColor.inkSecondary))
            }
            spec.plan?.let { PlanLine(it, context) }
        }
    }
}

private fun Modifier.rowSemantics(spec: RowSpec, context: RowContext): Modifier {
    val tap = spec.tap
    val plan = spec.plan
    val offer = plan?.let { p -> context.setlist.lastTimes.firstOrNull { it.entryId == p.id } }
    val spoken =
        (listOfNotNull(spec.copy.title, spec.copy.caption) +
                plan?.let { planTags(it, context.setlist) }.orEmpty())
            .joinToString(", ")
    val action = { label: String, perform: () -> Unit ->
        CustomAccessibilityAction(label) {
            perform()
            true
        }
    }
    return clearAndSetSemantics {
        contentDescription = spoken
        testTag = spec.tag
        if (tap != null) {
            role = Role.Button
            onClick(label = tap.first) {
                tap.second()
                true
            }
        }
        customActions =
            listOfNotNull(
                spec.move.up?.let { action("Move up", it) },
                spec.move.down?.let { action("Move down", it) },
                offer?.let { last ->
                    action(last.label) { context.send(SessionEvent.ApplyLastTime(last.entryId)) }
                },
            )
    }
}

@Composable
internal fun MoveButtons(title: String, canUp: Boolean, canDown: Boolean, onMove: (Int) -> Unit) {
    Row {
        IconAction(R.drawable.ic_chevron_up, "Move $title up", { onMove(-1) }, enabled = canUp)
        IconAction(
            R.drawable.ic_chevron_down,
            "Move $title down",
            { onMove(1) },
            enabled = canDown,
        )
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun PlanLine(entry: SetlistEntryView, context: RowContext) {
    val tags = planTags(entry, context.setlist)
    val offer = context.setlist.lastTimes.firstOrNull { it.entryId == entry.id }
    Column {
        if (tags.isNotEmpty()) {
            FlowRow(
                Modifier.padding(top = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                tags.forEach { TagChip(it) }
            }
        }
        if (offer != null) {
            Box(
                Modifier.heightIn(min = 48.dp)
                    .clickable(role = Role.Button) {
                        context.send(SessionEvent.ApplyLastTime(entry.id))
                    }
                    .testTag("builder.lastTime"),
                contentAlignment = Alignment.CenterStart,
            ) {
                BasicText(
                    "+ ${offer.label}",
                    style = IntradaFont.secondary.copy(color = IntradaColor.accent),
                )
            }
        }
    }
}
