package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.unit.dp
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.TagChip
import com.intrada.android.ui.components.TagChipStyle
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.LibraryItemView
import com.intrada.shared.SectionView

class PieceNavigation(
    val onEdit: () -> Unit,
    val onSection: (String?) -> Unit,
    val onAddExercises: () -> Unit,
    val onChooseSections: (String) -> Unit,
    val onOpenExercise: (String) -> Unit,
    val onClosed: () -> Unit,
)

class PieceScreenState(reordering: List<SectionView>? = null, editingLinks: Boolean = false) {
    var reordering by mutableStateOf(reordering)
    var editingLinks by mutableStateOf(editingLinks)
    var confirmingDelete by mutableStateOf(false)
}

class PieceActions(
    val navigation: PieceNavigation,
    val send: (Event) -> Boolean,
    val onDismissError: () -> Unit,
)

@Composable
fun PieceRoute(
    store: Store,
    id: String,
    navigation: PieceNavigation,
    modifier: Modifier = Modifier,
) {
    val rows by store.libraryRows.collectAsState()
    val viewModel by store.viewModel.collectAsState()
    val halted by store.halted.collectAsState()
    var closing by remember { mutableStateOf(false) }
    val found = rows.firstOrNull { it.id == id && it.itemType == ItemKind.PIECE }
    val item = rememberLastFound(found, closing)
    if (item == null) {
        MissingItem("Piece", NO_LONGER_THERE, modifier)
        return
    }
    val state = remember(id) { PieceScreenState() }
    val closingNavigation =
        PieceNavigation(
            navigation.onEdit,
            navigation.onSection,
            navigation.onAddExercises,
            navigation.onChooseSections,
            navigation.onOpenExercise,
            onClosed = {
                closing = true
                navigation.onClosed()
            },
        )
    PieceScreen(
        item,
        state,
        PieceActions(
            closingNavigation,
            send = { store.sendAccepted(it) },
            onDismissError = { store.send(Event.ClearError) },
        ),
        modifier,
        error = viewModel?.error,
        halted = halted,
    )
}

internal const val NO_LONGER_THERE = "This item is no longer in your library."

private class LastFound<T : Any> {
    var value: T? = null
}

@Composable
internal fun <T : Any> rememberLastFound(found: T?, closing: Boolean): T? {
    val last = remember { LastFound<T>() }
    if (found != null) last.value = found
    return if (closing) last.value else found
}

@Composable
fun PieceScreen(
    item: LibraryItemView,
    state: PieceScreenState,
    actions: PieceActions,
    modifier: Modifier = Modifier,
    error: String? = null,
    halted: Boolean = false,
) {
    ScreenScaffold(
        item.title,
        modifier,
        actions = { TextAction("Edit", "piece.edit", actions.navigation.onEdit) },
    ) {
        Column(Modifier.fillMaxSize()) {
            if (halted) GlobalBanner(Store.HALTED_MESSAGE, tag = "banner.halted")
            if (error != null)
                GlobalBanner(error, tag = "banner.error", onDismiss = actions.onDismissError)
            Column(
                Modifier.verticalScroll(rememberScrollState()).padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
            ) {
                if (item.subtitle.isNotEmpty()) {
                    BasicText(
                        item.subtitle,
                        style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
                    )
                }
                BadgeRow(item)
                item.notes?.takeIf { it.isNotEmpty() }?.let { NotesCard(it) }
                SectionsCard(item, state, actions)
                DetailRows(item)
                RelatedExercisesCard(item, state, actions)
                DeleteButton(
                    "Delete ${item.itemType.label.lowercase()}",
                    "piece.delete",
                    onClick = { state.confirmingDelete = true },
                    Modifier.padding(top = IntradaSpacing.controlGap),
                )
            }
        }
    }
    if (state.confirmingDelete) {
        ConfirmDialog(
            ConfirmCopy("Delete ${item.title}?", "This can't be undone.", "Delete", "piece.delete"),
            onConfirm = {
                state.confirmingDelete = false
                if (actions.send(Event.Item(ItemEvent.Delete(item.id)))) {
                    actions.navigation.onClosed()
                }
            },
            onDismiss = { state.confirmingDelete = false },
        )
    }
}

// ── Header ──

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun BadgeRow(item: LibraryItemView) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        BasicText(
            item.itemType.label,
            Modifier.background(item.itemType.badgeFill, CircleShape)
                .padding(horizontal = 10.dp, vertical = 5.dp),
            style = IntradaFont.smallMedium.copy(color = IntradaColor.ink),
        )
        item.tags.forEach { TagChip(it, style = TagChipStyle.Outlined) }
    }
}

private val ItemKind.badgeFill
    get() =
        when (this) {
            ItemKind.PIECE -> IntradaColor.pieceBadgeBg
            ItemKind.EXERCISE -> IntradaColor.exerciseBadgeBg
        }

@Composable
private fun NotesCard(notes: String) {
    Column(
        Modifier.fillMaxWidth().cardSurface().padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        FieldLabel("Notes")
        BasicText(notes, style = IntradaFont.body.copy(color = IntradaColor.inkSecondary))
    }
}

@Composable
private fun DetailRows(item: LibraryItemView) {
    val rows =
        listOfNotNull(item.keyLabel?.let { "Key" to it }, item.tempoLine?.let { "Tempo" to it })
    if (rows.isEmpty()) return
    Column(Modifier.fillMaxWidth().cardSurface()) {
        rows.forEachIndexed { index, (label, value) ->
            if (index > 0) HairlineDivider()
            Row(
                Modifier.fillMaxWidth()
                    .clearAndSetSemantics { contentDescription = "$label, $value" }
                    .padding(
                        horizontal = IntradaSpacing.card,
                        vertical = IntradaSpacing.cardCompact,
                    ),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                FieldLabel(label, Modifier.weight(1f))
                BasicText(value, style = IntradaFont.body.copy(color = IntradaColor.ink))
            }
        }
    }
}
