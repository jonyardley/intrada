package com.intrada.android.ui

import android.animation.ValueAnimator
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.awaitVerticalTouchSlopOrCancellation
import androidx.compose.foundation.gestures.verticalDrag
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.input.pointer.PointerInputChange
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import com.intrada.android.R
import com.intrada.android.core.ClickEngine
import com.intrada.android.ui.components.scaled
import com.intrada.shared.ActiveSessionView
import com.intrada.shared.LimitsView

/** Survives the reflection sheet, so the click keeps sounding while the musician scores. */
@Composable
internal fun rememberClick(model: PlayerModel): ClickController {
    val context = LocalContext.current
    val click = model.click ?: remember { ClickController { ClickEngine(context) } }
    DisposableEffect(model.active.currentPosition) {
        click.follow(model.active, model.limits)
        onDispose {}
    }
    LifecycleEventEffect(Lifecycle.Event.ON_STOP) { click.enteredBackground() }
    LifecycleEventEffect(Lifecycle.Event.ON_START) { click.enteredForeground() }
    return click
}

@Composable
internal fun PlayerClick(
    active: ActiveSessionView,
    limits: LimitsView,
    click: ClickController,
    changed: () -> Boolean,
) {
    var configuring by remember { mutableStateOf(false) }
    val target = click.soundsTarget
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        ClickRow(
            ClickRowState(
                click.bpm,
                click.metre.unit,
                click.status,
                click.isAtSeededTempo,
                if (target) active.currentItemTempoLine else null,
                if (target) active.currentItemTempoLineSpoken else null,
            ),
            ClickActions(
                onToggle = {
                    click.toggle()
                    changed()
                },
                onStep = {
                    click.step(it)
                    changed()
                },
                step = click.step,
                band = click.band,
                onDrag = {
                    click.dragTo(it)
                    changed()
                },
            ),
            Modifier.padding(top = IntradaSpacing.controlGap),
        )
        if (click.isRunning) {
            ClickBarLine(
                click.metre,
                click.bar.sounding,
                rememberCurrentBeat(click),
                onTap = { configuring = true },
                modifier = Modifier.padding(top = IntradaSpacing.controlGap),
            )
        }
    }
    if (configuring) {
        ClickSheet(
            click,
            limits,
            onClose = {
                configuring = false
                changed()
            },
        )
    }
}

/**
 * Polled each frame from the audio's own play position, so the ring cannot drift against the click
 * (T19). None when animations are off.
 */
@Composable
private fun rememberCurrentBeat(click: ClickController): Int? {
    val tracks = click.tracksBeat && ValueAnimator.areAnimatorsEnabled()
    val beat by
        produceState<Int?>(null, tracks) {
            value = null
            while (tracks) {
                withFrameNanos {}
                value = click.currentBeat()
            }
        }
    return beat
}

class ClickActions(
    val onToggle: () -> Unit,
    val onStep: (Int) -> Unit,
    val step: Int,
    val band: IntRange,
    val onDrag: (Int) -> Unit,
)

/** The player's metronome. The steppers show only while it sounds (design principles T14). */
@Composable
internal fun ClickRow(state: ClickRowState, actions: ClickActions, modifier: Modifier = Modifier) {
    Row(
        modifier,
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        val step = actions.step
        if (state.isRunning) {
            IconAction(
                R.drawable.ic_minus,
                "Slower",
                { actions.onStep(-step) },
                tint = IntradaColor.accent,
            )
        }
        ClickToggle(state, actions)
        if (state.isRunning) {
            IconAction(
                R.drawable.ic_plus,
                "Faster",
                { actions.onStep(step) },
                tint = IntradaColor.accent,
            )
        }
    }
}

@Composable
private fun ClickToggle(state: ClickRowState, actions: ClickActions) {
    var drag by remember { mutableStateOf<TempoDrag?>(null) }
    val anchor by rememberUpdatedState(state.bpm)
    val latest by rememberUpdatedState(actions)
    val dragging = drag != null
    val shown = drag?.let { state.copy(bpm = it.live, atSeededTempo = false) } ?: state
    val tint =
        when {
            dragging -> IntradaColor.accent
            state.unavailable -> IntradaColor.danger
            state.isRunning -> IntradaColor.accent
            else -> IntradaColor.inkSecondary
        }
    val fill =
        when {
            dragging -> IntradaColor.accent.copy(alpha = IntradaOpacity.wash)
            state.isRunning -> IntradaColor.marker
            else -> Color.Transparent
        }
    ToggleReadout(
        shown.readout,
        tint,
        dragging,
        Modifier.heightIn(min = 48.dp)
            .tempoDrag({ TempoDrag(anchor, latest.step, latest.band, latest.onDrag) }) { drag = it }
            .scale(if (dragging) DRAG_SCALE else 1f)
            .clip(CircleShape)
            .background(fill)
            .clickable(role = Role.Button, onClick = actions.onToggle)
            .toggleSemantics(state, actions)
            .padding(horizontal = IntradaSpacing.cardCompact),
    )
}

@Composable
private fun ToggleReadout(
    readout: String,
    tint: Color,
    dragging: Boolean,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier,
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap / 2),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(
            painterResource(R.drawable.ic_metronome),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.inline.scaled()),
            colorFilter = ColorFilter.tint(tint),
        )
        BasicText(
            readout,
            style =
                (if (dragging) IntradaFont.title else IntradaFont.bodyMedium).copy(color = tint),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        // The passive hint that this is a drag target, with no first-use tooltip (#1823).
        if (!dragging) {
            Image(
                painterResource(R.drawable.ic_arrows_vertical),
                contentDescription = null,
                modifier = Modifier.size(IntradaIconSize.caption.scaled()),
                colorFilter = ColorFilter.tint(IntradaColor.inkFaintIcon),
            )
        }
    }
}

/**
 * [begin] and [show] are read once, so they must read current state rather than capture it. Travel
 * runs from the finger-down point, as iOS's translation does, so the touch slop is not lost.
 */
private fun Modifier.tempoDrag(begin: () -> TempoDrag, show: (TempoDrag?) -> Unit) =
    pointerInput(Unit) {
        awaitEachGesture {
            val down = awaitFirstDown(requireUnconsumed = false)
            val travel = { change: PointerInputChange ->
                (change.position.y - down.position.y).toDp()
            }
            val crossed =
                awaitVerticalTouchSlopOrCancellation(down.id) { change, _ -> change.consume() }
                    ?: return@awaitEachGesture
            val drag = begin().also(show)
            drag.moved(travel(crossed).value)
            verticalDrag(crossed.id) { change ->
                change.consume()
                drag.moved(travel(change).value)
            }
            drag.ended()
            show(null)
        }
    }

private fun Modifier.toggleSemantics(state: ClickRowState, actions: ClickActions) =
    clearAndSetSemantics {
        contentDescription = if (state.isRunning) "Stop the metronome" else "Start the metronome"
        stateDescription = state.spoken
        role = Role.Button
        testTag = "click.toggle"
        onClick {
            actions.onToggle()
            true
        }
        // The steppers are gone while it is stopped and a drag is out of TalkBack's reach, so
        // these are its only way to set the tempo (#1943).
        customActions =
            listOf(
                CustomAccessibilityAction("Faster") {
                    actions.onStep(actions.step)
                    true
                },
                CustomAccessibilityAction("Slower") {
                    actions.onStep(-actions.step)
                    true
                },
            )
    }

private const val DRAG_SCALE = 1.04f
