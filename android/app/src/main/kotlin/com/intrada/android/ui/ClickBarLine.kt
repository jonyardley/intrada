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
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.testTag
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
    // A twelve-beat bar outgrows a phone's row, so a long bar tightens its gaps (#2139).
    val roomy = metre.beats.toInt() <= ROOMY_BEATS
    Row(
        modifier
            .heightIn(min = 48.dp)
            .clickable(role = Role.Button, onClick = onTap)
            .clearAndSetSemantics {
                contentDescription = "Metronome settings"
                stateDescription = spokenBar(metre, sounding)
                role = Role.Button
                testTag = "click.bar"
                onClick {
                    onTap()
                    true
                }
            },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(
            Modifier.heightIn(min = 36.dp)
                .background(IntradaColor.surfaceSunken, CircleShape)
                .border(1.dp, IntradaColor.divider, CircleShape)
                .padding(horizontal = IntradaSpacing.cardCompact),
            horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(
                metre.label,
                style = IntradaFont.label.copy(color = IntradaColor.inkSecondary),
                maxLines = 1,
            )
            BeatDots(metre, sounding, currentBeat, roomy)
            if (roomy) {
                Box(Modifier.width(1.dp).height(16.dp).background(IntradaColor.divider))
            }
            Image(
                painterResource(R.drawable.ic_sliders),
                contentDescription = null,
                modifier = Modifier.size(IntradaIconSize.inline.scaled()),
                colorFilter = ColorFilter.tint(IntradaColor.ink),
            )
        }
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

private const val ROOMY_BEATS = 8
private val DOT = 10.dp
private val DOT_MAX = 14.dp
private val RING_GAP = 3.dp
