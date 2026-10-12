package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.scaled
import com.intrada.shared.DraftWay
import com.intrada.shared.FinishRowView
import com.intrada.shared.NotePointView
import com.intrada.shared.NoteSpan
import com.intrada.shared.SectionView

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
    val way = DraftWay(row.playId, row.sectionId, row.key, row.variationIds)
    Column(
        Modifier.fillMaxWidth()
            .cardSurface(IntradaRadius.control)
            .padding(IntradaSpacing.cardCompact),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        if (choices.sections.isNotEmpty()) {
            WayGroup("Section") {
                ChoicePills(
                    listOf<SectionView?>(null) + choices.sections,
                    chosen = { it?.id == row.sectionId },
                    label = { it?.label ?: "Whole piece" },
                    tag = { "reflection.waySection.${it?.id ?: "whole"}" },
                    onTap = { onChange(way.copy(sectionId = it?.id)) },
                )
            }
        }
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

// ── Points read from the note (#2307), kept with a tap ──

@Composable
internal fun ReflectionNoteOffers(
    state: ReflectionState,
    offers: List<NotePointView>,
    draft: () -> Unit,
) {
    if (offers.isEmpty()) return
    NoteOffers(
        offers,
        onTap = { span ->
            val kept = offers.filter { it.confirmed }.map { it.span }
            state.notePoints = if (span in kept) kept - span else kept + span
            draft()
        },
        Modifier.padding(top = IntradaSpacing.card),
    )
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun NoteOffers(
    offers: List<NotePointView>,
    onTap: (NoteSpan) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
        FieldLabel("From your note")
        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        ) {
            offers.forEach { NoteOfferChip(it) { onTap(it.span) } }
        }
    }
}

@Composable
private fun NoteOfferChip(offer: NotePointView, onTap: () -> Unit) {
    val kept = offer.confirmed
    Row(
        Modifier.heightIn(min = 48.dp)
            .clip(CircleShape)
            .background(if (kept) IntradaColor.marker else IntradaColor.cardFill)
            .border(1.dp, if (kept) IntradaColor.ink else IntradaColor.inkSecondary, CircleShape)
            .clickable(role = Role.Button, onClick = onTap)
            .clearAndSetSemantics {
                contentDescription =
                    listOfNotNull(offer.label, offer.sectionLabel).joinToString(", ")
                stateDescription = if (kept) "kept" else "not kept"
                selected = kept
                role = Role.Button
                testTag = "reflection.noteOffer"
                onClick {
                    onTap()
                    true
                }
            }
            .padding(horizontal = IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(
            painterResource(if (kept) R.drawable.ic_tick else R.drawable.ic_plus),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.caption.scaled()),
            colorFilter = ColorFilter.tint(IntradaColor.ink),
        )
        BasicText(offer.label, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
        offer.sectionLabel?.let {
            BasicText(
                it,
                style =
                    IntradaFont.small.copy(
                        color = if (kept) IntradaColor.ink else IntradaColor.inkSecondary
                    ),
            )
        }
    }
}
