package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.toMutableStateList
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.LibraryItemView
import com.intrada.shared.VariationEvent
import com.intrada.shared.VariationOptionView
import com.intrada.shared.VariationView

// ── Variations on the piece ──

@Composable
internal fun KeysAndVariations(item: LibraryItemView, state: PieceScreenState, topMark: Int) {
    if (item.keys.isNotEmpty()) KeysCard(item, topMark) { state.choosingKeys = true }
    VariationsCard(item) { state.choosingVariations = true }
    if (item.keys.isEmpty()) OtherKeysRow { state.choosingKeys = true }
}

@Composable
internal fun VariationsCard(item: LibraryItemView, onChoose: () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
        SectionHeader(
            "Variations",
            caption = item.variationsCaption,
            action =
                item.variations
                    .takeIf { it.isNotEmpty() }
                    ?.let {
                        HeaderAction(
                            "Choose",
                            "Choose variations",
                            "variations.header",
                            perform = onChoose,
                        )
                    },
        )
        if (item.variations.isEmpty()) {
            AddRow(
                "Choose variations",
                "Choose variations for this ${item.itemType.label.lowercase()}",
                "variations.choose",
                onChoose,
                Modifier.cardSurface(),
                hint = "Hands separately, dotted rhythms, your own",
            )
        } else {
            Column(Modifier.fillMaxWidth().cardSurface()) {
                item.variations.forEachIndexed { index, variation ->
                    if (index > 0) HairlineDivider()
                    VariationListRow(variation)
                }
            }
        }
    }
}

@Composable
private fun VariationListRow(variation: VariationView) {
    Column(
        Modifier.fillMaxWidth()
            .testTag("variations.row")
            .clearAndSetSemantics {
                contentDescription = "${variation.label}, ${variation.caption}"
            }
            .padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        BasicText(variation.label, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
        BasicText(
            variation.caption,
            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
        )
    }
}

// ── Variations sheet ──

/**
 * What the sheet has ticked before Done: library rows by id in the order ticked, and new labels.
 */
class VariationTicks(ids: List<String>) {
    val ids = ids.toMutableStateList()
    val newLabels = mutableStateListOf<String>()

    fun isChosen(id: String) = id in ids

    fun toggle(id: String) {
        if (!ids.remove(id)) ids.add(id)
    }

    fun toggleNew(label: String) {
        if (!newLabels.remove(label)) newLabels.add(label)
    }

    fun addable(query: String, library: List<VariationOptionView>): String? {
        val text = query.trim()
        if (text.isEmpty()) return null
        val taken = library.map { it.label } + newLabels
        return text.takeIf { typed -> taken.none { it.equals(typed, ignoreCase = true) } }
    }

    companion object {
        fun matches(query: String, library: List<VariationOptionView>): List<VariationOptionView> {
            val text = query.trim()
            if (text.isEmpty()) return emptyList()
            return library.filter { it.label.contains(text, ignoreCase = true) }
        }
    }
}

@Composable
internal fun VariationsSheet(
    item: LibraryItemView,
    library: List<VariationOptionView>,
    sendFromSheet: (Event) -> String?,
    onClose: () -> Unit,
) {
    FullScreenDialog(onClose) { VariationsSheetScreen(item, library, sendFromSheet, onClose) }
}

@Composable
internal fun VariationsSheetScreen(
    item: LibraryItemView,
    library: List<VariationOptionView>,
    sendFromSheet: (Event) -> String?,
    onClose: () -> Unit,
) {
    val choice = remember { VariationTicks(item.variations.map { it.id }) }
    var query by remember { mutableStateOf("") }
    var formError by remember { mutableStateOf<String?>(null) }
    var renaming by remember { mutableStateOf<VariationOptionView?>(null) }
    fun send(event: Event): Boolean {
        formError = sendFromSheet(event)
        return formError == null
    }
    fun save() {
        val unchanged =
            choice.ids.toList() == item.variations.map { it.id } && choice.newLabels.isEmpty()
        if (
            unchanged ||
                send(
                    Event.Item(
                        ItemEvent.UpdateItemVariations(
                            item.id,
                            choice.ids.toList(),
                            choice.newLabels.toList(),
                        )
                    )
                )
        ) {
            onClose()
        }
    }
    ScreenScaffold(
        "Variations",
        actions = {
            TextAction("Cancel", "variationsSheet.cancel", onClose)
            TextAction("Done", "sheet.done", ::save, emphasised = true)
        },
    ) {
        Column(
            Modifier.fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
        ) {
            formError?.let { FormErrorBanner(it, Modifier.testTag("variationsSheet.error")) }
            Box(Modifier.cardSurface()) {
                FormField(
                    "Find or add",
                    query,
                    { query = it },
                    "variationsSheet.find",
                    placeholder = "Left hand, slow practice",
                )
            }
            if (query.isNotBlank()) {
                MatchesCard(query, library, choice) { query = "" }
            }
            Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
                FieldLabel("All variations")
                AllCard(library, choice) { renaming = it }
            }
            BasicText(
                "A new variation joins the list on every piece and exercise.",
                style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
    renaming?.let { variation ->
        RenameDialog(
            variation,
            onRename = { label ->
                renaming = null
                send(Event.Variation(VariationEvent.Rename(variation.id, label)))
            },
            onDismiss = { renaming = null },
        )
    }
}

@Composable
private fun MatchesCard(
    query: String,
    library: List<VariationOptionView>,
    choice: VariationTicks,
    onAdd: () -> Unit,
) {
    val matches = VariationTicks.matches(query, library)
    val addable = choice.addable(query, library)
    Column(Modifier.fillMaxWidth().cardSurface()) {
        matches.forEachIndexed { index, variation ->
            if (index > 0) HairlineDivider()
            TickRow(
                variation.label,
                variation.usage,
                choice.isChosen(variation.id),
                "variationsSheet.row",
                { choice.toggle(variation.id) },
            )
        }
        if (addable != null) {
            if (matches.isNotEmpty()) HairlineDivider()
            AddRow(
                "Add “$addable”",
                "Add $addable as a new variation",
                "variationsSheet.add",
                {
                    choice.toggleNew(addable)
                    onAdd()
                },
            )
        } else if (matches.isEmpty()) {
            BasicText(
                "Already ticked below",
                Modifier.fillMaxWidth().padding(IntradaSpacing.card),
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}

@Composable
private fun AllCard(
    library: List<VariationOptionView>,
    choice: VariationTicks,
    onRename: (VariationOptionView) -> Unit,
) {
    Column(Modifier.fillMaxWidth().cardSurface()) {
        choice.newLabels.forEachIndexed { index, label ->
            if (index > 0) HairlineDivider()
            TickRow(label, null, true, "variationsSheet.row", { choice.toggleNew(label) })
        }
        library.forEachIndexed { index, variation ->
            if (index > 0 || choice.newLabels.isNotEmpty()) HairlineDivider()
            TickRow(
                variation.label,
                variation.usage,
                choice.isChosen(variation.id),
                "variationsSheet.row",
                { choice.toggle(variation.id) },
                longAction = LongAction("Rename") { onRename(variation) },
            )
        }
    }
}

@Composable
private fun RenameDialog(
    variation: VariationOptionView,
    onRename: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    var text by remember { mutableStateOf(variation.label) }
    Dialog(onDismissRequest = onDismiss) {
        Column(
            Modifier.fillMaxWidth().cardSurface().padding(top = IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        ) {
            BasicText(
                "Rename ${variation.label}",
                Modifier.fillMaxWidth().padding(horizontal = IntradaSpacing.card),
                style = IntradaFont.cardTitle.copy(color = IntradaColor.ink),
            )
            BasicText(
                "The new name shows on every piece and exercise that uses it.",
                Modifier.fillMaxWidth().padding(horizontal = IntradaSpacing.card),
                style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
            )
            FormField("Name", text, { text = it }, "variationRename.name")
            HairlineDivider()
            Row(
                Modifier.fillMaxWidth().padding(horizontal = IntradaSpacing.controlGap),
                horizontalArrangement = Arrangement.End,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                TextAction("Cancel", "variationRename.cancel", onDismiss)
                TextAction(
                    "Rename",
                    "variationRename.confirm",
                    { onRename(text) },
                    emphasised = true,
                )
            }
        }
    }
}
