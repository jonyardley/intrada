package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
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
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.components.FieldLabel

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

@Composable
internal fun AddInputRow(
    addLabel: String,
    tag: String,
    onAdd: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var typed by rememberSaveable { mutableStateOf("") }
    Row(modifier, verticalAlignment = Alignment.Bottom) {
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
