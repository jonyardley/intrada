package com.intrada.android.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaGlyph
import com.intrada.shared.InstrumentIcon

// 22pt of the 36pt header badge in the profile mock (#1692).
private const val GLYPH_SHARE = 0.62f

@Composable
fun ProfileBadge(
    icon: InstrumentIcon,
    marker: Color,
    modifier: Modifier = Modifier,
    size: Dp = IntradaGlyph.tile,
) {
    Box(modifier.size(size).background(marker, CircleShape), contentAlignment = Alignment.Center) {
        InstrumentGlyph(icon, size = size * GLYPH_SHARE, tint = IntradaColor.onMarker)
    }
}
