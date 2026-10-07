package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
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

/** Survives the reflection sheet, so the click keeps sounding while the musician scores. */
@Composable
internal fun rememberClick(model: PlayerModel): ClickController {
    val context = LocalContext.current
    val click = remember { ClickController(model.clickOutput ?: { ClickEngine(context) }) }
    DisposableEffect(model.active.currentPosition) {
        click.reseed(model.active, model.limits)
        onDispose {}
    }
    LifecycleEventEffect(Lifecycle.Event.ON_STOP) { click.enteredBackground() }
    LifecycleEventEffect(Lifecycle.Event.ON_START) { click.enteredForeground() }
    DisposableEffect(click) { onDispose { click.stop() } }
    return click
}

@Composable
internal fun PlayerClick(
    active: ActiveSessionView,
    click: ClickController,
    changed: () -> Boolean,
) {
    val target = click.soundsTarget
    ClickRow(
        ClickRowState(
            click.bpm,
            click.metre.unit,
            click.status,
            click.isAtSeededTempo,
            if (target) tempoDisplay(active.currentItemTempoMarking, active.currentItemTempoBpm)
            else null,
            if (target) tempoSpoken(active.currentItemTempoMarking, active.currentItemTempoBpm)
            else null,
        ),
        onToggle = {
            click.toggle()
            changed()
        },
        onStep = {
            click.step(it)
            changed()
        },
        step = click.step,
        modifier = Modifier.padding(top = IntradaSpacing.controlGap),
    )
}

/** The player's metronome. The steppers show only while it sounds (design principles T14). */
@Composable
internal fun ClickRow(
    state: ClickRowState,
    onToggle: () -> Unit,
    onStep: (Int) -> Unit,
    step: Int,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier,
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (state.isRunning) {
            IconAction(R.drawable.ic_minus, "Slower", { onStep(-step) }, tint = IntradaColor.accent)
        }
        ClickToggle(state, onToggle, onStep, step)
        if (state.isRunning) {
            IconAction(R.drawable.ic_plus, "Faster", { onStep(step) }, tint = IntradaColor.accent)
        }
    }
}

@Composable
private fun ClickToggle(
    state: ClickRowState,
    onToggle: () -> Unit,
    onStep: (Int) -> Unit,
    step: Int,
) {
    val tint =
        when {
            state.unavailable -> IntradaColor.danger
            state.isRunning -> IntradaColor.accent
            else -> IntradaColor.inkSecondary
        }
    Row(
        Modifier.heightIn(min = 48.dp)
            .clip(CircleShape)
            .background(if (state.isRunning) IntradaColor.marker else Color.Transparent)
            .clickable(role = Role.Button, onClick = onToggle)
            .clearAndSetSemantics {
                contentDescription =
                    if (state.isRunning) "Stop the metronome" else "Start the metronome"
                stateDescription = state.spoken
                role = Role.Button
                testTag = "click.toggle"
                onClick {
                    onToggle()
                    true
                }
                // The steppers are gone while it is stopped, so these are TalkBack's only way to
                // set the tempo (#1943).
                customActions =
                    listOf(
                        CustomAccessibilityAction("Faster") {
                            onStep(step)
                            true
                        },
                        CustomAccessibilityAction("Slower") {
                            onStep(-step)
                            true
                        },
                    )
            }
            .padding(horizontal = IntradaSpacing.cardCompact),
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
            state.readout,
            style = IntradaFont.bodyMedium.copy(color = tint),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
