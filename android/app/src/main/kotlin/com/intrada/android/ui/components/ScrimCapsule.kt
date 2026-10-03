package com.intrada.android.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaFont
import com.intrada.android.ui.IntradaOpacity
import com.intrada.android.ui.IntradaSpacing

// At 0.8 over a white page the label reads 8:1; 0.55 was 3.4:1, under the AA floor.
@Composable
fun ScrimCapsule(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier
            .heightIn(min = 48.dp)
            .widthIn(min = 48.dp)
            .clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Box(
            Modifier.clip(CircleShape)
                .background(IntradaColor.viewerBackdrop.copy(alpha = IntradaOpacity.strong))
                .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
            contentAlignment = Alignment.Center,
        ) {
            BasicText(label, style = IntradaFont.bodyMedium.copy(color = IntradaColor.onAccent))
        }
    }
}
