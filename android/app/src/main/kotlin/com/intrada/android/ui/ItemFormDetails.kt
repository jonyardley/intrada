package com.intrada.android.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface

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
