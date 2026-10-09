package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.TagChip
import com.intrada.android.ui.components.cardSurface

private class ChipListLabels(val label: String, val addLabel: String, val tag: String)

@Composable
internal fun ItemFormNotes(form: ItemFormState, modifier: Modifier = Modifier) {
    Column(modifier.cardSurface()) {
        FormField(
            "Notes",
            form.notes,
            { form.notes = it },
            "itemForm.notes",
            singleLine = false,
        )
    }
}

@Composable
internal fun ItemFormTags(form: ItemFormState, modifier: Modifier = Modifier) {
    ChipListCard(
        ChipListLabels("Tags", "Add a tag", "itemForm.tag"),
        form.tags,
        onRemove = { form.tags.removeAt(it) },
        onAdd = { form.tags.add(it) },
        modifier,
    )
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
        AddInputRow(labels.addLabel, labels.tag, onAdd)
    }
}
