package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.ItemKind

@Composable
internal fun ItemFormDetails(form: ItemFormState, modifier: Modifier = Modifier) {
    Column(modifier.cardSurface()) {
        FormField(
            "Title",
            form.title,
            { form.title = it },
            "itemForm.title",
            placeholder = "Required",
        )
        HairlineDivider()
        FormField("Composer", form.composer, { form.composer = it }, "itemForm.composer")
        HairlineDivider()
        KeyPicker(form.key, { form.key = it })
    }
}

@Composable
internal fun KindSegment(
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
