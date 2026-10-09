package com.intrada.android.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface

@Composable
internal fun ItemFormPractice(form: ItemFormState, modifier: Modifier = Modifier) {
    Column(modifier.cardSurface()) {
        FormField(
            "Tempo marking",
            form.marking,
            { form.marking = it },
            "itemForm.marking",
            placeholder = "e.g. Allegro",
        )
        HairlineDivider()
        FormField(
            "Beats per minute",
            form.bpm,
            { form.bpm = it },
            "itemForm.bpm",
            keyboard = KeyboardType.Number,
        )
    }
}
