package com.intrada.android.ui.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaFont
import com.intrada.android.ui.IntradaRadius
import com.intrada.android.ui.IntradaSpacing

@Composable
fun FormErrorBanner(message: String, modifier: Modifier = Modifier) {
    val text = IntradaFont.bodyMedium.copy(color = IntradaColor.danger)
    val density = LocalDensity.current
    val firstLine = with(density) { rememberTextMeasurer().measure("A", text).size.height.toDp() }
    val glyph = with(density) { text.fontSize.toDp() }
    val shape = RoundedCornerShape(IntradaRadius.card)
    Row(
        modifier
            .fillMaxWidth()
            .background(IntradaColor.dangerWash, shape)
            .border(1.dp, IntradaColor.dangerEdge, shape)
            .clearAndSetSemantics { contentDescription = "Error: $message" }
            .padding(IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        verticalAlignment = Alignment.Top,
    ) {
        Box(Modifier.height(firstLine), contentAlignment = Alignment.Center) {
            Image(
                painterResource(R.drawable.ic_warning_triangle),
                contentDescription = null,
                modifier = Modifier.size(glyph),
                colorFilter = ColorFilter.tint(IntradaColor.danger),
            )
        }
        BasicText(message, Modifier.weight(1f), style = text)
    }
}
