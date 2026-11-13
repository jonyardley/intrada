package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.DraftWay
import com.intrada.shared.FinishRowView

// ── Change what a row says was played (#2249) ──

@Composable
internal fun ChangeWayAction(title: String, open: Boolean, onClick: () -> Unit) {
    Box(
        Modifier.sizeIn(minWidth = 48.dp, minHeight = 48.dp)
            .buttonSemantics(
                if (open) "Done changing $title" else "Change what you played on $title",
                "reflection.changeWay",
                onClick,
            )
            .padding(horizontal = IntradaSpacing.controlGap),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(
            if (open) "Done" else "Change",
            style = IntradaFont.bodyMedium.copy(color = IntradaColor.inkSecondary),
        )
    }
}

@Composable
internal fun PlayWayEditor(
    row: FinishRowView,
    choices: PlayWayChoices,
    onChange: (DraftWay) -> Unit,
) {
    val way = DraftWay(row.playId, row.key, row.variationIds)
    Column(
        Modifier.fillMaxWidth()
            .cardSurface(IntradaRadius.control)
            .padding(IntradaSpacing.cardCompact),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        if (choices.keys.isNotEmpty()) {
            WayGroup("Key") {
                ChoicePills(
                    choices.keys,
                    chosen = { it.key == row.key },
                    label = { it.label },
                    tag = { "reflection.wayKey.${it.label}" },
                    onTap = { onChange(way.copy(key = it.key)) },
                )
            }
        }
        if (choices.variations.isNotEmpty()) {
            WayGroup("Variations") {
                ChoicePills(
                    choices.variations,
                    chosen = { it.id in row.variationIds },
                    label = { it.label },
                    tag = { "reflection.wayVariation.${it.id}" },
                    onTap = { tapped ->
                        val ids = row.variationIds
                        onChange(
                            way.copy(
                                variationIds =
                                    if (tapped.id in ids) ids - tapped.id else ids + tapped.id
                            )
                        )
                    },
                )
            }
        }
        choices.planned?.let {
            BasicText(
                "Planned: $it",
                style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}

@Composable
private fun WayGroup(title: String, pills: @Composable () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
        FieldLabel(title)
        pills()
    }
}
