package com.intrada.android.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.ui.components.FieldCard
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.InstrumentGlyph
import com.intrada.android.ui.components.accessibilityLabel
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.scaled
import com.intrada.shared.ClickStart
import com.intrada.shared.HighlighterColour
import com.intrada.shared.InstrumentIcon
import com.intrada.shared.LimitsView
import com.intrada.shared.PracticeDefaults
import com.intrada.shared.ProfileField

private const val GRID_COLUMNS = 4
private const val MAX_SUGGESTIONS = 6

// The token sheet's order, butter first; the core's enum order is its wire order, not this.
private val SWATCH_ORDER =
    listOf(
        HighlighterColour.BUTTER,
        HighlighterColour.CORAL,
        HighlighterColour.MINT,
        HighlighterColour.SKY,
        HighlighterColour.LAVENDER,
        HighlighterColour.SAGE,
        HighlighterColour.PEACH,
        HighlighterColour.POWDER,
    )

@Composable
internal fun ProfileNameFields(form: ProfileEditState, instrumentNames: List<String>) {
    val faulted = form.refusal?.field
    val focusManager = LocalFocusManager.current
    var instrumentFocused by remember { mutableStateOf(false) }
    val query = form.instrument.trim()
    val matches =
        if (!instrumentFocused) emptyList()
        else
            instrumentNames
                .filter {
                    it.contains(query, ignoreCase = true) && !it.equals(query, ignoreCase = true)
                }
                .take(MAX_SUGGESTIONS)
    Column(Modifier.cardSurface()) {
        FormField(
            "Name",
            form.name,
            { form.name = it },
            "profileEdit.name",
            placeholder = "Your name",
            capitalization = KeyboardCapitalization.Words,
            faulted = faulted == ProfileField.NAME,
        )
        HairlineDivider()
        FormField(
            "Instrument",
            form.instrument,
            { form.instrument = it },
            "profileEdit.instrument",
            Modifier.onFocusChanged { instrumentFocused = it.hasFocus },
            placeholder = "e.g. Cello",
            capitalization = KeyboardCapitalization.Words,
            faulted = faulted == ProfileField.INSTRUMENT,
        )
        matches.forEach { match ->
            HairlineDivider(Modifier.padding(start = IntradaSpacing.card))
            BasicText(
                match,
                Modifier.fillMaxWidth()
                    .heightIn(min = 48.dp)
                    .clickable(onClickLabel = "Fills Instrument with $match", role = Role.Button) {
                        form.instrument = match
                        focusManager.clearFocus()
                    }
                    .testTag("suggestion.row")
                    .padding(
                        horizontal = IntradaSpacing.card,
                        vertical = IntradaSpacing.cardCompact,
                    ),
                style = IntradaFont.body.copy(color = IntradaColor.ink),
            )
        }
    }
}

@Composable
internal fun HighlighterSwatches(colour: HighlighterColour, onPick: (HighlighterColour) -> Unit) {
    FieldCard("Highlighter") {
        Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
            SWATCH_ORDER.chunked(GRID_COLUMNS).forEach { row ->
                Row(Modifier.fillMaxWidth()) {
                    row.forEach { swatch ->
                        Swatch(swatch, swatch == colour, { onPick(swatch) }, Modifier.weight(1f))
                    }
                }
            }
        }
    }
}

@Composable
private fun Swatch(
    swatch: HighlighterColour,
    selected: Boolean,
    onPick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier
            .heightIn(min = 48.dp)
            .selectable(selected, role = Role.RadioButton, onClick = onPick)
            .semantics(mergeDescendants = true) { contentDescription = swatch.label }
            .testTag("profileEdit.highlighter.${swatch.name.lowercase()}"),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Box(
            Modifier.size(40.dp).background(IntradaColor.marker(swatch), CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            if (selected) {
                Image(
                    painterResource(R.drawable.ic_tick),
                    contentDescription = null,
                    modifier = Modifier.size(IntradaIconSize.inline.scaled()),
                    colorFilter = ColorFilter.tint(IntradaColor.onMarker),
                )
            }
        }
        BasicText(
            swatch.label,
            Modifier.clearAndSetSemantics {},
            style =
                IntradaFont.smallMedium.copy(
                    color = if (selected) IntradaColor.ink else IntradaColor.inkSecondary
                ),
        )
    }
}

@Composable
internal fun InstrumentIconPicker(
    suggested: InstrumentIcon,
    choice: InstrumentIcon?,
    marker: Color,
    onChoose: (InstrumentIcon) -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val shown = choice ?: suggested
    ScreenScaffold(
        "Icon",
        modifier,
        actions = { TextAction("Done", "iconPicker.done", onDone, emphasised = true) },
    ) {
        Column(
            Modifier.verticalScroll(rememberScrollState())
                .padding(horizontal = IntradaSpacing.card)
                .padding(top = IntradaSpacing.controlGap, bottom = IntradaSpacing.section),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
        ) {
            if (suggested != InstrumentIcon.OTHER) {
                SectionHeader("Matches ${suggested.tileLabel.lowercase()}")
                IconGrid(listOf(suggested), shown, marker, onChoose)
                Spacer(Modifier.size(IntradaSpacing.cardCompact))
            }
            SectionHeader("All icons")
            IconGrid(
                InstrumentIcon.entries.filter {
                    suggested == InstrumentIcon.OTHER || it != suggested
                },
                shown,
                marker,
                onChoose,
            )
        }
    }
}

@Composable
private fun IconGrid(
    icons: List<InstrumentIcon>,
    shown: InstrumentIcon,
    marker: Color,
    onPick: (InstrumentIcon) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
        icons.chunked(GRID_COLUMNS).forEach { row ->
            Row(
                Modifier.height(IntrinsicSize.Min),
                horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
            ) {
                row.forEach { icon ->
                    IconTile(
                        icon,
                        icon == shown,
                        marker,
                        { onPick(icon) },
                        Modifier.weight(1f).fillMaxHeight(),
                    )
                }
                repeat(GRID_COLUMNS - row.size) { Spacer(Modifier.weight(1f)) }
            }
        }
    }
}

@Composable
private fun IconTile(
    icon: InstrumentIcon,
    selected: Boolean,
    marker: Color,
    onPick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val shape = RoundedCornerShape(IntradaRadius.card)
    Column(
        modifier
            .heightIn(min = 84.dp)
            .clip(shape)
            .background(if (selected) marker else IntradaColor.cardFill)
            .border(1.dp, if (selected) IntradaColor.ink else IntradaColor.hairline, shape)
            .selectable(selected, role = Role.RadioButton, onClick = onPick)
            .clearAndSetSemantics { contentDescription = icon.accessibilityLabel }
            .padding(horizontal = 4.dp, vertical = IntradaSpacing.controlGap),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterVertically),
    ) {
        InstrumentGlyph(icon, size = IntradaGlyph.bar)
        BasicText(
            icon.tileLabel,
            style =
                IntradaFont.smallMedium.copy(
                    color = if (selected) IntradaColor.ink else IntradaColor.inkSecondary,
                    textAlign = TextAlign.Center,
                ),
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
internal fun PracticeDefaultsSection(
    defaults: PracticeDefaults,
    limits: LimitsView,
    onSave: (PracticeDefaults) -> Unit,
) {
    val save = { next: PracticeDefaults -> if (next != defaults) onSave(next) }
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
        SectionHeader("Practice defaults")
        Column(Modifier.fillMaxWidth().cardSurface()) {
            Column(
                Modifier.padding(
                    horizontal = IntradaSpacing.card,
                    vertical = IntradaSpacing.controlGap,
                )
            ) {
                FieldLabel("Repetitions")
                StepperRow(
                    "${defaults.repTarget} per item",
                    "repetitions",
                    "profile.repTarget",
                    onStep = {
                        save(defaults.copy(repTarget = (defaults.repTarget.toInt() + it).toUByte()))
                    },
                    canDecrease = defaults.repTarget > limits.repTargetMin,
                    canIncrease = defaults.repTarget < limits.repTargetMax,
                )
            }
            HairlineDivider(Modifier.padding(start = IntradaSpacing.card))
            Column(
                Modifier.padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
            ) {
                FieldLabel("Metronome starts on")
                SegmentedPills(
                    ClickStart.entries,
                    defaults.click,
                    { save(defaults.copy(click = it)) },
                    label = { it.title },
                    tag = { "profile.click.${it.tagSuffix}" },
                )
            }
            HairlineDivider(Modifier.padding(start = IntradaSpacing.card))
            SessionLengthDefault(defaults, limits, save)
        }
        BasicText(
            "New sessions start with these. Change them in the session.",
            style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
        )
    }
}

@Composable
private fun SessionLengthDefault(
    defaults: PracticeDefaults,
    limits: LimitsView,
    save: (PracticeDefaults) -> Unit,
) {
    val length = defaults.sessionLengthMins?.toInt()
    val step = limits.sessionLengthStepMins.toInt()
    Column(
        Modifier.padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.controlGap)
    ) {
        ToggleRow(
            "Session length",
            on = length != null,
            tag = "profile.sessionLength.toggle",
            onChange = { on ->
                save(
                    defaults.copy(
                        sessionLengthMins = if (on) limits.sessionLengthDefaultMins else null
                    )
                )
            },
        )
        if (length != null) {
            StepperRow(
                "$length min",
                "session length",
                "profile.sessionLength.stepper",
                onStep = {
                    save(defaults.copy(sessionLengthMins = (length + it * step).toUShort()))
                },
                canDecrease = length - step >= limits.sessionLengthMinMins.toInt(),
                canIncrease = length + step <= limits.sessionLengthMaxMins.toInt(),
            )
        }
    }
}

private val ClickStart.title: String
    get() =
        when (this) {
            ClickStart.EVERYBEAT -> "Every beat"
            ClickStart.TWOANDFOUR -> "2 and 4"
        }

private val ClickStart.tagSuffix: String
    get() =
        when (this) {
            ClickStart.EVERYBEAT -> "everyBeat"
            ClickStart.TWOANDFOUR -> "twoAndFour"
        }
