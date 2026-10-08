package com.intrada.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.testTag
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.intrada.android.R
import com.intrada.android.ui.components.FieldCard
import com.intrada.ffi.clickTempoWords
import com.intrada.shared.ClickPreset
import com.intrada.shared.LimitsView
import com.intrada.shared.Metre

/**
 * Two layers down from the tempo row (T19): the time signature, offered with the piece's answer
 * already in it, and which beats of the bar sound. Both hold for this session only.
 */
@Composable
internal fun ClickSheet(click: ClickController, limits: LimitsView, onClose: () -> Unit) {
    Dialog(
        onDismissRequest = onClose,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        ClickSheetPage(click, limits, onClose)
    }
}

@Composable
internal fun ClickSheetPage(
    click: ClickController,
    limits: LimitsView,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ScreenScaffold(
        "Metronome",
        modifier,
        actions = { IconAction(R.drawable.ic_close, "Done", onClose) },
    ) {
        Column(
            Modifier.fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
        ) {
            TempoNote(click)
            MetreSection(click, limits)
            SoundsOnSection(click)
        }
    }
}

@Composable
private fun TempoNote(click: ClickController) {
    val words = clickTempoWords(click.bpm.toUShort(), click.metre.unit)
    Column(
        Modifier.clearAndSetSemantics { contentDescription = "${words.spoken}. $TEMPO_NOTE" },
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        BasicText(words.text, style = IntradaFont.title.copy(color = IntradaColor.ink))
        BasicText(TEMPO_NOTE, style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary))
    }
}

@Composable
private fun MetreSection(click: ClickController, limits: LimitsView) {
    val metre = click.metre
    var other by remember { mutableStateOf(metre !in limits.clickMetrePresets) }
    FieldCard("Time signature") {
        SegmentedPills(
            options = limits.clickMetrePresets + listOf(null),
            selection = if (other) null else metre,
            onSelect = { chosen ->
                other = chosen == null
                if (chosen != null) click.bar.choose(chosen)
            },
            label = { it?.label ?: "Other" },
            tag = {
                it?.let { m -> "clickSheet.metre.${m.beats}-${m.unit}" } ?: "clickSheet.metre.other"
            },
        )
        BasicText(
            "From the piece. Changing it here holds for this session only.",
            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
        )
    }
    if (other) OtherSection(click, limits)
}

@Composable
private fun OtherSection(click: ClickController, limits: LimitsView) {
    val metre = click.metre
    val beats = metre.beats
    FieldCard("Beats in the bar") {
        StepperRow(
            "$beats",
            "beats",
            "clickSheet.beats",
            onStep = { click.bar.choose(Metre((beats.toInt() + it).toUByte(), metre.unit)) },
            canDecrease = beats > limits.metreBeatsMin,
            canIncrease = beats < limits.metreBeatsMax,
        )
    }
    FieldCard("Beat value") {
        SegmentedPills(
            options = limits.metreUnits,
            selection = metre.unit,
            onSelect = { click.bar.choose(metre.copy(unit = it)) },
            label = { "$it" },
            tag = { "clickSheet.unit.$it" },
            spoken = { "$it, ${spokenUnit(it)}" },
        )
    }
    val groupings = limits.clickGroupings(beats)
    if (groupings.isNotEmpty()) {
        FieldCard("Grouped") {
            SegmentedPills(
                options = listOf(null) + groupings,
                selection = metre.groups,
                onSelect = { click.bar.choose(metre.copy(groups = it)) },
                label = { it?.joinToString(" + ") ?: "Not grouped" },
                tag = { "clickSheet.grouping.${it?.joinToString("-") ?: "none"}" },
            )
        }
    }
}

@Composable
private fun SoundsOnSection(click: ClickController) {
    FieldCard("Sounds on") {
        // Nullable, so a hand-toggled pattern matching no preset simply selects nothing.
        SegmentedPills<ClickPreset?>(
            options = click.bar.presets.map { it.preset },
            selection = click.bar.matchingPreset,
            onSelect = { it?.let(click.bar::apply) },
            label = { it?.title.orEmpty() },
            tag = { "clickSheet.pattern.${it?.name?.lowercase()}" },
        )
        BeatGrid(click)
    }
}

/**
 * One tap per beat, in rows: the metre's own groups when it declares them, so 3 + 2 + 2 reads as
 * the shape of the bar, and otherwise halved so a twelve-beat bar still fits the sheet.
 */
@Composable
private fun BeatGrid(click: ClickController) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
        gridRows(click.metre).forEach { row ->
            Row(horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
                row.forEach { beat -> BeatToggle(beat, sounds(click.bar.sounding, beat), click) }
            }
        }
    }
}

@Composable
private fun BeatToggle(beat: Int, on: Boolean, click: ClickController) {
    val shape = RoundedCornerShape(IntradaRadius.control)
    Box(
        Modifier.size(48.dp)
            .background(if (on) IntradaColor.accent else IntradaColor.cardFill, shape)
            .border(1.dp, if (on) IntradaColor.accent else IntradaColor.hairline, shape)
            .clickable(role = Role.Button) { click.bar.toggleBeat(beat) }
            .clearAndSetSemantics {
                contentDescription = "Beat ${beat + 1}"
                stateDescription = if (on) "sounds" else "silent"
                role = Role.Button
                selected = on
                testTag = "clickSheet.beat.${beat + 1}"
                onClick {
                    click.bar.toggleBeat(beat)
                    true
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        BasicText(
            "${beat + 1}",
            style =
                IntradaFont.bodyMedium.copy(
                    color = if (on) IntradaColor.onAccent else IntradaColor.inkSecondary
                ),
        )
    }
}

/** Six 48dp cells and their gaps are the widest row that fits, so a longer bar is halved. */
internal fun gridRows(metre: Metre): List<IntRange> {
    val beats = metre.beats.toInt()
    val first = (beats + 1) / 2
    return when {
        beats <= GRID_ROW_MAX -> listOf(0 until beats)
        metre.groups != null -> groupRanges(metre)
        else -> listOf(0 until first, first until beats)
    }
}

private const val GRID_ROW_MAX = 6
private const val TEMPO_NOTE =
    "The tempo stays as it is. The pattern changes which beats you hear, not how fast you play."
