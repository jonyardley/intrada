package com.intrada.android.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.scaled
import com.intrada.shared.ActiveSessionView

// ── The variation being played and the last-time offer (#2249) ──

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun PlayChips(active: ActiveSessionView, actions: PlayChipActions) {
    val lastTime = active.record.lastTime
    val hasVariations = active.currentVariations.isNotEmpty()
    if (!hasVariations && lastTime == null) {
        active.currentPlayLabel?.let {
            BasicText(it, style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary))
        }
        return
    }
    FlowRow(
        horizontalArrangement =
            Arrangement.spacedBy(IntradaSpacing.controlGap, Alignment.CenterHorizontally),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        if (hasVariations) VariationChip(active.currentPlayLabel, actions.onPickVariation)
        if (lastTime != null) {
            OfferChip(
                lastTime.label,
                "player.lastTime",
                "Plays it the way you did last time",
                actions.onLastTime,
            )
        }
    }
}

@Composable
private fun VariationChip(label: String?, onClick: () -> Unit) {
    Row(
        Modifier.heightIn(min = 48.dp)
            .clip(CircleShape)
            .background(IntradaColor.cardFill)
            .border(1.dp, IntradaColor.hairline, CircleShape)
            .clickable(role = Role.Button, onClick = onClick)
            .clearAndSetSemantics {
                contentDescription = "Variation"
                stateDescription = label ?: "none picked"
                role = Role.Button
                testTag = "player.variation"
                onClick(label = "Switches to another variation of this exercise") {
                    onClick()
                    true
                }
            }
            .padding(horizontal = IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(
            label ?: "Pick a variation",
            style =
                IntradaFont.segment.copy(
                    color = if (label == null) IntradaColor.inkSecondary else IntradaColor.ink
                ),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        ChipIcon(R.drawable.ic_chevron_down, IntradaIconSize.caption)
    }
}

/** An offer the musician can take or ignore: dashed, so it never reads as a choice made. */
@Composable
internal fun OfferChip(
    title: String,
    tag: String,
    hint: String,
    onClick: () -> Unit,
    @DrawableRes icon: Int = R.drawable.ic_restore,
) {
    Row(
        Modifier.heightIn(min = 48.dp)
            .clip(CircleShape)
            .background(IntradaColor.cardFill)
            .dashedCapsule()
            .clickable(role = Role.Button, onClick = onClick)
            .clearAndSetSemantics {
                contentDescription = title
                role = Role.Button
                testTag = tag
                onClick(label = hint) {
                    onClick()
                    true
                }
            }
            .padding(horizontal = IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        ChipIcon(icon, IntradaIconSize.badge)
        BasicText(title, style = IntradaFont.smallMedium.copy(color = IntradaColor.ink))
    }
}

@Composable
private fun ChipIcon(@DrawableRes icon: Int, size: IntradaIconSize) {
    Image(
        painterResource(icon),
        contentDescription = null,
        modifier = Modifier.size(size.scaled()),
        colorFilter = ColorFilter.tint(IntradaColor.inkSecondary),
    )
}

private fun Modifier.dashedCapsule() = drawBehind {
    val stroke = 1.dp.toPx()
    drawRoundRect(
        IntradaColor.addDashOutline,
        topLeft = Offset(stroke / 2, stroke / 2),
        size = Size(size.width - stroke, size.height - stroke),
        cornerRadius = CornerRadius(size.height / 2),
        style = Stroke(stroke, pathEffect = PathEffect.dashPathEffect(floatArrayOf(DASH, GAP))),
    )
}

internal fun Modifier.buttonSemantics(spoken: String, tag: String, onClick: () -> Unit) =
    clickable(role = Role.Button, onClick = onClick).clearAndSetSemantics {
        contentDescription = spoken
        role = Role.Button
        testTag = tag
        onClick {
            onClick()
            true
        }
    }

private const val DASH = 12f
private const val GAP = 9f
