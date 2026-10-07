package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.TagChip
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.ffi.itemFormCanSave
import com.intrada.shared.CreateItem
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.KeyEdit
import com.intrada.shared.LibraryItemView
import com.intrada.shared.TempoInput
import com.intrada.shared.UpdateItem

class VariationRow(val variantId: String?, val label: String)

enum class ItemFormMode(val confirmLabel: String) {
    ADD("Add"),
    EDIT("Save");

    fun title(kind: ItemKind) = if (this == ADD) "New ${kind.label}" else "Edit"
}

private class ChipListLabels(val label: String, val addLabel: String, val tag: String)

class ItemFormState(kind: ItemKind = ItemKind.PIECE) {
    var kind by mutableStateOf(kind)
    var title by mutableStateOf("")
    var composer by mutableStateOf("")
    var marking by mutableStateOf("")
    var bpm by mutableStateOf("")
    var notes by mutableStateOf("")
    val tags = mutableStateListOf<String>()
    val variations = mutableStateListOf<VariationRow>()
    var formError by mutableStateOf<String?>(null)

    val canSave: Boolean
        get() = itemFormCanSave(title)

    fun addEvent(): Event =
        Event.Item(
            ItemEvent.Add(
                CreateItem(
                    title = title,
                    kind = kind,
                    composer = composer,
                    tempo = TempoInput(marking, bpm),
                    notes = notes,
                    tags = tags.toList(),
                    variationLabels = variations.map { it.label },
                )
            )
        )

    fun editEvent(id: String): Event =
        Event.Item(
            ItemEvent.Edit(
                id,
                UpdateItem(
                    title = title,
                    kind = kind,
                    composer = composer,
                    key = KeyEdit.Keep,
                    tempo = TempoInput(marking, bpm),
                    notes = notes,
                    tags = tags.toList(),
                ),
                variations.mapNotNull { it.variantId },
                variations.filter { it.variantId == null }.map { it.label },
            )
        )

    companion object {
        fun of(item: LibraryItemView) =
            ItemFormState(item.itemType).apply {
                title = item.title
                composer = item.subtitle
                marking = item.tempoMarking.orEmpty()
                bpm = item.tempoBpm?.toString().orEmpty()
                notes = item.notes.orEmpty()
                tags.addAll(item.tags)
                variations.addAll(item.variations.map { VariationRow(it.id, it.label) })
            }
    }
}

// A refusal comes back for the form to show inline and leaves the core, so the app banner does not
// repeat it; nothing closes until the core accepts (#1595).
fun Store.sendFromForm(event: Event): String? {
    val accepted = sendAccepted(event)
    val error = viewModel.value?.error ?: if (accepted) null else SAVE_FAILED
    if (error != null) send(Event.ClearError)
    return error
}

private const val SAVE_FAILED = "Couldn't save. Try again."

@Composable
fun LibraryAddRoute(store: Store, onDone: () -> Unit, modifier: Modifier = Modifier) {
    val form = remember { ItemFormState() }
    var closing by remember { mutableStateOf(false) }
    ItemFormScreen(
        form,
        ItemFormMode.ADD,
        onCancel = onDone,
        onConfirm = {
            if (!closing) {
                form.formError = store.sendFromForm(form.addEvent())
                closing = form.formError == null
                if (closing) onDone()
            }
        },
        modifier = modifier,
    )
}

@Composable
fun LibraryEditRoute(store: Store, id: String, onDone: () -> Unit, modifier: Modifier = Modifier) {
    val rows by store.libraryRows.collectAsState()
    val item = rows.firstOrNull { it.id == id }
    if (item == null) {
        MissingItem("Edit", NO_LONGER_THERE, modifier)
        return
    }
    val form = remember(id) { ItemFormState.of(item) }
    var closing by remember { mutableStateOf(false) }
    ItemFormScreen(
        form,
        ItemFormMode.EDIT,
        onCancel = onDone,
        onConfirm = {
            if (!closing) {
                form.formError = store.sendFromForm(form.editEvent(id))
                closing = form.formError == null
                if (closing) onDone()
            }
        },
        modifier = modifier,
    )
}

@Composable
fun ItemFormScreen(
    form: ItemFormState,
    mode: ItemFormMode,
    onCancel: () -> Unit,
    onConfirm: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ScreenScaffold(
        mode.title(form.kind),
        modifier,
        actions = {
            TextAction("Cancel", "itemForm.cancel", onCancel)
            TextAction(
                mode.confirmLabel,
                "itemForm.confirm",
                onConfirm,
                emphasised = true,
                enabled = form.canSave,
            )
        },
    ) {
        Column(Modifier.fillMaxSize()) {
            val formError = form.formError
            if (formError != null) {
                FormErrorBanner(
                    formError,
                    Modifier.padding(horizontal = IntradaSpacing.card)
                        .padding(top = IntradaSpacing.cardCompact)
                        .testTag("itemForm.error"),
                )
            }
            ItemFormFields(form, Modifier.verticalScroll(rememberScrollState()))
        }
    }
}

@Composable
private fun ItemFormFields(form: ItemFormState, modifier: Modifier = Modifier) {
    Column(
        modifier.padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
    ) {
        KindSegment(form.kind, { form.kind = it })
        Column(Modifier.cardSurface()) {
            FormField(
                "Title",
                form.title,
                { form.title = it },
                "itemForm.title",
                placeholder = "Required",
            )
            HairlineDivider()
            FormField("Composer", form.composer, { form.composer = it }, "itemForm.composer")
        }
        if (form.kind == ItemKind.EXERCISE) {
            ChipListCard(
                ChipListLabels("Variations", "Add a variation", "itemForm.variation"),
                form.variations.map { it.label },
                onRemove = { form.variations.removeAt(it) },
                onAdd = { form.variations.add(VariationRow(null, it)) },
            )
        }
        Column(Modifier.cardSurface()) {
            FormField(
                "Tempo marking",
                form.marking,
                { form.marking = it },
                "itemForm.marking",
                placeholder = "e.g. Allegro",
            )
            HairlineDivider()
            FormField(
                "Beats per minute",
                form.bpm,
                { form.bpm = it },
                "itemForm.bpm",
                keyboard = KeyboardType.Number,
            )
        }
        Column(Modifier.cardSurface()) {
            FormField(
                "Notes",
                form.notes,
                { form.notes = it },
                "itemForm.notes",
                singleLine = false,
            )
        }
        ChipListCard(
            ChipListLabels("Tags", "Add a tag", "itemForm.tag"),
            form.tags,
            onRemove = { form.tags.removeAt(it) },
            onAdd = { form.tags.add(it) },
        )
    }
}

@Composable
private fun KindSegment(
    selection: ItemKind,
    onSelect: (ItemKind) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
        SegmentedPills(
            ItemKind.entries,
            selection,
            onSelect,
            label = { it.label },
            tag = { "itemForm.kind.${it.name.lowercase()}" },
        )
        BasicText(
            selection.caption,
            Modifier.fillMaxWidth(),
            style =
                IntradaFont.secondary.copy(
                    color = IntradaColor.inkSecondary,
                    textAlign = TextAlign.Center,
                ),
        )
    }
}

private val ItemKind.caption: String
    get() =
        when (this) {
            ItemKind.PIECE -> "Repertoire to learn and keep up"
            ItemKind.EXERCISE -> "Drills and studies to build technique"
        }

@Composable
internal fun FormField(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    tag: String,
    modifier: Modifier = Modifier,
    placeholder: String = "",
    keyboard: KeyboardType = KeyboardType.Text,
    singleLine: Boolean = true,
    note: String? = null,
) {
    Column(
        modifier
            .fillMaxWidth()
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        if (label.isNotEmpty()) FieldLabel(label)
        BasicTextField(
            value,
            onValueChange,
            Modifier.fillMaxWidth()
                .semantics { contentDescription = label.ifEmpty { placeholder } }
                .testTag(tag),
            textStyle = IntradaFont.body.copy(color = IntradaColor.ink),
            singleLine = singleLine,
            minLines = if (singleLine) 1 else 3,
            keyboardOptions = KeyboardOptions(keyboardType = keyboard),
            cursorBrush = SolidColor(IntradaColor.ink),
            decorationBox = { field ->
                Box {
                    if (value.isEmpty() && placeholder.isNotEmpty()) {
                        BasicText(
                            placeholder,
                            style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
                        )
                    }
                    field()
                }
            },
        )
        if (note != null) {
            BasicText(note, style = IntradaFont.small.copy(color = IntradaColor.inkSecondary))
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun ChipListCard(
    labels: ChipListLabels,
    chips: List<String>,
    onRemove: (Int) -> Unit,
    onAdd: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val tag = labels.tag
    val addLabel = labels.addLabel
    var typed by remember { mutableStateOf("") }
    Column(modifier.cardSurface()) {
        FieldLabel(
            labels.label,
            Modifier.padding(horizontal = IntradaSpacing.card)
                .padding(top = IntradaSpacing.cardCompact),
        )
        if (chips.isNotEmpty()) {
            FlowRow(
                Modifier.fillMaxWidth().padding(horizontal = IntradaSpacing.card),
                horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
            ) {
                chips.forEachIndexed { index, chip ->
                    TagChip(chip, Modifier.testTag("$tag.chip"), onRemove = { onRemove(index) })
                }
            }
        }
        Row(verticalAlignment = Alignment.Bottom) {
            Box(Modifier.weight(1f)) {
                FormField("", typed, { typed = it }, "$tag.input", placeholder = addLabel)
            }
            TextAction(
                "Add",
                "$tag.add",
                onClick = {
                    if (typed.isNotBlank()) {
                        onAdd(typed)
                        typed = ""
                    }
                },
                Modifier.padding(end = IntradaSpacing.controlGap).semantics {
                    contentDescription = addLabel
                },
            )
        }
    }
}
