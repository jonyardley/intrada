package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.layout.Placeable
import androidx.compose.ui.layout.SubcomposeLayout
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.min
import com.intrada.android.R
import com.intrada.android.ui.components.scaled
import com.intrada.shared.Metre

/**
 * The bar under the tempo row: the metre and one dot per beat, filled where the click sounds, with
 * a ring on the beat being heard. Filled and hollow, never colour alone (T3). Tapping it opens the
 * pattern sheet, so the readout's start and stop tap is never contested (T19).
 */
@Composable
internal fun ClickBarLine(
    metre: Metre,
    sounding: UShort,
    currentBeat: Int?,
    onTap: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier
            .heightIn(min = 48.dp)
            .clickable(role = Role.Button, onClick = onTap)
            .clearAndSetSemantics {
                contentDescription = "Metronome settings"
                stateDescription = spokenBar(metre, sounding)
                role = Role.Button
                testTag = "click.bar"
                onClick(label = "Choose the time signature and which beats sound") {
                    onTap()
                    true
                }
            },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // A twelve-beat bar outgrows a phone's row, so the line tightens its groups, then drops the
        // metre, before it overflows (#2139, #2540).
        FirstThatFits(BarFit.entries.size) {
            BarPill(metre, sounding, currentBeat, BarFit.entries[it])
        }
    }
}

private enum class BarFit {
    FULL,
    COMPACT,
    BARE,
}

@Composable
private fun BarPill(metre: Metre, sounding: UShort, currentBeat: Int?, fit: BarFit) {
    Row(
        Modifier.heightIn(min = 36.dp)
            .background(IntradaColor.surfaceSunken, CircleShape)
            .border(1.dp, IntradaColor.divider, CircleShape)
            .padding(horizontal = IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (fit != BarFit.BARE) {
            BasicText(
                metre.label,
                style = IntradaFont.label.copy(color = IntradaColor.inkSecondary),
                maxLines = 1,
            )
        }
        BeatDots(metre, sounding, currentBeat, fit == BarFit.FULL)
        if (fit == BarFit.FULL) {
            val height = DIVIDER * LocalDensity.current.fontScale
            Box(Modifier.width(1.dp).height(height).background(IntradaColor.divider))
        }
        Image(
            painterResource(R.drawable.ic_sliders),
            contentDescription = null,
            modifier = Modifier.size(IntradaIconSize.inline.scaled()),
            colorFilter = ColorFilter.tint(IntradaColor.ink),
        )
    }
}

@Composable
private fun BeatDots(metre: Metre, sounding: UShort, currentBeat: Int?, roomy: Boolean) {
    val beatGap = if (roomy) IntradaSpacing.controlGap else IntradaSpacing.controlGap / 2
    val groupGap = if (roomy) IntradaSpacing.cardCompact + beatGap else IntradaSpacing.cardCompact
    Row(horizontalArrangement = Arrangement.spacedBy(groupGap)) {
        groupRanges(metre).forEach { group ->
            Row(horizontalArrangement = Arrangement.spacedBy(beatGap)) {
                group.forEach { beat -> BeatDot(sounds(sounding, beat), beat == currentBeat) }
            }
        }
    }
}

/** Lays out the first of [count] versions whose natural width fits, the last when none does. */
@Composable
internal fun FirstThatFits(
    count: Int,
    modifier: Modifier = Modifier,
    content: @Composable (Int) -> Unit,
) {
    SubcomposeLayout(modifier) { constraints ->
        val natural = constraints.copy(minWidth = 0, maxWidth = Constraints.Infinity)
        var placeables = emptyList<Placeable>()
        for (index in 0 until count) {
            placeables = subcompose(index) { content(index) }.map { it.measure(natural) }
            if ((placeables.maxOfOrNull { it.width } ?: 0) <= constraints.maxWidth) break
        }
        val width =
            (placeables.maxOfOrNull { it.width } ?: 0).coerceIn(
                constraints.minWidth,
                constraints.maxWidth,
            )
        val height =
            (placeables.maxOfOrNull { it.height } ?: 0).coerceIn(
                constraints.minHeight,
                constraints.maxHeight,
            )
        layout(width, height) {
            placeables.forEach { it.placeRelative(0, (height - it.height) / 2) }
        }
    }
}

@Composable
private fun BeatDot(sounding: Boolean, current: Boolean) {
    // Scales with the font, capped so a twelve-beat bar still fits the row (#1950).
    val diameter: Dp = min(DOT * LocalDensity.current.fontScale, DOT_MAX)
    Box(
        Modifier.size(diameter + RING_GAP * 2)
            .border(1.5.dp, if (current) IntradaColor.ink else Color.Transparent, CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Box(
            Modifier.size(diameter)
                .background(if (sounding) IntradaColor.accent else Color.Transparent, CircleShape)
                .border(
                    1.5.dp,
                    if (sounding) Color.Transparent else IntradaColor.slotOutline,
                    CircleShape,
                )
        )
    }
}

/** One run per group, so 3 + 2 + 2 reads as the shape of the bar; one run when none is declared. */
internal fun groupRanges(metre: Metre): List<IntRange> {
    val groups = metre.groups?.map { it.toInt() } ?: listOf(metre.beats.toInt())
    var start = 0
    return groups.map { length -> (start until start + length).also { start += length } }
}

internal fun sounds(sounding: UShort, beat: Int): Boolean = sounding.toInt() and (1 shl beat) != 0

internal fun spokenBar(metre: Metre, sounding: UShort): String {
    val heard = (0 until metre.beats.toInt()).filter { sounds(sounding, it) }.map { "${it + 1}" }
    val sounded =
        when {
            heard.size == metre.beats.toInt() -> "every beat"
            heard.size == 1 -> "beat ${heard.first()}"
            else -> "beats ${heard.joinToString(", ")}"
        }
    return "${metre.beats} ${spokenUnit(metre.unit)} beats, metronome on $sounded"
}

private val DOT = 10.dp
private val DOT_MAX = 14.dp
private val RING_GAP = 3.dp
private val DIVIDER = 16.dp
