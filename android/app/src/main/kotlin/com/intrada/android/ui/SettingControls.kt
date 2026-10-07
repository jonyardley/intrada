package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.semantics.toggleableState
import androidx.compose.ui.state.ToggleableState
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.FieldLabel

@Composable
internal fun ToggleRow(
    title: String,
    on: Boolean,
    tag: String,
    onChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .clickable(role = Role.Switch) { onChange(!on) }
            .clearAndSetSemantics {
                contentDescription = title
                role = Role.Switch
                toggleableState = ToggleableState(on)
                testTag = tag
                onClick {
                    onChange(!on)
                    true
                }
            },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        FieldLabel(title, Modifier.weight(1f))
        SwitchTrack(on)
    }
}

@Composable
private fun SwitchTrack(on: Boolean) {
    Box(
        Modifier.size(width = 48.dp, height = 28.dp)
            .background(if (on) IntradaColor.accent else IntradaColor.surfaceSunken, CircleShape)
            .padding(3.dp),
        contentAlignment = if (on) Alignment.CenterEnd else Alignment.CenterStart,
    ) {
        Box(Modifier.size(22.dp).background(IntradaColor.cardFill, CircleShape))
    }
}

/** Plus and minus beside a value the core reports, so a refused step never shows (#1736). */
@Composable
internal fun StepperRow(
    value: String,
    spoken: String,
    tag: String,
    onStep: (Int) -> Unit,
    modifier: Modifier = Modifier,
    canDecrease: Boolean = true,
    canIncrease: Boolean = true,
) {
    Row(
        modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        BasicText(
            value,
            Modifier.weight(1f),
            style = IntradaFont.body.copy(color = IntradaColor.ink),
        )
        IconAction(
            R.drawable.ic_minus,
            "Less $spoken",
            onClick = { onStep(-1) },
            modifier = Modifier.semantics { testTag = "$tag.decrease" },
            enabled = canDecrease,
        )
        IconAction(
            R.drawable.ic_plus,
            "More $spoken",
            onClick = { onStep(1) },
            modifier = Modifier.semantics { testTag = "$tag.increase" },
            enabled = canIncrease,
        )
    }
}
