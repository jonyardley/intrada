package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.FocusChoiceView
import com.intrada.shared.FocusTargetView
import com.intrada.shared.FocusView
import com.intrada.shared.IntentionFocus
import com.intrada.shared.SectionView

/** What the aim checks, where, and against which number (#2303); the core validates the target. */
@Composable
internal fun EntryFocusCard(
    current: FocusView?,
    choices: List<FocusChoiceView>,
    sections: List<SectionView>,
    send: (IntentionFocus?) -> Unit,
) {
    val focus = current?.focus
    Column(
        Modifier.fillMaxWidth()
            .cardSurface()
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.controlGap),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            FieldLabel("Focus", Modifier.weight(1f))
            if (focus != null) TextAction("Clear", "entrySettings.focusClear", { send(null) })
        }
        ChoicePills(
            choices,
            chosen = { it.kind == focus?.kind },
            label = { it.label },
            tag = { "entrySettings.focusKind" },
            onTap = { choice ->
                if (choice.kind != focus?.kind) send(IntentionFocus(choice.kind, focus?.sectionId))
            },
        )
        if (focus != null) {
            FocusDetail(focus, current.targetCaption, choices, sections, send)
        }
    }
}

@Composable
private fun FocusDetail(
    focus: IntentionFocus,
    caption: String?,
    choices: List<FocusChoiceView>,
    sections: List<SectionView>,
    send: (IntentionFocus?) -> Unit,
) {
    if (sections.isNotEmpty()) {
        FieldLabel("Where")
        ChoicePills(
            listOf<SectionView?>(null) + sections,
            chosen = { it?.id == focus.sectionId },
            label = { it?.label ?: "Whole piece" },
            tag = { "entrySettings.focusWhere" },
            onTap = { section ->
                if (section?.id != focus.sectionId) send(focus.copy(sectionId = section?.id))
            },
        )
    }
    choices
        .firstOrNull { it.kind == focus.kind }
        ?.target
        ?.let { range -> TargetStepper(focus, caption, range, send) }
}

@Composable
private fun TargetStepper(
    focus: IntentionFocus,
    caption: String?,
    range: FocusTargetView,
    send: (IntentionFocus?) -> Unit,
) {
    val value = (focus.target ?: range.min).toInt()
    Column {
        FieldLabel("Target")
        StepperRow(
            caption.orEmpty(),
            "target",
            "entrySettings.focusTarget",
            onStep = { direction ->
                val next = value + direction * range.step.toInt()
                send(
                    focus.copy(
                        target = next.coerceIn(range.min.toInt(), range.max.toInt()).toUShort()
                    )
                )
            },
            canDecrease = value > range.min.toInt(),
            canIncrease = value < range.max.toInt(),
        )
    }
}
