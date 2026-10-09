package com.intrada.android.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.intrada.android.ui.components.cardSurface

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
