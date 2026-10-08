package com.intrada.android.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.disabled
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.semantics.toggleableState
import androidx.compose.ui.state.ToggleableState
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.intrada.android.R
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.scaled
import com.intrada.shared.Event

fun Store.sendAccepted(event: Event): Boolean {
    val before = viewModel.value?.errorSeq
    send(event)
    val after = viewModel.value
    return before != null && after != null && !halted.value && after.errorSeq == before
}

@Composable
internal fun TextAction(
    text: String,
    tag: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    emphasised: Boolean = false,
    enabled: Boolean = true,
) {
    val ink = if (enabled) IntradaColor.accent else IntradaColor.inkFainter
    Box(
        modifier
            .sizeIn(minWidth = 48.dp, minHeight = 48.dp)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .testTag(tag)
            .padding(horizontal = IntradaSpacing.controlGap),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(
            text,
            style = (if (emphasised) IntradaFont.button else IntradaFont.label).copy(color = ink),
        )
    }
}

class HeaderAction(
    val title: String,
    val spoken: String,
    val tag: String,
    val enabled: Boolean = true,
    val perform: () -> Unit,
)

@Composable
internal fun SectionHeader(
    title: String,
    modifier: Modifier = Modifier,
    caption: String? = null,
    action: HeaderAction? = null,
) {
    Row(modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        FieldLabel(title)
        if (caption != null) {
            BasicText(
                caption,
                Modifier.padding(start = IntradaSpacing.controlGap).clearAndSetSemantics {},
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        }
        Box(Modifier.weight(1f))
        if (action != null) {
            TextAction(
                action.title,
                action.tag,
                action.perform,
                Modifier.semantics { contentDescription = action.spoken },
                enabled = action.enabled,
            )
        }
    }
}

@Composable
internal fun AddRow(
    title: String,
    spoken: String,
    tag: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    hint: String? = null,
) {
    Column(
        modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
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
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(3.dp),
    ) {
        BasicText("+ $title", style = IntradaFont.bodyMedium.copy(color = IntradaColor.accent))
        if (hint != null) {
            BasicText(hint, style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary))
        }
    }
}

@Composable
internal fun DeleteButton(
    title: String,
    tag: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .cardSurface()
            .clickable(role = Role.Button, onClick = onClick)
            .testTag(tag)
            .padding(vertical = IntradaSpacing.cardCompact),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(title, style = IntradaFont.bodyMedium.copy(color = IntradaColor.danger))
    }
}

@Composable
internal fun IconAction(
    @DrawableRes icon: Int,
    spoken: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    tint: Color = IntradaColor.inkSecondary,
    enabled: Boolean = true,
) {
    Box(
        modifier
            .size(48.dp)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .clearAndSetSemantics {
                contentDescription = spoken
                role = Role.Button
                if (!enabled) disabled()
                onClick {
                    if (enabled) onClick()
                    enabled
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        Image(
            painterResource(icon),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.control.scaled()),
            colorFilter = ColorFilter.tint(if (enabled) tint else IntradaColor.inkFainter),
        )
    }
}

@Composable
internal fun Chevron(modifier: Modifier = Modifier) {
    Image(
        painterResource(R.drawable.ic_chevron_right),
        contentDescription = null,
        modifier = modifier.size(IntradaIconSize.inline.scaled()),
        colorFilter = ColorFilter.tint(IntradaColor.inkFaintIcon),
    )
}

@Composable
internal fun TickRow(
    label: String,
    caption: String?,
    chosen: Boolean,
    tag: String,
    onToggle: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .clickable(role = Role.Checkbox, onClick = onToggle)
            .clearAndSetSemantics {
                contentDescription = caption?.let { "$label, $it" } ?: label
                role = Role.Checkbox
                toggleableState = ToggleableState(chosen)
                testTag = tag
                onClick {
                    onToggle()
                    true
                }
            }
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        TickMark(chosen)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            BasicText(label, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
            if (caption != null) {
                BasicText(
                    caption,
                    style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
    }
}

private const val TICK_SCALE = 0.7f

@Composable
private fun TickMark(chosen: Boolean) {
    val size = IntradaIconSize.control.scaled()
    if (chosen) {
        Box(
            Modifier.size(size).background(IntradaColor.accent, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Image(
                painterResource(R.drawable.ic_tick),
                contentDescription = null,
                modifier = Modifier.size(size * TICK_SCALE),
                colorFilter = ColorFilter.tint(IntradaColor.onAccent),
            )
        }
    } else {
        Image(
            painterResource(R.drawable.ic_circle),
            contentDescription = null,
            modifier = Modifier.size(size),
            colorFilter = ColorFilter.tint(IntradaColor.inkFaintIcon),
        )
    }
}

@Composable
internal fun <T> SegmentedPills(
    options: List<T>,
    selection: T,
    onSelect: (T) -> Unit,
    label: (T) -> String,
    tag: (T) -> String,
    modifier: Modifier = Modifier,
    spoken: ((T) -> String)? = null,
) {
    val track = RoundedCornerShape(IntradaRadius.control)
    Row(
        modifier
            .fillMaxWidth()
            .background(IntradaColor.surfaceSunken, track)
            .padding(IntradaSpacing.controlGap / 2)
    ) {
        options.forEach { option ->
            val chosen = option == selection
            Box(
                Modifier.weight(1f)
                    .heightIn(min = 48.dp)
                    .background(
                        if (chosen) IntradaColor.cardFill else IntradaColor.surfaceSunken,
                        track,
                    )
                    .clickable(role = Role.Tab) { onSelect(option) }
                    .semantics {
                        role = Role.Tab
                        selected = chosen
                        spoken?.let { contentDescription = it(option) }
                    }
                    .testTag(tag(option)),
                contentAlignment = Alignment.Center,
            ) {
                BasicText(
                    label(option),
                    style =
                        IntradaFont.segment.copy(
                            color = if (chosen) IntradaColor.ink else IntradaColor.inkSecondary,
                            textAlign = TextAlign.Center,
                        ),
                )
            }
        }
    }
}

class ConfirmCopy(val title: String, val message: String?, val confirm: String, val tag: String)

@Composable
internal fun ConfirmDialog(copy: ConfirmCopy, onConfirm: () -> Unit, onDismiss: () -> Unit) {
    Dialog(onDismissRequest = onDismiss) {
        Column(
            Modifier.fillMaxWidth().cardSurface().padding(top = IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        ) {
            BasicText(
                copy.title,
                Modifier.fillMaxWidth().padding(horizontal = IntradaSpacing.card),
                style = IntradaFont.cardTitle.copy(color = IntradaColor.ink),
            )
            if (copy.message != null) {
                BasicText(
                    copy.message,
                    Modifier.fillMaxWidth().padding(horizontal = IntradaSpacing.card),
                    style = IntradaFont.body.copy(color = IntradaColor.inkSecondary),
                )
            }
            HairlineDivider(Modifier.padding(top = IntradaSpacing.controlGap))
            Row(
                Modifier.fillMaxWidth().padding(horizontal = IntradaSpacing.controlGap),
                horizontalArrangement = Arrangement.End,
            ) {
                TextAction("Cancel", "${copy.tag}.cancel", onDismiss)
                Box(
                    Modifier.sizeIn(minWidth = 48.dp, minHeight = 48.dp)
                        .clickable(role = Role.Button, onClick = onConfirm)
                        .testTag("${copy.tag}.confirm")
                        .padding(horizontal = IntradaSpacing.controlGap),
                    contentAlignment = Alignment.Center,
                ) {
                    BasicText(
                        copy.confirm,
                        style = IntradaFont.button.copy(color = IntradaColor.danger),
                    )
                }
            }
        }
    }
}
