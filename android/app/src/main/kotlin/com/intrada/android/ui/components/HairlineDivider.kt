package com.intrada.android.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.IntradaColor

@Composable
fun HairlineDivider(
    modifier: Modifier = Modifier,
    orientation: Orientation = Orientation.Horizontal,
    colour: Color = IntradaColor.hairline,
) {
    val sized =
        when (orientation) {
            Orientation.Horizontal -> modifier.fillMaxWidth().height(1.dp)
            Orientation.Vertical -> modifier.fillMaxHeight().width(1.dp)
        }
    Box(sized.background(colour))
}
