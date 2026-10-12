package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.core.Store
import com.intrada.android.core.withIds
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.Event
import com.intrada.shared.ItemKind
import com.intrada.shared.LibraryItemView
import com.intrada.shared.ListQuery
import com.intrada.shared.SessionEvent

class PickerCopy(val title: String, val tag: String, val note: String?, val empty: String)

/** Each tick adds or removes at once, so Done only closes. */
class PickerActions(val onDone: () -> Unit, val onToggle: (LibraryItemView) -> Unit)

class PickerSearch(val text: String, val onChange: (String) -> Unit)

/** The core narrows the rows; the add screen only says what was typed. */
class PickerRows(
    val items: List<LibraryItemView>,
    val recent: List<LibraryItemView> = emptyList(),
    val search: PickerSearch? = null,
)

@Composable
fun AddToSessionRoute(store: Store, onDone: () -> Unit, modifier: Modifier = Modifier) {
    val viewModel by store.viewModel.collectAsState()
    val rows by store.libraryRows.collectAsState()
    var text by rememberSaveable { mutableStateOf("") }
    LibraryQueryScope(store, text)
    val entries = viewModel?.buildingSetlist?.entries.orEmpty()
    val entryByItem = entries.associateBy({ it.itemId }, { it.id })
    val filtered = viewModel?.activeQuery != null
    LibraryPickerScreen(
        PickerCopy(
            "Add to session",
            "addToSession",
            "Pieces bring their related exercises as a group.",
            "The library is empty · add pieces and exercises first.",
        ),
        PickerRows(
            rows.withIds(viewModel?.visibleIds.orEmpty()),
            recent =
                if (filtered) emptyList()
                else rows.withIds(viewModel?.recentlyPractisedIds.orEmpty()),
            search =
                PickerSearch(text) {
                    text = it
                    store.send(Event.SetQuery(searchQuery(it)))
                },
        ),
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
        alerts = storeAlerts(store),
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
        PickerRows(rows.filter { it.itemType == ItemKind.EXERCISE && it.id !in taken }),
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
        alerts = storeAlerts(store),
    )
}

@Composable
fun LibraryPickerScreen(
    copy: PickerCopy,
    rows: PickerRows,
    added: Set<String>,
    actions: PickerActions,
    modifier: Modifier = Modifier,
    alerts: ScreenAlerts = ScreenAlerts(),
) {
    val tag = copy.tag
    ScreenScaffold(
        copy.title,
        modifier,
        actions = { TextAction("Done", "$tag.done", actions.onDone, emphasised = true) },
    ) {
        Column(Modifier.fillMaxSize()) {
            AlertBanners(alerts)
            rows.search?.let { SearchField(it, "${copy.tag}.search") }
            PickerList(copy, rows, added, actions)
        }
    }
}

@Composable
private fun ColumnScope.PickerList(
    copy: PickerCopy,
    rows: PickerRows,
    added: Set<String>,
    actions: PickerActions,
) {
    val items = rows.items
    val typed = rows.search?.text?.trim().orEmpty()
    Column(
        Modifier.weight(1f)
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        if (items.isEmpty()) {
            BasicText(
                if (typed.isEmpty()) copy.empty else "No items match \u201c$typed\u201d.",
                style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
            )
        } else {
            if (rows.recent.isNotEmpty()) {
                SectionHeader("Recently practised")
                PickerCard(rows.recent, added, copy.tag, actions)
                SectionHeader("Library")
            }
            copy.note?.let {
                BasicText(
                    it,
                    style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                )
            }
            PickerCard(items, added, copy.tag, actions)
        }
    }
}

@Composable
private fun PickerCard(
    items: List<LibraryItemView>,
    added: Set<String>,
    tag: String,
    actions: PickerActions,
) {
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

@Composable
private fun SearchField(search: PickerSearch, tag: String) {
    Row(
        Modifier.fillMaxWidth()
            .padding(horizontal = IntradaSpacing.card)
            .padding(top = IntradaSpacing.cardCompact)
            .heightIn(min = 48.dp)
            .background(IntradaColor.cardFill, CircleShape)
            .border(1.dp, IntradaColor.hairline, CircleShape)
            .padding(start = IntradaSpacing.card),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicTextField(
            search.text,
            search.onChange,
            Modifier.weight(1f).semantics { contentDescription = "Search library" }.testTag(tag),
            textStyle = IntradaFont.body.copy(color = IntradaColor.ink),
            singleLine = true,
            keyboardOptions =
                KeyboardOptions(autoCorrectEnabled = false, imeAction = ImeAction.Search),
            cursorBrush = SolidColor(IntradaColor.ink),
            decorationBox = { field ->
                Box {
                    if (search.text.isEmpty()) {
                        BasicText(
                            "Search library",
                            style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
                        )
                    }
                    field()
                }
            },
        )
        if (search.text.isNotEmpty()) {
            IconAction(
                R.drawable.ic_close,
                "Clear search",
                { search.onChange("") },
                tint = IntradaColor.inkFaintIcon,
            )
        }
    }
}

/**
 * The add screen starts from its own search, so a restored field still narrows the list, and gives
 * the library its own query back (#1440).
 */
@Composable
private fun LibraryQueryScope(store: Store, restored: String) {
    DisposableEffect(store) {
        val before = store.viewModel.value?.activeQuery
        store.send(Event.SetQuery(searchQuery(restored)))
        onDispose { store.send(Event.SetQuery(before)) }
    }
}

private fun searchQuery(text: String): ListQuery? =
    text.trim().takeIf { it.isNotEmpty() }?.let { ListQuery(it, null, null, emptyList(), false) }
