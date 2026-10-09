package com.intrada.android.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.min
import androidx.compose.ui.unit.sp
import com.intrada.android.R
import com.intrada.android.ui.components.dropShadow
import com.intrada.shared.CompletionStatus
import com.intrada.shared.HighlighterColour
import com.intrada.shared.ItemKind
import com.intrada.shared.LastPractisedView
import com.intrada.shared.PracticeDayView
import com.intrada.shared.PracticeSessionView

// ── Hero ──

@Composable
internal fun LastPractisedHero(
    last: LastPractisedView?,
    colour: HighlighterColour,
    onStart: () -> Unit,
) {
    val heading = last?.let { "Last practised · ${it.relativeDay}" } ?: "First session"
    val spoken = last?.let { "${it.label}, ${it.itemTitle}" } ?: heading
    val heroShape = RoundedCornerShape(IntradaRadius.hero)
    Column(
        Modifier.fillMaxWidth()
            .dropShadow(IntradaShadow.hero, heroShape)
            .clip(heroShape)
            .background(IntradaGradient.practiceHero(colour))
            .padding(IntradaSpacing.section),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        Column(
            Modifier.clearAndSetSemantics { contentDescription = spoken },
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        ) {
            BasicText(
                heading,
                style =
                    IntradaFont.label.copy(
                        color = IntradaColor.onAccent.copy(alpha = IntradaOpacity.secondary)
                    ),
            )
            last?.let {
                BasicText(
                    it.itemTitle,
                    style =
                        IntradaFont.title.copy(
                            color = IntradaColor.paperTop,
                            textAlign = TextAlign.Center,
                        ),
                    maxLines = 2,
                )
            }
        }
        Box(
            Modifier.padding(vertical = IntradaSpacing.controlGap)
                .size(HERO_BUTTON)
                .dropShadow(IntradaShadow.heroButton, CircleShape)
                .clip(CircleShape)
                .background(IntradaColor.marker(colour))
                .clickable(role = Role.Button, onClick = onStart)
                .semantics { contentDescription = "Start practising" }
                .testTag("practice.start"),
            contentAlignment = Alignment.Center,
        ) {
            Image(
                painterResource(R.drawable.ic_play),
                contentDescription = null,
                modifier = Modifier.size(IntradaIconSize.hero.points),
                colorFilter = ColorFilter.tint(IntradaColor.onMarker),
            )
        }
    }
}

private val HERO_BUTTON = 96.dp

// ── Week strip ──

@Composable
internal fun WeekStrip(days: List<PracticeDayView>, selected: String?, onSelect: (String) -> Unit) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        days.forEach { day ->
            WeekDayCell(day, day.date == selected, { onSelect(day.date) }, Modifier.weight(1f))
        }
    }
}

@Composable
private fun WeekDayCell(
    day: PracticeDayView,
    isSelected: Boolean,
    onTap: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val practised = day.sessionIds.isNotEmpty()
    val marked = isSelected || day.isToday
    val spoken =
        (if (day.isToday) "Today, ${day.fullDate}" else day.fullDate) +
            if (practised) ", practised" else ", no practice"
    Column(
        modifier.heightIn(min = 48.dp).clickable(onClick = onTap).clearAndSetSemantics {
            testTag = "practice.day.${day.date}"
            contentDescription = spoken
            role = Role.Tab
            selected = isSelected
            onClick {
                onTap()
                true
            }
        },
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(5.dp),
    ) {
        BasicText(
            day.weekdayInitial,
            style =
                IntradaFont.small.copy(
                    color = if (marked) IntradaColor.accent else IntradaColor.inkSecondary,
                    fontWeight = if (marked) FontWeight.SemiBold else FontWeight.Normal,
                ),
        )
        DayNumber(day, isSelected)
        Box(
            Modifier.size(5.dp)
                .clip(CircleShape)
                .background(if (practised) IntradaColor.accent else Color.Transparent)
        )
    }
}

@Composable
private fun DayNumber(day: PracticeDayView, isSelected: Boolean) {
    val circle = min(DAY_CIRCLE * LocalDensity.current.fontScale, DAY_CIRCLE_MAX)
    val colour =
        when {
            isSelected -> IntradaColor.onAccent
            day.isFuture -> IntradaColor.inkSecondary
            else -> IntradaColor.ink
        }
    Box(
        Modifier.size(circle)
            .clip(CircleShape)
            .background(if (isSelected) IntradaColor.accent else Color.Transparent)
            .then(
                if (day.isToday && !isSelected)
                    Modifier.border(1.5.dp, IntradaColor.accent, CircleShape)
                else Modifier
            ),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(
            day.dayNumber.toString(),
            style = IntradaFont.smallMedium.copy(color = colour),
            maxLines = 1,
        )
    }
}

private val DAY_CIRCLE = 32.dp
private val DAY_CIRCLE_MAX = 36.dp

// ── Session card ──

@Composable
internal fun SessionCard(session: PracticeSessionView, onOpen: () -> Unit) {
    val meta = "${session.totalDurationSummary} · ${session.itemCountDisplay}"
    val endedEarly = session.completionStatus == CompletionStatus.ENDEDEARLY
    val spoken =
        listOfNotNull(
                session.dayLabel,
                session.totalDurationSummary,
                session.itemCountDisplay,
                session.playedSummary.ifEmpty { null },
                if (endedEarly) "ended early" else null,
            )
            .joinToString(", ")
    Column(
        Modifier.fillMaxWidth()
            .clip(RoundedCornerShape(IntradaRadius.card))
            .background(IntradaColor.cardFill)
            .border(1.dp, IntradaColor.hairline, RoundedCornerShape(IntradaRadius.card))
            .clickable(onClick = onOpen)
            .clearAndSetSemantics {
                testTag = "practice.sessionCard"
                contentDescription = spoken
                role = Role.Button
                onClick(label = "Opens the session") {
                    onOpen()
                    true
                }
            }
            .padding(IntradaSpacing.card),
        verticalArrangement = Arrangement.spacedBy(3.dp),
    ) {
        BasicText(session.dayLabel, style = IntradaFont.cardTitle.copy(color = IntradaColor.ink))
        BasicText(meta, style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary))
        if (session.playedSummary.isNotEmpty()) {
            BasicText(
                session.playedSummary,
                Modifier.padding(top = 2.dp),
                style = IntradaFont.body.copy(color = IntradaColor.ink),
            )
        }
        if (endedEarly) {
            BasicText(
                "Ended early",
                Modifier.padding(top = 2.dp),
                style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}

// HACK(#2515): the core should word this line; iOS builds it in Swift too.
internal val PracticeSessionView.itemCountDisplay: String
    get() {
        val count = entries.size
        val noun =
            when {
                entries.all { it.itemType == ItemKind.PIECE } ->
                    if (count == 1) "piece" else "pieces"
                entries.all { it.itemType == ItemKind.EXERCISE } ->
                    if (count == 1) "exercise" else "exercises"
                else -> if (count == 1) "item" else "items"
            }
        return "$count $noun"
    }

@Composable
internal fun EmptyDayCard(future: Boolean) {
    val message = if (future) "Nothing logged yet" else "No practice logged"
    Column(
        Modifier.fillMaxWidth()
            .clip(RoundedCornerShape(IntradaRadius.card))
            .background(IntradaColor.cardFill)
            .drawBehind {
                val stroke = 1.dp.toPx()
                val radius = IntradaRadius.card.toPx()
                drawRoundRect(
                    IntradaColor.slotOutline,
                    topLeft = Offset(stroke / 2, stroke / 2),
                    size = Size(size.width - stroke, size.height - stroke),
                    cornerRadius = CornerRadius(radius),
                    style =
                        Stroke(
                            stroke,
                            pathEffect = PathEffect.dashPathEffect(floatArrayOf(DASH, DASH)),
                        ),
                )
            }
            .padding(IntradaSpacing.card)
            .clearAndSetSemantics { contentDescription = message },
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        BasicText(
            message,
            style =
                IntradaFont.bodyMedium.copy(
                    color = IntradaColor.inkSecondary,
                    textAlign = TextAlign.Center,
                ),
        )
    }
}

// ── Score ring ──

@Composable
internal fun ScoreRing(score: Int, topMark: Int, modifier: Modifier = Modifier, size: Dp = 46.dp) {
    val clamped = score.coerceIn(1, topMark.coerceAtLeast(1))
    val fraction = clamped.toFloat() / topMark.coerceAtLeast(1)
    val line = maxOf(3.dp, size * RING_LINE)
    Box(
        modifier.size(size).clearAndSetSemantics {
            contentDescription = "Mark $clamped of $topMark"
        },
        contentAlignment = Alignment.Center,
    ) {
        Canvas(Modifier.fillMaxSize()) {
            val stroke = line.toPx()
            val inset = stroke / 2
            val arc = Size(this.size.width - stroke, this.size.height - stroke)
            drawArc(
                IntradaColor.masteryTrack,
                0f,
                FULL_TURN,
                false,
                Offset(inset, inset),
                arc,
                style = Stroke(stroke),
            )
            drawArc(
                IntradaColor.masteryFill,
                TWELVE_O_CLOCK,
                FULL_TURN * fraction,
                false,
                Offset(inset, inset),
                arc,
                style = Stroke(stroke, cap = StrokeCap.Round),
            )
        }
        BasicText(
            clamped.toString(),
            style =
                IntradaFont.scoreNumeral((size.value * NUMERAL).sp).copy(color = IntradaColor.ink),
        )
    }
}

private const val DASH = 5f
private const val RING_LINE = 0.09f
private const val NUMERAL = 0.36f
private const val FULL_TURN = 360f
private const val TWELVE_O_CLOCK = -90f
