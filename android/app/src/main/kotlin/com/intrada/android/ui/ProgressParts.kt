package com.intrada.android.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.intrada.android.R
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.dropShadow
import com.intrada.android.ui.components.scaled
import com.intrada.shared.ConsistencyWeekView
import java.util.Locale

// ── Hero ──

@Composable
internal fun MasteryHeroCard(
    mastery: Double,
    topMark: Int,
    change: String?,
    climbing: String?,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier.fillMaxWidth().cardSurface(IntradaRadius.panel).padding(HERO_GUTTER),
        horizontalArrangement = Arrangement.spacedBy(HERO_GUTTER),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        MasteryDial(mastery, topMark)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            FieldLabel("Overall mastery")
            if (change != null) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(5.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Glyph(R.drawable.ic_tab_progress, IntradaColor.success)
                    BasicText(
                        change,
                        style = IntradaFont.secondary.copy(color = IntradaColor.success),
                    )
                }
            }
            if (climbing != null) {
                BasicText(
                    climbing,
                    style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
    }
}

private val HERO_GUTTER = 18.dp

@Composable
internal fun MasteryDial(value: Double, topMark: Int, modifier: Modifier = Modifier) {
    val number = oneDecimal(value)
    val of = oneDecimal(topMark.toDouble())
    val fraction = if (topMark > 0) (value / topMark).toFloat().coerceIn(0f, 1f) else 0f
    Box(
        modifier.size(DIAL_SIZE).clearAndSetSemantics {
            contentDescription = "Overall mastery $number of $of"
        },
        contentAlignment = Alignment.Center,
    ) {
        Canvas(Modifier.fillMaxSize()) {
            val stroke = DIAL_RING.toPx()
            val inset = stroke / 2
            val arc = Size(size.width - stroke, size.height - stroke)
            drawArc(
                IntradaColor.dialTrack,
                0f,
                FULL_TURN,
                false,
                Offset(inset, inset),
                arc,
                style = Stroke(stroke),
            )
            drawArc(
                IntradaGradient.ringSweep,
                TWELVE_O_CLOCK,
                FULL_TURN * fraction,
                false,
                Offset(inset, inset),
                arc,
                style = Stroke(stroke, cap = StrokeCap.Round),
            )
        }
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(2.dp),
        ) {
            BasicText(
                number,
                style = IntradaFont.scoreNumeral(DIAL_NUMERAL.sp).copy(color = IntradaColor.ink),
            )
            BasicText(
                "of $of",
                style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}

private fun oneDecimal(value: Double) = String.format(Locale.UK, "%.1f", value)

private val DIAL_SIZE = 128.dp
private val DIAL_RING = 9.dp
private const val DIAL_NUMERAL = 128 * 0.297f
private const val FULL_TURN = 360f
private const val TWELVE_O_CLOCK = -90f

// ── Consistency ──

@Composable
internal fun ConsistencyBars(weeks: List<ConsistencyWeekView>, modifier: Modifier = Modifier) {
    val peak = maxOf(1u, weeks.maxOfOrNull { it.minutes } ?: 1u).toFloat()
    Row(
        modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(9.dp),
        verticalAlignment = Alignment.Bottom,
    ) {
        weeks.forEach { week ->
            Column(
                Modifier.weight(1f).clearAndSetSemantics { contentDescription = week.spoken },
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                val bar = RoundedCornerShape(5.dp)
                Box(
                    Modifier.fillMaxWidth()
                        .height(maxOf(BAR_MIN, BAR_MAX * (week.minutes.toFloat() / peak)))
                        .then(
                            if (week.isCurrent) Modifier.dropShadow(IntradaShadow.glow, bar)
                            else Modifier
                        )
                        .clip(bar)
                        .background(
                            if (week.isCurrent) IntradaGradient.inkBar
                            else SolidColor(IntradaColor.consistencyTrack)
                        )
                )
                BasicText(
                    week.label,
                    style =
                        IntradaFont.small.copy(
                            color =
                                if (week.isCurrent) IntradaColor.accent
                                else IntradaColor.inkSecondary,
                            fontWeight =
                                if (week.isCurrent) FontWeight.SemiBold else FontWeight.Normal,
                        ),
                )
            }
        }
    }
}

private val BAR_MAX = 58.dp
private val BAR_MIN = 6.dp

// ── Solid counts ──

@Composable
internal fun SolidCountRow(
    title: String,
    trailing: String,
    solid: Int,
    total: Int,
    spoken: String,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier
            .fillMaxWidth()
            .cardSurface()
            .clearAndSetSemantics { contentDescription = spoken }
            .padding(IntradaSpacing.cardCompact),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
            BasicText(
                title,
                Modifier.weight(1f).alignByBaseline(),
                style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink),
            )
            BasicText(
                trailing,
                Modifier.alignByBaseline(),
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        }
        Row(horizontalArrangement = Arrangement.spacedBy(5.dp)) {
            repeat(total.coerceAtLeast(0)) { index ->
                Box(
                    Modifier.weight(1f)
                        .height(6.dp)
                        .background(
                            if (index < solid) IntradaColor.accent else IntradaColor.divider,
                            RoundedCornerShape(IntradaRadius.pill),
                        )
                )
            }
        }
    }
}

// ── Mastery changes ──

@Composable
internal fun MasteryDelta(
    title: String,
    subtitle: String?,
    was: Int?,
    now: Int,
    modifier: Modifier = Modifier,
) {
    val spoken =
        listOfNotNull(
                title,
                subtitle,
                if (was != null) "mastery up from $was to $now" else "mastery $now",
            )
            .joinToString(", ")
    Row(
        modifier
            .fillMaxWidth()
            .cardSurface()
            .clearAndSetSemantics { contentDescription = spoken }
            .padding(horizontal = 14.dp, vertical = 11.dp),
        horizontalArrangement = Arrangement.spacedBy(11.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.size(8.dp).background(IntradaColor.accent, CircleShape))
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            BasicText(title, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
            if (subtitle != null) {
                BasicText(
                    subtitle,
                    style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
        Row(
            horizontalArrangement = Arrangement.spacedBy(4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (was != null) {
                BasicText(
                    "$was",
                    style = IntradaFont.cardTitle.copy(color = IntradaColor.inkSecondary),
                )
                Glyph(R.drawable.ic_arrow_right, IntradaColor.inkFaintIcon)
            }
            BasicText("$now", style = IntradaFont.cardTitle.copy(color = IntradaColor.success))
        }
    }
}

@Composable
internal fun MasteryDeltaToast(
    title: String,
    subtitle: String,
    was: Int,
    now: Int,
    modifier: Modifier = Modifier,
) {
    val ink = IntradaColor.celebrationInk
    Row(
        modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(IntradaRadius.card))
            .background(IntradaGradient.celebration)
            .clearAndSetSemantics { contentDescription = "$title, mastery up from $was to $now" }
            .padding(horizontal = 16.dp, vertical = 14.dp),
        horizontalArrangement = Arrangement.spacedBy(13.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier.size(34.dp).background(IntradaColor.marker, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Glyph(R.drawable.ic_sparkles, IntradaColor.onMarker)
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            BasicText(title, style = IntradaFont.bodyMedium.copy(color = ink))
            BasicText(
                subtitle,
                style = IntradaFont.secondary.copy(color = ink.copy(alpha = IntradaOpacity.strong)),
            )
        }
        Row(
            horizontalArrangement = Arrangement.spacedBy(5.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(
                "$was",
                style = IntradaFont.title.copy(color = ink.copy(alpha = IntradaOpacity.dimmed)),
            )
            Glyph(R.drawable.ic_arrow_right, IntradaColor.marker)
            BasicText("$now", style = IntradaFont.title.copy(color = ink))
        }
    }
}

@Composable
private fun Glyph(@DrawableRes icon: Int, tint: Color) {
    Image(
        painterResource(icon),
        contentDescription = null,
        modifier = Modifier.size(IntradaIconSize.inline.scaled()),
        colorFilter = ColorFilter.tint(tint),
    )
}
