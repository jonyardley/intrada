package com.intrada.android.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaRadius

fun Modifier.cardSurface(cornerRadius: Dp = IntradaRadius.card): Modifier {
    val shape = RoundedCornerShape(cornerRadius)
    return clip(shape).background(IntradaColor.cardFill).border(1.dp, IntradaColor.hairline, shape)
}
