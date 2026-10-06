package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.input.KeyboardType
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldCard
import com.intrada.android.ui.components.FormErrorBanner
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.Event
import com.intrada.shared.ItemEvent
import com.intrada.shared.LibraryItemView
import com.intrada.shared.SectionChange
import com.intrada.shared.SectionKind

@Composable
fun SectionRoute(
    store: Store,
    pieceId: String,
    sectionId: String?,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val rows by store.libraryRows.collectAsState()
    var closing by remember { mutableStateOf(false) }
    val piece = rows.firstOrNull { it.id == pieceId }
    val existing = piece?.sections?.firstOrNull { it.id == sectionId }
    val item = rememberLastFound(piece?.takeIf { sectionId == null || existing != null }, closing)
    if (item == null) {
        MissingItem("Section", NO_LONGER_THERE, modifier)
        return
    }
    val form = remember(pieceId, sectionId) { SectionFormState(existing) }
    fun send(change: SectionChange) {
        if (closing) return
        form.formError = store.sendFromForm(Event.Item(ItemEvent.ChangeSection(pieceId, change)))
        closing = form.formError == null
        if (closing) onDone()
    }
    SectionScreen(
        item,
        form,
        onCancel = onDone,
        onSave = { send(form.saveChange()) },
        onRemove = { id -> send(SectionChange.Remove(id)) },
        modifier,
    )
}

@Composable
fun SectionScreen(
    item: LibraryItemView,
    form: SectionFormState,
    onCancel: () -> Unit,
    onSave: () -> Unit,
    onRemove: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    ScreenScaffold(
        form.existing?.label ?: "New section",
        modifier,
        actions = {
            TextAction("Cancel", "sectionSheet.cancel", onCancel)
            TextAction("Save", "sectionSheet.save", onSave, emphasised = true)
        },
    ) {
        Column(Modifier.fillMaxSize()) {
            form.formError?.let {
                FormErrorBanner(
                    it,
                    Modifier.padding(horizontal = IntradaSpacing.card)
                        .padding(top = IntradaSpacing.cardCompact)
                        .testTag("sectionSheet.error"),
                )
            }
            Column(
                Modifier.verticalScroll(rememberScrollState()).padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
            ) {
                SectionFields(form, item)
                if (form.existing != null) {
                    DeleteButton(
                        "Remove section",
                        "sectionSheet.remove",
                        onClick = { form.confirmingRemoval = true },
                    )
                }
            }
        }
    }
    val existing = form.existing
    if (existing != null && form.confirmingRemoval) {
        ConfirmDialog(
            ConfirmCopy("Remove ${existing.label}?", null, "Remove", "sectionSheet.removal"),
            onConfirm = {
                form.confirmingRemoval = false
                onRemove(existing.id)
            },
            onDismiss = { form.confirmingRemoval = false },
        )
    }
}

@Composable
private fun SectionFields(form: SectionFormState, item: LibraryItemView) {
    val kindName = item.itemType.label.lowercase()
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.card)) {
        Column(Modifier.cardSurface()) {
            FormField(
                "Name",
                form.name,
                { form.name = it },
                "sectionSheet.name",
                placeholder = "A1, Coda, Exposition",
            )
            HairlineDivider()
            FormField(
                "Bars",
                form.bars,
                { form.bars = it },
                "sectionSheet.bars",
                placeholder = "1 to 16",
                note = "Optional. Bar 12, or 12 to 14",
            )
        }
        FieldCard("Kind") {
            SegmentedPills(
                listOf(SectionKind.FORM, SectionKind.TROUBLESPOT),
                form.kind,
                { form.kind = it },
                label = { if (it == SectionKind.FORM) "Part of the $kindName" else TRICKY_SPOT },
                tag = {
                    if (it == SectionKind.FORM) "sectionSheet.kind.form"
                    else "sectionSheet.kind.spot"
                },
            )
        }
        Column(Modifier.cardSurface()) {
            FormField(
                "Target tempo",
                form.bpm,
                { form.bpm = it },
                "sectionSheet.bpm",
                placeholder = item.tempoBpm?.toString().orEmpty(),
                keyboard = KeyboardType.Number,
                note = "Beats per minute. Leave empty to use the $kindName's tempo",
            )
        }
    }
}
