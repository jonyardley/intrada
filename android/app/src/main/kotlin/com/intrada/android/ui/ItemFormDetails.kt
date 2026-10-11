package com.intrada.android.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalFocusManager
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.ffi.FormReadField
import com.intrada.ffi.formSuggestions

@Composable
internal fun ItemFormDetails(
    form: ItemFormState,
    composers: List<String>,
    modifier: Modifier = Modifier,
) {
    val focusManager = LocalFocusManager.current
    var composerFocused by remember { mutableStateOf(false) }
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
        FormField(
            "Composer",
            form.composer,
            { form.composer = it },
            "itemForm.composer",
            Modifier.onFocusChanged { composerFocused = it.hasFocus },
        )
        form.readFrom[FormReadField.COMPOSER]?.let { FieldMark(it) }
        if (composerFocused) {
            SuggestionRows(
                formSuggestions(composers, form.composer, emptyList()),
                { "Fills Composer with $it" },
            ) {
                form.composer = it
                focusManager.clearFocus()
            }
        }
        HairlineDivider()
        KeyPicker(form.key, { form.key = it })
    }
}
