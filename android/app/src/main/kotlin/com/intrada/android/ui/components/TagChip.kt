package com.intrada.android.ui.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.IntradaColor
import com.intrada.android.ui.IntradaFont
import com.intrada.android.ui.IntradaIconSize

@Composable
fun TagChip(
    text: String,
    modifier: Modifier = Modifier,
    style: TagChipStyle = TagChipStyle.Sunken,
    onRemove: (() -> Unit)? = null,
) {
    if (onRemove == null) {
        Chip(text, style, removable = false, modifier)
    } else {
        Box(
            modifier
                .heightIn(min = 48.dp)
                .clickable(onClickLabel = "remove", role = Role.Button, onClick = onRemove)
                .clearAndSetSemantics { contentDescription = text },
            contentAlignment = Alignment.Center,
        ) {
            Chip(text, style, removable = true)
        }
    }
}

@Composable
private fun Chip(
    text: String,
    style: TagChipStyle,
    removable: Boolean,
    modifier: Modifier = Modifier,
) {
    val outlined = style == TagChipStyle.Outlined
    val vertical =
        when {
            outlined -> 5.dp
            removable -> 4.dp
            else -> 3.dp
        }
    val horizontal =
        when {
            outlined -> 10.dp
            removable -> 9.dp
            else -> 8.dp
        }
    val fill = if (outlined) IntradaColor.cardFill else IntradaColor.surfaceSunken
    val edge = if (outlined) Modifier.border(1.dp, IntradaColor.hairline, CircleShape) else Modifier
    Row(
        modifier
            .background(fill, CircleShape)
            .then(edge)
            .padding(horizontal = horizontal, vertical = vertical),
        horizontalArrangement = Arrangement.spacedBy(5.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(
            text,
            Modifier.weight(1f, fill = false),
            style = IntradaFont.metaMedium.copy(color = IntradaColor.inkSecondary),
            overflow = TextOverflow.Ellipsis,
            maxLines = 1,
        )
        if (removable) {
            Image(
                painterResource(R.drawable.ic_close),
                contentDescription = null,
                modifier = Modifier.size(IntradaIconSize.badge.scaled()),
                colorFilter = ColorFilter.tint(IntradaColor.inkFaintIcon),
            )
        }
    }
}
