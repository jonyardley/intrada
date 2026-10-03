package com.intrada.android.ui.components

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.IntradaRadius
import com.intrada.android.ui.IntradaShadow

fun Modifier.dropShadow(
    token: IntradaShadow,
    shape: Shape = RoundedCornerShape(IntradaRadius.card),
): Modifier =
    dropShadow(
        shape,
        Shadow(radius = token.radius, color = token.color, offset = DpOffset(0.dp, token.y)),
    )

fun Modifier.cardShadow(shape: Shape = RoundedCornerShape(IntradaRadius.card)): Modifier =
    dropShadow(IntradaShadow.card, shape)
