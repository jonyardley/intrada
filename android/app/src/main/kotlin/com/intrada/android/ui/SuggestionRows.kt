package com.intrada.android.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.components.HairlineDivider

@Composable
internal fun SuggestionRows(
    matches: List<String>,
    clickLabel: (String) -> String,
    onPick: (String) -> Unit,
) {
    matches.forEach { match ->
        HairlineDivider(Modifier.padding(start = IntradaSpacing.card))
        BasicText(
            match,
            Modifier.fillMaxWidth()
                .heightIn(min = 48.dp)
                .clickable(onClickLabel = clickLabel(match), role = Role.Button) { onPick(match) }
                .testTag("suggestion.row")
                .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
            style = IntradaFont.body.copy(color = IntradaColor.ink),
        )
    }
}
