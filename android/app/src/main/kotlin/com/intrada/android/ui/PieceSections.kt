package com.intrada.android.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
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
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.TagChip
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.LibraryItemView
import com.intrada.shared.SectionChange
import com.intrada.shared.SectionKind
import com.intrada.shared.SectionView

// ── Sections ──

@Composable
internal fun SectionsCard(item: LibraryItemView, state: PieceScreenState, actions: PieceActions) {
    val reordering = state.reordering
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
        SectionHeader("Sections", action = sectionsAction(item, state, actions))
        when {
            reordering != null ->
                Column(Modifier.cardSurface()) {
                    reordering.forEachIndexed { index, section ->
                        if (index > 0) HairlineDivider()
                        ReorderRow(section, index, reordering.size, state)
                    }
                }
            item.sections.isEmpty() ->
                AddRow(
                    "Add sections",
                    "Add sections to this ${item.itemType.label.lowercase()}",
                    "sections.add",
                    { actions.navigation.onSection(null) },
                    Modifier.cardSurface(),
                    hint = "A1, B, Coda, or bars 12 to 14",
                )
            else ->
                Column(Modifier.cardSurface()) {
                    item.sections.forEach { section ->
                        SectionRow(section) { actions.navigation.onSection(section.id) }
                        HairlineDivider()
                    }
                    AddRow(
                        "Add section",
                        "Add a section",
                        "sections.add",
                        { actions.navigation.onSection(null) },
                    )
                }
        }
    }
}

private fun sectionsAction(
    item: LibraryItemView,
    state: PieceScreenState,
    actions: PieceActions,
): HeaderAction? {
    val reordering = state.reordering
    return when {
        reordering != null ->
            HeaderAction("Done", "Done reordering sections", "sections.header") {
                finishReordering(item, reordering, state, actions)
            }
        item.sections.isEmpty() -> null
        else ->
            HeaderAction("Reorder", "Reorder or remove sections", "sections.header") {
                state.reordering = item.sections
            }
    }
}

// Removals and the new order land as one write, and the list stays in reorder until the core
// accepts it, so a refusal loses nothing (#2447).
private fun finishReordering(
    item: LibraryItemView,
    rows: List<SectionView>,
    state: PieceScreenState,
    actions: PieceActions,
) {
    val ids = rows.map { it.id }
    if (
        ids == item.sections.map { it.id } ||
            actions.send(Event.Item(ItemEvent.ChangeSection(item.id, SectionChange.Arrange(ids))))
    ) {
        state.reordering = null
    }
}

@Composable
private fun SectionRow(section: SectionView, onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth()
            .heightIn(min = 48.dp)
            .clickable(onClickLabel = "edit", role = Role.Button, onClick = onClick)
            .clearAndSetSemantics {
                contentDescription = section.spoken
                role = Role.Button
                testTag = "sections.row"
                onClick(label = "edit") {
                    onClick()
                    true
                }
            }
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SectionTitleStack(section, Modifier.weight(1f))
        section.targetBpm?.let {
            BasicText("♩ $it", style = IntradaFont.figure.copy(color = IntradaColor.inkSecondary))
        }
        Chevron()
    }
}

@Composable
private fun ReorderRow(section: SectionView, index: Int, count: Int, state: PieceScreenState) {
    Row(
        Modifier.fillMaxWidth().padding(horizontal = IntradaSpacing.controlGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconAction(
            R.drawable.ic_minus_circle,
            "Remove ${section.label}",
            onClick = { state.reordering = state.reordering?.filterNot { it.id == section.id } },
            tint = IntradaColor.danger,
        )
        SectionTitleStack(
            section,
            Modifier.weight(1f).padding(vertical = IntradaSpacing.cardCompact),
        )
        IconAction(
            R.drawable.ic_chevron_up,
            "Move ${section.label} up",
            onClick = { state.moveSection(index, -1) },
            enabled = index > 0,
        )
        IconAction(
            R.drawable.ic_chevron_down,
            "Move ${section.label} down",
            onClick = { state.moveSection(index, 1) },
            enabled = index < count - 1,
        )
    }
}

private fun PieceScreenState.moveSection(from: Int, step: Int) {
    val rows = reordering?.toMutableList() ?: return
    val to = from + step
    if (to !in rows.indices) return
    rows.add(to, rows.removeAt(from))
    reordering = rows
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun SectionTitleStack(section: SectionView, modifier: Modifier = Modifier) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(3.dp)) {
        BasicText(section.label, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
        val tricky = section.kind == SectionKind.TROUBLESPOT
        if (section.barsCaption != null || tricky || section.isWeakest) {
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
                verticalArrangement = Arrangement.spacedBy(3.dp),
            ) {
                section.barsCaption?.let {
                    BasicText(
                        it,
                        style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                    )
                }
                if (tricky) TagChip(TRICKY_SPOT)
                if (section.isWeakest) TagChip(WEAKEST)
            }
        }
    }
}

internal const val TRICKY_SPOT = "Tricky spot"
private const val WEAKEST = "Weakest"

private val SectionView.spoken: String
    get() =
        listOfNotNull(
                label,
                barsCaption?.lowercase(),
                TRICKY_SPOT.lowercase().takeIf { kind == SectionKind.TROUBLESPOT },
                targetBpm?.let { "$it beats per minute" },
                caption,
                WEAKEST.lowercase().takeIf { isWeakest },
            )
            .joinToString(", ")
