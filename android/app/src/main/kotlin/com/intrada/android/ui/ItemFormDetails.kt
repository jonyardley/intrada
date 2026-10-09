package com.intrada.android.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.ffi.FormReadField

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
        form.readFrom[FormReadField.TITLE]?.let { FieldMark(it) }
        HairlineDivider()
        FormField("Composer", form.composer, { form.composer = it }, "itemForm.composer")
        form.readFrom[FormReadField.COMPOSER]?.let { FieldMark(it) }
        HairlineDivider()
        KeyPicker(form.key, { form.key = it })
    }
}
