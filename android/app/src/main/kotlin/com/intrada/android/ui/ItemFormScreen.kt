package com.intrada.android.ui

import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.mapSaver
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.style.TextAlign
import com.intrada.android.core.Reporter
import com.intrada.android.core.SentryReporter
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.label
import com.intrada.ffi.CoreException
import com.intrada.ffi.FormFieldNow
import com.intrada.ffi.FormReadField
import com.intrada.ffi.fillFormFromRead
import com.intrada.ffi.itemFormCanSave
import com.intrada.shared.CreateItem
import com.intrada.shared.Event
import com.intrada.shared.FormErrorTarget
import com.intrada.shared.ItemEvent
import com.intrada.shared.ItemKind
import com.intrada.shared.Key
import com.intrada.shared.KeyEdit
import com.intrada.shared.LibraryItemView
import com.intrada.shared.PhotoDraft
import com.intrada.shared.TempoInput
import com.intrada.shared.UpdateItem
import com.intrada.shared.ViewModel

enum class ItemFormMode(val confirmLabel: String) {
    ADD("Add"),
    EDIT("Save");

    fun title(kind: ItemKind) = if (this == ADD) "New ${kind.label}" else "Edit"
}

class ItemFormState(kind: ItemKind = ItemKind.PIECE) {
    var kind by mutableStateOf(kind)
    var key by mutableStateOf<Key?>(null)
    var notes by mutableStateOf("")

    /** The page the fields were read off, kept on the piece so it is not photographed twice. */
    var photoId by mutableStateOf<String?>(null)
    var filledFrom by mutableStateOf<String?>(null)

    /**
     * Fields still holding the page's read, and whether that read was weak. Typing takes a field
     * off: from that keystroke it is the musician's, not the page's.
     */
    val readFrom = mutableStateMapOf<FormReadField, Boolean>()

    private var storedTitle by mutableStateOf("")
    private var storedComposer by mutableStateOf("")
    private var storedMarking by mutableStateOf("")
    private var storedBpm by mutableStateOf("")

    var title: String
        get() = storedTitle
        set(value) {
            storedTitle = value
            readFrom.remove(FormReadField.TITLE)
        }

    var composer: String
        get() = storedComposer
        set(value) {
            storedComposer = value
            readFrom.remove(FormReadField.COMPOSER)
        }

    var marking: String
        get() = storedMarking
        set(value) {
            storedMarking = value
            readFrom.remove(FormReadField.MARKING)
        }

    var bpm: String
        get() = storedBpm
        set(value) {
            storedBpm = value
            readFrom.remove(FormReadField.BPM)
        }

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

    /** The core picks the fields (#2229); nothing is saved until Add. */
    fun fill(draft: PhotoDraft, reporter: Reporter = SentryReporter) {
        val now =
            listOf(
                FormReadField.TITLE to storedTitle,
                FormReadField.COMPOSER to storedComposer,
                FormReadField.MARKING to storedMarking,
                FormReadField.BPM to storedBpm,
            )
        val fills =
            try {
                fillFormFromRead(
                    draft.bincodeSerialize(),
                    now.map { (field, text) -> FormFieldNow(field, text, field in readFrom) },
                )
            } catch (e: CoreException) {
                reporter.report(e, "bridge")
                return
            }
        for (fill in fills) {
            when (fill.field) {
                FormReadField.TITLE -> storedTitle = fill.value
                FormReadField.COMPOSER -> storedComposer = fill.value
                FormReadField.MARKING -> storedMarking = fill.value
                FormReadField.BPM -> storedBpm = fill.value
                // Chord charts wait for #2025 on Android, so a read chart has nowhere to go.
                FormReadField.CHART -> continue
            }
            readFrom[fill.field] = fill.weak
        }
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
                photoId = photoId,
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
                        "photo" to form.photoId,
                        "filledFrom" to form.filledFrom,
                        "readFields" to ArrayList(form.readFrom.keys.map { it.name }),
                        "readWeak" to form.readFrom.values.toBooleanArray(),
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
                photoId = saved["photo"] as? String
                filledFrom = saved["filledFrom"] as? String
                val weak = saved["readWeak"] as? BooleanArray ?: BooleanArray(0)
                strings("readFields").forEachIndexed { index, name ->
                    val field = FormReadField.entries.firstOrNull { it.name == name }
                    if (field != null && index < weak.size) readFrom[field] = weak[index]
                }
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

data class FormVocabulary(
    val composers: List<String> = emptyList(),
    val tags: List<String> = emptyList(),
) {
    companion object {
        fun of(view: ViewModel?) =
            FormVocabulary(view?.availableComposers.orEmpty(), view?.availableTags.orEmpty())
    }
}

@Composable
fun LibraryAddRoute(
    store: Store,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    kind: ItemKind = ItemKind.PIECE,
) {
    val form = rememberSaveable(saver = ItemFormState.Saver) { ItemFormState(kind) }
    val rows by store.libraryRows.collectAsState()
    val view by store.viewModel.collectAsState()
    var closing by remember { mutableStateOf(false) }
    val recognition = view?.photoRecognition
    // Keyed on the projection, not the draft: a rescan of the same page reads to an equal draft.
    DisposableEffect(recognition) {
        recognition?.photoId?.let { form.photoId = it }
        val draft = recognition?.draft
        if (draft != null && recognition.photoId != form.filledFrom) {
            form.fill(draft)
            form.filledFrom = recognition.photoId
        }
        onDispose {}
    }
    val activity = LocalActivity.current
    DisposableEffect(store) {
        onDispose {
            if (activity?.isChangingConfigurations != true) store.send(Event.DiscardPhotoDraft)
        }
    }
    ItemFormScreen(
        form,
        ItemFormMode.ADD,
        onCancel = onDone,
        header = {
            if (recognition != null) {
                ScanPageEntry(recognition, { store.send(Event.Item(ItemEvent.ReadPhoto(it))) })
            }
        },
        exerciseLibrary = rows.filter { it.itemType == ItemKind.EXERCISE },
        vocabulary = FormVocabulary.of(view),
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
    val view by store.viewModel.collectAsState()
    var closing by remember { mutableStateOf(false) }
    ItemFormScreen(
        form,
        ItemFormMode.EDIT,
        onCancel = onDone,
        vocabulary = FormVocabulary.of(view),
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
    header: @Composable () -> Unit = {},
    exerciseLibrary: List<LibraryItemView> = emptyList(),
    vocabulary: FormVocabulary = FormVocabulary(),
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
                vocabulary,
                header,
                Modifier.verticalScroll(rememberScrollState()),
            )
        }
    }
}

@Composable
private fun ItemFormFields(
    form: ItemFormState,
    exerciseLibrary: List<LibraryItemView>?,
    vocabulary: FormVocabulary,
    header: @Composable () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier.padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
    ) {
        header()
        KindSegment(form.kind, form::switchKind)
        ItemFormDetails(form, vocabulary.composers)
        if (form.kind == ItemKind.EXERCISE) VariationRowsCard(form.variations)
        ItemFormPractice(form)
        ItemFormNotes(form)
        ItemFormTags(form, vocabulary.tags)
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
