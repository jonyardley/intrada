package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.min
import com.intrada.android.R
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.scaled
import com.intrada.shared.FirstRunView

private class StartHereStep(val title: String, val done: Boolean, val action: () -> Unit)

private enum class StepState {
    DONE,
    CURRENT,
    LATER,
}

// The ticks are the core's, read from the library and the history (#2118), so the card never
// disagrees with them.
@Composable
internal fun StartHereCard(
    progress: FirstRunView,
    marker: Color,
    onAdd: () -> Unit,
    onStartSession: () -> Unit,
) {
    val steps =
        listOf(
            StartHereStep(
                progress.firstItemTitle?.let { "Added $it" } ?: "Add a piece or exercise",
                progress.added,
                onAdd,
            ),
            StartHereStep("Build a session", progress.built, onStartSession),
            StartHereStep("Play it through", progress.played, onStartSession),
            StartHereStep("Mark how it went", progress.marked, onStartSession),
        )
    val current = steps.indexOfFirst { !it.done }
    Column(Modifier.fillMaxWidth().cardSurface().testTag("practice.startHere")) {
        SectionHeader(
            "Start here",
            Modifier.padding(
                    start = IntradaSpacing.card,
                    end = IntradaSpacing.card,
                    top = IntradaSpacing.card,
                    bottom = IntradaSpacing.controlGap,
                )
                .clearAndSetSemantics {
                    contentDescription = "Start here, ${steps.count { it.done }} of ${steps.size}"
                },
            trailing = "${steps.count { it.done }} of ${steps.size}",
        )
        steps.forEachIndexed { index, step ->
            if (index > 0) HairlineDivider()
            val state =
                when {
                    step.done -> StepState.DONE
                    index == current -> StepState.CURRENT
                    else -> StepState.LATER
                }
            StepRow(step, state, marker)
        }
    }
}

@Composable
private fun StepRow(step: StartHereStep, state: StepState, marker: Color) {
    val semantics =
        if (state == StepState.CURRENT) {
            Modifier.clickable(role = Role.Button, onClick = step.action).clearAndSetSemantics {
                contentDescription = "${step.title}, next step"
                role = Role.Button
                testTag = "practice.startHere.next"
                onClick {
                    step.action()
                    true
                }
            }
        } else {
            Modifier.clearAndSetSemantics {
                contentDescription =
                    if (state == StepState.DONE) "${step.title}, done" else step.title
            }
        }
    Row(
        Modifier.fillMaxWidth()
            .heightIn(min = 48.dp)
            .then(semantics)
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Tick(state, marker)
        BasicText(
            step.title,
            Modifier.weight(1f),
            style =
                (if (state == StepState.CURRENT) IntradaFont.bodyMedium else IntradaFont.body).copy(
                    color =
                        if (state == StepState.LATER) IntradaColor.inkSecondary
                        else IntradaColor.ink
                ),
        )
        if (state == StepState.CURRENT) Chevron()
    }
}

@Composable
private fun Tick(state: StepState, marker: Color) {
    val tickSize = min(24.dp * LocalDensity.current.fontScale, 36.dp)
    when (state) {
        StepState.DONE ->
            Box(
                Modifier.size(tickSize).background(marker, CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                Image(
                    painterResource(R.drawable.ic_tick),
                    contentDescription = null,
                    modifier = Modifier.size(IntradaIconSize.caption.scaled()),
                    colorFilter = ColorFilter.tint(IntradaColor.onMarker),
                )
            }
        StepState.CURRENT ->
            Box(Modifier.size(tickSize).border(1.5.dp, IntradaColor.ink, CircleShape))
        StepState.LATER ->
            Box(Modifier.size(tickSize).border(1.5.dp, IntradaColor.divider, CircleShape))
    }
}
