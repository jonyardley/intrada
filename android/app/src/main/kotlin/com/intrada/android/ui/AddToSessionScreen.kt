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
import androidx.compose.ui.Modifier
import com.intrada.android.core.Store
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.Event
import com.intrada.shared.ItemKind
import com.intrada.shared.LibraryItemView
import com.intrada.shared.SessionEvent

class PickerCopy(val title: String, val tag: String, val note: String?, val empty: String)

/** Each tick adds or removes at once, so Done only closes. */
class PickerActions(val onDone: () -> Unit, val onToggle: (LibraryItemView) -> Unit)

@Composable
fun AddToSessionRoute(store: Store, onDone: () -> Unit, modifier: Modifier = Modifier) {
    val viewModel by store.viewModel.collectAsState()
    val rows by store.libraryRows.collectAsState()
    val entries = viewModel?.buildingSetlist?.entries.orEmpty()
    val entryByItem = entries.associateBy({ it.itemId }, { it.id })
    LibraryPickerScreen(
        PickerCopy(
            "Add to session",
            "addToSession",
            "Pieces bring their related exercises as a group.",
            "The library is empty · add pieces and exercises first.",
        ),
        rows,
        added = entryByItem.keys,
        PickerActions(onDone) { item ->
            val entryId = entryByItem[item.id]
            store.sendAccepted(
                Event.Session(
                    if (entryId != null) SessionEvent.RemoveFromSetlist(entryId)
                    else SessionEvent.AddToSetlist(item.id)
                )
            )
        },
        modifier = modifier,
    )
}

@Composable
fun AddRelatedRoute(
    store: Store,
    groupId: String,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val viewModel by store.viewModel.collectAsState()
    val rows by store.libraryRows.collectAsState()
    val block = viewModel?.buildingSetlist?.blocks?.firstOrNull { it.groupId == groupId }
    val taken = block?.takenElsewhere.orEmpty().toSet()
    val entryByItem = block?.entries.orEmpty().associateBy({ it.itemId }, { it.id })
    LibraryPickerScreen(
        PickerCopy("Add a related exercise", "addRelated", null, "No other exercises left to add."),
        rows.filter { it.itemType == ItemKind.EXERCISE && it.id !in taken },
        added = entryByItem.keys,
        PickerActions(onDone) { item ->
            val entryId = entryByItem[item.id]
            store.sendAccepted(
                Event.Session(
                    if (entryId != null) SessionEvent.RemoveFromSetlist(entryId)
                    else SessionEvent.AddExerciseToBlock(groupId, item.id)
                )
            )
        },
        modifier = modifier,
    )
}

@Composable
fun LibraryPickerScreen(
    copy: PickerCopy,
    items: List<LibraryItemView>,
    added: Set<String>,
    actions: PickerActions,
    modifier: Modifier = Modifier,
) {
    val tag = copy.tag
    ScreenScaffold(
        copy.title,
        modifier,
        actions = { TextAction("Done", "$tag.done", actions.onDone, emphasised = true) },
    ) {
        Column(
            Modifier.fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        ) {
            if (items.isEmpty()) {
                BasicText(
                    copy.empty,
                    style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
                )
            } else {
                copy.note?.let {
                    BasicText(
                        it,
                        style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                    )
                }
                Column(Modifier.cardSurface()) {
                    items.forEachIndexed { index, item ->
                        if (index > 0) HairlineDivider()
                        TickRow(
                            item.title,
                            listOf(item.itemType.label, item.subtitle)
                                .filter { it.isNotEmpty() }
                                .joinToString(" · "),
                            chosen = item.id in added,
                            tag = "$tag.row",
                            onToggle = { actions.onToggle(item) },
                        )
                    }
                }
            }
        }
    }
}
