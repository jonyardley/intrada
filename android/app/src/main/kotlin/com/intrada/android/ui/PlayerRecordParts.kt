package com.intrada.android.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.scaled
import com.intrada.shared.ActiveSessionView
import com.intrada.shared.SegmentClockView
import com.intrada.shared.SegmentProgress
import java.time.Instant

// ── The section clock and its move-on offer (#2315, #2471) ──

/** Rounded up, so the last second reads 00:01 rather than 00:00 while it is still running. */
internal fun secondsLeft(endsAt: Instant, now: Instant): Long =
    -Math.floorDiv(now.toEpochMilli() - endsAt.toEpochMilli(), MILLIS_PER_SECOND)

/** The core leaves this clock comparison to the shell (#2315). */
internal fun offersMoveOn(segment: SegmentClockView?, now: Instant): Boolean {
    val endsAt = segment?.let { SessionClock.parse(it.endsAt) } ?: return false
    return segment.moveLabel != null && secondsLeft(endsAt, now) <= 0
}

@Composable
internal fun SegmentClockRow(
    segment: SegmentClockView,
    now: Instant,
    onMove: () -> Unit,
    onStay: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val endsAt = SessionClock.parse(segment.endsAt) ?: return
    val left = secondsLeft(endsAt, now)
    val moveLabel = segment.moveLabel
    Column(
        modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        val clock = "${SessionClock.clockDisplay(left)} left"
        Row(
            Modifier.clearAndSetSemantics {
                contentDescription = "${segment.label}, $clock"
                testTag = "player.segment"
            },
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(segment.label, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
            BasicText(clock, style = IntradaFont.figure.copy(color = IntradaColor.inkSecondary))
        }
        SectionStrip(segment)
        if (moveLabel != null && left <= 0) {
            InkButton(moveLabel, "player.moveOn", onMove)
            segment.stayLabel?.let { stay ->
                val shape = RoundedCornerShape(IntradaRadius.control)
                Box(
                    Modifier.fillMaxWidth()
                        .heightIn(min = 48.dp)
                        .clip(shape)
                        .background(IntradaColor.cardFill)
                        .border(1.dp, IntradaColor.divider, shape)
                        .buttonSemantics(stay, "player.stay", onStay)
                        .padding(horizontal = IntradaSpacing.cardCompact),
                    contentAlignment = Alignment.Center,
                ) {
                    BasicText(
                        stay,
                        style =
                            IntradaFont.bodyMedium.copy(
                                color = IntradaColor.ink,
                                textAlign = TextAlign.Center,
                            ),
                    )
                }
            }
        }
    }
}

// Every section of the item, filled once done, with what Stay took (#2394). Drawn by hand until
// the shared strip lands (#2636).
@Composable
private fun SectionStrip(segment: SegmentClockView) {
    val bars = segment.bars
    Column(
        Modifier.fillMaxWidth().clearAndSetSemantics { contentDescription = segment.positionLabel },
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.segmentGap),
    ) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.segmentGap),
        ) {
            bars.forEach { bar ->
                Box(
                    Modifier.weight(1f)
                        .height(6.dp)
                        .clip(CircleShape)
                        .background(
                            if (bar.progress == SegmentProgress.DONE) IntradaColor.accent
                            else IntradaColor.divider
                        )
                )
            }
        }
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.segmentGap),
        ) {
            bars.forEach { bar ->
                val running = bar.progress == SegmentProgress.RUNNING
                Column(Modifier.weight(1f), horizontalAlignment = Alignment.CenterHorizontally) {
                    BasicText(
                        bar.label,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        style =
                            (if (running) IntradaFont.smallMedium else IntradaFont.small).copy(
                                color = if (running) IntradaColor.ink else IntradaColor.inkSecondary
                            ),
                    )
                    val caption = bar.caption
                    if (caption != null) {
                        BasicText(
                            caption,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
                        )
                    }
                }
            }
        }
    }
}

// ── The variation being played and the last-time offer (#2249) ──

class PlayChipActions(val onPickVariation: () -> Unit, val onLastTime: () -> Unit)

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

// ── Trouble spot (#2249) ──

@Composable
internal fun TroubleSpotButton(onClick: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier
            .heightIn(min = 48.dp)
            .clip(CircleShape)
            .background(IntradaColor.cardFill)
            .border(1.dp, IntradaColor.divider, CircleShape)
            .buttonSemantics("Trouble spot", "player.troubleSpot", onClick)
            .padding(horizontal = IntradaSpacing.card),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        ChipIcon(R.drawable.ic_plus, IntradaIconSize.caption)
        BasicText("Trouble spot", style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
    }
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

private const val MILLIS_PER_SECOND = 1000L
private const val DASH = 12f
private const val GAP = 9f
