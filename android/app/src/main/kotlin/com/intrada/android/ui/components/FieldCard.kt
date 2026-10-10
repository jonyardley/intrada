package com.intrada.android.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaFont
import com.intrada.android.ui.IntradaSpacing

@Composable
fun FieldLabel(
    text: String,
    modifier: Modifier = Modifier,
    colour: Color = IntradaColor.inkSecondary,
) {
    BasicText(text, modifier, style = IntradaFont.label.copy(color = colour))
}

@Composable
fun FieldCard(
    label: String,
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier
            .fillMaxWidth()
            .cardSurface()
            .padding(horizontal = IntradaSpacing.card, vertical = 10.dp),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        FieldLabel(label)
        content()
    }
}
