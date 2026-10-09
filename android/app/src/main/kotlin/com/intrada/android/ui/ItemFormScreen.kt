package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.mapSaver
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.style.TextAlign
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.label
import com.intrada.ffi.itemFormCanSave
import com.intrada.shared.CreateItem
import com.intrada.shared.Event
import com.intrada.shared.FormErrorTarget
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.Key
import com.intrada.shared.KeyEdit
import com.intrada.shared.LibraryItemView
import com.intrada.shared.TempoInput
import com.intrada.shared.UpdateItem

enum class ItemFormMode(val confirmLabel: String) {
    ADD("Add"),
    EDIT("Save");

    fun title(kind: ItemKind) = if (this == ADD) "New ${kind.label}" else "Edit"
}

class ItemFormState(kind: ItemKind = ItemKind.PIECE) {
    var kind by mutableStateOf(kind)
    var title by mutableStateOf("")
    var composer by mutableStateOf("")
    var key by mutableStateOf<Key?>(null)
    var marking by mutableStateOf("")
    var bpm by mutableStateOf("")
    var notes by mutableStateOf("")
    val tags = mutableStateListOf<String>()
    val variations = mutableStateListOf<VariationRow>()
    val exercises = mutableStateListOf<StagedExercise>()
    var faultedExercise by mutableStateOf<FormErrorTarget.Exercise?>(null)
    var formError by mutableStateOf<String?>(null)

    val canSave: Boolean
        get() = itemFormCanSave(title)

    // An exercise carries no related exercises, so switching kind drops what was staged.
    fun switchKind(to: ItemKind) {
        kind = to
        if (to != ItemKind.PIECE) chooseExercises(emptyList())
    }

    fun chooseExercises(staged: List<StagedExercise>) {
        exercises.clear()
        exercises.addAll(staged)
        faultedExercise = null
    }

    fun removeExercise(id: String) {
        exercises.removeAll { it.id == id }
        faultedExercise = null
    }

    fun addEvent(): Event {
        val piece =
            CreateItem(
                title = title,
                kind = kind,
                composer = composer,
                key = key,
                tempo = TempoInput(marking, bpm),
                notes = notes,
                tags = tags.toList(),
                variationLabels = variations.map { it.label },
            )
        return if (kind == ItemKind.PIECE && exercises.isNotEmpty()) {
            Event.Item(ItemEvent.AddPieceInFull(piece, null, exercises.map { it.entry }))
        } else {
            Event.Item(ItemEvent.Add(piece))
        }
    }

    fun editEvent(id: String): Event =
        Event.Item(
            ItemEvent.Edit(
                id,
                UpdateItem(
                    title = title,
                    kind = kind,
                    composer = composer,
                    key = key?.let { KeyEdit.Set(it) } ?: KeyEdit.Clear,
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
                key = item.key
                marking = item.tempoMarking.orEmpty()
                bpm = item.tempoBpm?.toString().orEmpty()
                notes = item.notes.orEmpty()
                tags.addAll(item.tags)
                variations.addAll(
                    item.variations.map {
                        VariationRow(it.id, it.label, hasMarks = it.scoreHistory.isNotEmpty())
                    }
                )
            }

        // Key is not Bundle-able, so it crosses as the core's bincode.
        val Saver: Saver<ItemFormState, Any> =
            mapSaver(
                save = { form ->
                    mapOf(
                        "kind" to form.kind.name,
                        "title" to form.title,
                        "composer" to form.composer,
                        "key" to form.key?.bincodeSerialize(),
                        "marking" to form.marking,
                        "bpm" to form.bpm,
                        "notes" to form.notes,
                        "tags" to ArrayList(form.tags),
                        "variantIds" to ArrayList(form.variations.map { it.variantId }),
                        "labels" to ArrayList(form.variations.map { it.label }),
                        "rowIds" to ArrayList(form.variations.map { it.id }),
                        "marks" to form.variations.map { it.hasMarks }.toBooleanArray(),
                        "error" to form.formError,
                        "fault" to form.faultedExercise?.bincodeSerialize(),
                    ) + savedExercises(form.exercises)
                },
                restore = ::restored,
            )

        private fun restored(saved: Map<String, Any?>): ItemFormState? {
            val kind = ItemKind.entries.firstOrNull { it.name == saved["kind"] } ?: return null
            fun text(name: String) = (saved[name] as? String).orEmpty()
            fun strings(name: String) = (saved[name] as? List<*>).orEmpty()
            return ItemFormState(kind).apply {
                title = text("title")
                composer = text("composer")
                key = (saved["key"] as? ByteArray)?.let(Key::bincodeDeserialize)
                marking = text("marking")
                bpm = text("bpm")
                notes = text("notes")
                tags.addAll(strings("tags").filterIsInstance<String>())
                val ids = strings("variantIds").map { it as? String }
                val rowIds = strings("rowIds").map { it as? String }
                val marks = saved["marks"] as? BooleanArray ?: BooleanArray(0)
                strings("labels").filterIsInstance<String>().forEachIndexed { index, label ->
                    val variantId = ids.getOrNull(index)
                    val hasMarks = marks.getOrElse(index) { false }
                    val rowId = rowIds.getOrNull(index)
                    variations.add(
                        if (rowId != null) VariationRow(variantId, label, hasMarks, rowId)
                        else VariationRow(variantId, label, hasMarks)
                    )
                }
                formError = saved["error"] as? String
                exercises.addAll(restoredExercises(saved))
                faultedExercise =
                    (saved["fault"] as? ByteArray)?.let(FormErrorTarget::bincodeDeserialize)
                        as? FormErrorTarget.Exercise
            }
        }

        private fun savedExercises(staged: List<StagedExercise>): Map<String, Any?> =
            mapOf("exerciseCount" to staged.size) +
                staged.flatMapIndexed { index, row ->
                    when (row) {
                        is StagedExercise.Written ->
                            listOf(
                                "exercise.$index.title" to row.title,
                                "exercise.$index.key" to row.key?.bincodeSerialize(),
                                "exercise.$index.bpm" to row.bpm,
                                "exercise.$index.id" to row.id,
                            )
                        is StagedExercise.Chosen ->
                            listOf(
                                "exercise.$index.title" to row.title,
                                "exercise.$index.meta" to row.meta,
                                "exercise.$index.chosen" to row.id,
                            )
                    }
                }

        // All rows or none: a dropped row would shift every later one under the restored mark.
        private fun restoredExercises(saved: Map<String, Any?>): List<StagedExercise> {
            val rows =
                (0 until (saved["exerciseCount"] as? Int ?: 0)).map { restoredExercise(saved, it) }
            return if (rows.all { it != null }) rows.filterNotNull() else emptyList()
        }

        private fun restoredExercise(saved: Map<String, Any?>, index: Int): StagedExercise? {
            val title = saved["exercise.$index.title"] as? String
            val chosen = saved["exercise.$index.chosen"] as? String
            val id = saved["exercise.$index.id"] as? String
            return when {
                title == null -> null
                chosen != null ->
                    StagedExercise.Chosen(chosen, title, saved["exercise.$index.meta"] as? String)
                id != null ->
                    StagedExercise.Written(
                        title,
                        (saved["exercise.$index.key"] as? ByteArray)?.let(Key::bincodeDeserialize),
                        (saved["exercise.$index.bpm"] as? String).orEmpty(),
                        id,
                    )
                else -> null
            }
        }
    }
}

// A refusal, and the row it names, come back for the form to show inline and leave the core, so the
// app banner does not repeat it; nothing closes until the core accepts (#1595).
fun Store.sendFromForm(event: Event, onTarget: (FormErrorTarget?) -> Unit = {}): String? {
    val accepted = sendAccepted(event)
    val error = viewModel.value?.error ?: if (accepted) null else SAVE_FAILED
    onTarget(if (error != null) viewModel.value?.errorTarget else null)
    if (error != null) send(Event.ClearError)
    return error
}

private const val SAVE_FAILED = "Couldn't save. Try again."

@Composable
fun LibraryAddRoute(store: Store, onDone: () -> Unit, modifier: Modifier = Modifier) {
    val form = rememberSaveable(saver = ItemFormState.Saver) { ItemFormState() }
    val rows by store.libraryRows.collectAsState()
    var closing by remember { mutableStateOf(false) }
    ItemFormScreen(
        form,
        ItemFormMode.ADD,
        onCancel = onDone,
        exerciseLibrary = rows.filter { it.itemType == ItemKind.EXERCISE },
        onConfirm = {
            if (!closing) {
                form.formError =
                    store.sendFromForm(form.addEvent()) {
                        form.faultedExercise = it as? FormErrorTarget.Exercise
                    }
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
    val form = rememberSaveable(id, saver = ItemFormState.Saver) { ItemFormState.of(item) }
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
    exerciseLibrary: List<LibraryItemView> = emptyList(),
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
            ItemFormFields(
                form,
                exerciseLibrary.takeIf { mode == ItemFormMode.ADD },
                Modifier.verticalScroll(rememberScrollState()),
            )
        }
    }
}

@Composable
private fun ItemFormFields(
    form: ItemFormState,
    exerciseLibrary: List<LibraryItemView>?,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier.padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
    ) {
        KindSegment(form.kind, form::switchKind)
        ItemFormDetails(form)
        if (form.kind == ItemKind.EXERCISE) VariationRowsCard(form.variations)
        ItemFormPractice(form)
        ItemFormNotes(form)
        ItemFormTags(form)
        if (exerciseLibrary != null && form.kind == ItemKind.PIECE) {
            ItemFormExercises(form, exerciseLibrary)
        }
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
