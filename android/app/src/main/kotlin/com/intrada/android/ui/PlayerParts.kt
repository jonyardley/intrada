package com.intrada.android.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
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
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.ProgressBarRangeInfo
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.progressBarRangeInfo
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.bar
import com.intrada.shared.ItemKind

@Composable
internal fun InkButton(
    title: String,
    tag: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    spoken: String = title,
) {
    Box(
        modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .clip(RoundedCornerShape(IntradaRadius.control))
            .background(IntradaGradient.inkBar)
            .clickable(role = Role.Button, onClick = onClick)
            .clearAndSetSemantics {
                contentDescription = spoken
                role = Role.Button
                testTag = tag
                onClick {
                    onClick()
                    true
                }
            }
            .padding(vertical = IntradaSpacing.cardCompact),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(title, style = IntradaFont.button.copy(color = IntradaColor.onAccent))
    }
}

/** Tapping the mark already chosen clears it, as on the iPhone. */
@Composable
internal fun ScoreSelector(
    score: Int,
    range: IntRange,
    spoken: String,
    tag: String,
    onSelect: (UByte?) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier.fillMaxWidth().semantics { contentDescription = spoken },
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        range.forEach { value ->
            val chosen = value == score
            Box(
                Modifier.weight(1f)
                    .heightIn(min = 48.dp)
                    .clip(RoundedCornerShape(IntradaRadius.control))
                    .background(if (chosen) IntradaColor.ink else IntradaColor.surfaceSunken)
                    .clickable(role = Role.RadioButton) {
                        onSelect(if (chosen) null else value.toUByte())
                    }
                    .semantics {
                        contentDescription = "$spoken, $value"
                        selected = chosen
                    }
                    .testTag("$tag.$value"),
                contentAlignment = Alignment.Center,
            ) {
                BasicText(
                    "$value",
                    style =
                        IntradaFont.figure.copy(
                            color = if (chosen) IntradaColor.onAccent else IntradaColor.ink,
                            textAlign = TextAlign.Center,
                        ),
                )
            }
        }
    }
}

/** Pills that wrap; tapping a chosen one again is how the screen takes it back. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun <T> ChoicePills(
    options: List<T>,
    chosen: (T) -> Boolean,
    label: (T) -> String,
    tag: (T) -> String,
    onTap: (T) -> Unit,
    modifier: Modifier = Modifier,
) {
    FlowRow(
        modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        options.forEach { option ->
            val on = chosen(option)
            Box(
                Modifier.heightIn(min = 48.dp)
                    .clip(CircleShape)
                    .background(if (on) IntradaColor.ink else IntradaColor.surfaceSunken)
                    .clickable(role = Role.Checkbox) { onTap(option) }
                    .semantics { selected = on }
                    .testTag(tag(option))
                    .padding(horizontal = IntradaSpacing.card),
                contentAlignment = Alignment.Center,
            ) {
                BasicText(
                    label(option),
                    style =
                        IntradaFont.segment.copy(
                            color = if (on) IntradaColor.onAccent else IntradaColor.ink
                        ),
                )
            }
        }
    }
}

/**
 * Elapsed time centred in a still ring; the arc sweeps toward the planned time when there is one.
 */
@Composable
internal fun TimerRing(elapsed: Long, planned: Long?, modifier: Modifier = Modifier) {
    val fraction = planned?.takeIf { it > 0 }?.let { (elapsed.toFloat() / it).coerceAtMost(1f) }
    val spokenTime =
        SessionClock.clockDisplay(elapsed) +
            planned?.let { " of ${SessionClock.clockDisplay(it)}" }.orEmpty()
    Box(
        modifier.size(RING_SIZE).clearAndSetSemantics {
            contentDescription = "This item"
            stateDescription = spokenTime
            testTag = "player.timer"
        },
        contentAlignment = Alignment.Center,
    ) {
        Canvas(Modifier.fillMaxSize().padding(18.dp)) {
            val stroke = 10.dp.toPx()
            val inset = stroke / 2
            val arc = Size(size.width - stroke, size.height - stroke)
            drawArc(
                IntradaColor.timerTrack,
                0f,
                FULL_TURN,
                false,
                Offset(inset, inset),
                arc,
                style = Stroke(stroke),
            )
            if (fraction != null) {
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
        }
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            BasicText(
                SessionClock.clockDisplay(elapsed),
                style = IntradaFont.timer(48.sp).copy(color = IntradaColor.ink),
            )
            if (planned != null) {
                BasicText(
                    "of ${SessionClock.clockDisplay(planned)}",
                    style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
    }
}

private val RING_SIZE = 200.dp
private const val FULL_TURN = 360f
private const val TWELVE_O_CLOCK = -90f

class RepState(
    val count: Int,
    val slots: Int,
    val touched: Boolean,
    val reached: Boolean,
    val extra: Int,
    val canUndo: Boolean,
)

class RepActions(val onGotIt: () -> Unit, val onNotQuite: () -> Unit, val onUndo: () -> Unit)

@Composable
internal fun RepCounter(state: RepState, actions: RepActions, modifier: Modifier = Modifier) {
    val tail =
        when {
            state.extra > 0 -> " of ${state.slots} · ${state.extra} extra"
            state.touched && !state.reached ->
                " of ${state.slots} · ${state.slots - state.count} to go"
            else -> " of ${state.slots}"
        }
    Column(
        modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        Row(
            Modifier.fillMaxWidth().clearAndSetSemantics {
                contentDescription = "Repetitions"
                stateDescription = "${state.count}${tail.replace(" · ", ", ")}"
                testTag = "player.reps"
            },
            verticalAlignment = Alignment.CenterVertically,
        ) {
            FieldLabel("Repetitions")
            Box(Modifier.weight(1f))
            BasicText(
                "${state.count}",
                style = IntradaFont.figure.copy(color = IntradaColor.ink),
            )
            BasicText(tail, style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary))
        }
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        ) {
            RepButton(
                "Not quite",
                "player.notQuite",
                IntradaColor.repMissedFg,
                IntradaColor.repMissedBg,
                actions.onNotQuite,
                Modifier.weight(1f),
            )
            RepButton(
                "Got it",
                "player.gotIt",
                IntradaColor.repCleanFg,
                IntradaColor.repCleanBg,
                actions.onGotIt,
                Modifier.weight(1f),
            )
        }
        if (state.canUndo) TextAction("Undo", "player.undo", actions.onUndo)
    }
}

@Composable
private fun RepButton(
    title: String,
    tag: String,
    ink: Color,
    fill: Color,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier
            .heightIn(min = 48.dp)
            .clip(RoundedCornerShape(IntradaRadius.control))
            .background(fill)
            .border(1.dp, IntradaColor.hairline, RoundedCornerShape(IntradaRadius.control))
            .clickable(role = Role.Button, onClick = onClick)
            .testTag(tag)
            .padding(vertical = IntradaSpacing.cardCompact),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(title, style = IntradaFont.bodyMedium.copy(color = ink))
    }
}

/** Session so far, where the musician is, and one bar per item in the item's own colour. */
@Composable
internal fun OrientationBand(
    sessionElapsed: Long?,
    positionLabel: String,
    kinds: List<ItemKind>,
    filled: Int,
    modifier: Modifier = Modifier,
    menu: @Composable () -> Unit,
) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            BasicText(
                positionLabel,
                Modifier.testTag("player.position"),
                style = IntradaFont.label.copy(color = IntradaColor.inkSecondary),
            )
            Box(Modifier.weight(1f))
            if (sessionElapsed != null) {
                BasicText(
                    SessionClock.clockDisplay(sessionElapsed),
                    Modifier.clearAndSetSemantics {
                        contentDescription = "Session so far"
                        stateDescription = SessionClock.clockDisplay(sessionElapsed)
                    },
                    style = IntradaFont.figure.copy(color = IntradaColor.inkSecondary),
                )
            }
            menu()
        }
        Row(
            Modifier.fillMaxWidth().semantics {
                progressBarRangeInfo =
                    ProgressBarRangeInfo(filled.toFloat(), 0f..kinds.size.toFloat())
            },
            horizontalArrangement = Arrangement.spacedBy(3.dp),
        ) {
            kinds.forEachIndexed { index, kind ->
                Box(
                    Modifier.weight(1f)
                        .height(BAND_HEIGHT)
                        .clip(CircleShape)
                        .then(
                            if (index < filled) Modifier.background(kind.bar)
                            else Modifier.background(IntradaColor.timerTrack)
                        )
                )
            }
        }
    }
}

private val BAND_HEIGHT: Dp = 4.dp

@Composable
internal fun MenuButton(spoken: String, tag: String, onClick: () -> Unit) {
    IconAction(
        com.intrada.android.R.drawable.ic_ellipsis,
        spoken,
        onClick,
        Modifier.sizeIn(minWidth = 48.dp, minHeight = 48.dp).testTag(tag),
    )
}
