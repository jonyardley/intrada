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
import com.intrada.ffi.formSuggestions

private class ChipListLabels(val label: String, val addLabel: String, val tag: String)

@Composable
internal fun ItemFormTags(form: ItemFormState, tags: List<String>, modifier: Modifier = Modifier) {
    ChipListCard(
        ChipListLabels("Tags", "Add a tag", "itemForm.tag"),
        form.tags,
        onRemove = { form.tags.removeAt(it) },
        onAdd = { form.tags.add(it) },
        suggest = { formSuggestions(tags, it, form.tags) },
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
    suggest: (String) -> List<String>,
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
        AddInputRow(
            labels.addLabel,
            labels.tag,
            onAdd,
            suggest = suggest,
            suggestionLabel = { "Adds the tag $it" },
        )
    }
}
