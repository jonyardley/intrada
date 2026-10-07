package com.intrada.android.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.intrada.android.R
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.shared.BuildingSetlistView
import com.intrada.shared.Event
import com.intrada.shared.LimitsView
import com.intrada.shared.PickerVariationView
import com.intrada.shared.SectionView
import com.intrada.shared.SegmentView
import com.intrada.shared.SessionEvent
import com.intrada.shared.SetlistEntryView

@Composable
fun EntrySettingsRoute(
    store: Store,
    entryId: String,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val viewModel by store.viewModel.collectAsState()
    val setlist = viewModel?.buildingSetlist
    val entry = setlist?.entries?.firstOrNull { it.id == entryId }
    val limits = viewModel?.limits
    if (setlist == null || entry == null || limits == null) {
        MissingItem("Item settings", "This item is no longer in the session.", modifier)
        return
    }
    EntrySettingsScreen(
        entry,
        setlist,
        limits,
        send = { store.sendAccepted(Event.Session(it)) },
        onDone = onDone,
        modifier = modifier,
    )
}

@Composable
fun EntrySettingsScreen(
    entry: SetlistEntryView,
    setlist: BuildingSetlistView,
    limits: LimitsView,
    send: (SessionEvent) -> Boolean,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val plannable = setlist.entryVariations.firstOrNull { it.entryId == entry.id }
    ScreenScaffold(
        entry.itemTitle,
        modifier,
        actions = { TextAction("Done", "entrySettings.done", onDone, emphasised = true) },
    ) {
        Column(
            Modifier.fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
        ) {
            AimCard(entry, send)
            setlist.lastTimes
                .firstOrNull { it.entryId == entry.id }
                ?.let { offer ->
                    AddRow(
                        offer.label,
                        "Plan it as last time, ${offer.label}",
                        "entrySettings.lastTime",
                        { send(SessionEvent.ApplyLastTime(entry.id)) },
                        Modifier.cardSurface(),
                    )
                }
            val sections = plannable?.sections.orEmpty()
            if (sections.isNotEmpty()) SectionsCard(entry, sections, send)
            val variations = plannable?.variations.orEmpty()
            if (variations.isNotEmpty()) VariationCard(entry, variations, send)
            RepsCard(entry, limits, send)
            DurationCard(entry, limits, send)
            if (entry.removable) {
                DeleteButton(
                    "Remove from this session",
                    "entrySettings.remove",
                    { if (send(SessionEvent.RemoveFromSetlist(entry.id))) onDone() },
                )
            }
        }
    }
}

@Composable
private fun VariationCard(
    entry: SetlistEntryView,
    variations: List<PickerVariationView>,
    send: (SessionEvent) -> Boolean,
) {
    val chosen = entry.plannedVariationIds.firstOrNull()
    val plan = { id: String? -> send(SessionEvent.SetEntryVariations(entry.id, listOfNotNull(id))) }
    Column(Modifier.cardSurface()) {
        FieldLabel(
            "Variation",
            Modifier.padding(
                horizontal = IntradaSpacing.card,
                vertical = IntradaSpacing.controlGap,
            ),
        )
        TickRow("No variation", null, chosen == null, "entrySettings.variation", { plan(null) })
        variations.forEach { variation ->
            HairlineDivider()
            TickRow(
                variation.label,
                variation.caption.takeIf { it.isNotEmpty() },
                chosen == variation.id,
                "entrySettings.variation",
                { plan(variation.id) },
            )
        }
    }
}

@Composable
private fun AimCard(entry: SetlistEntryView, send: (SessionEvent) -> Boolean) {
    var text by remember(entry.id) { mutableStateOf(entry.intention.orEmpty()) }
    Column(Modifier.cardSurface()) {
        FormField(
            "Aim",
            text,
            { value ->
                text = value
                val next = value.trim().ifEmpty { null }
                if (next != entry.intention) send(SessionEvent.SetEntryIntention(entry.id, next))
            },
            "entrySettings.aim",
            placeholder = "What are you aiming for on this one?",
            singleLine = false,
        )
    }
}

@Composable
private fun SectionsCard(
    entry: SetlistEntryView,
    sections: List<SectionView>,
    send: (SessionEvent) -> Boolean,
) {
    val segments = entry.record.segments
    val unplanned = sections.filter { section -> segments.none { it.sectionId == section.id } }
    Column(
        Modifier.fillMaxWidth()
            .cardSurface()
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.controlGap)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            FieldLabel("Sections", Modifier.weight(1f))
            if (segments.isNotEmpty()) {
                TextAction(
                    "Whole piece",
                    "entrySettings.wholePiece",
                    { send(SessionEvent.SetSegments(entry.id, emptyList())) },
                )
            }
        }
        if (segments.isEmpty()) {
            BasicText(
                "Whole piece",
                Modifier.padding(vertical = IntradaSpacing.controlGap),
                style = IntradaFont.body.copy(color = IntradaColor.ink),
            )
        }
        segments.forEach { segment ->
            SegmentRow(
                entry,
                segment,
                sections,
                timed = entry.plannedDurationSecs != null && segments.size > 1,
                send,
            )
            HairlineDivider()
        }
        if (entry.record.canAddSection) {
            unplanned.forEach { section ->
                AddRow(
                    if (section.isWeakest) "${section.label} · Weakest" else section.label,
                    "Add ${section.label}",
                    "entrySettings.addSection",
                    { send(SessionEvent.AddSegment(entry.id, section.id)) },
                )
            }
        }
    }
}

@Composable
private fun SegmentRow(
    entry: SetlistEntryView,
    segment: SegmentView,
    sections: List<SectionView>,
    timed: Boolean,
    send: (SessionEvent) -> Boolean,
) {
    val section = sections.firstOrNull { it.id == segment.sectionId }
    Row(
        Modifier.fillMaxWidth().padding(vertical = IntradaSpacing.controlGap),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            BasicText(segment.label, style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink))
            section?.barsCaption?.let {
                BasicText(it, style = IntradaFont.small.copy(color = IntradaColor.inkSecondary))
            }
        }
        if (timed) {
            StepperRow(
                segment.plannedDisplay,
                "time on ${segment.label}",
                "entrySettings.segmentMinutes",
                onStep = {
                    send(SessionEvent.StepSegment(entry.id, segment.sectionId, it.toByte()))
                },
                modifier = Modifier.weight(1f),
                canDecrease = segment.canTakeMinute,
                canIncrease = segment.canAddMinute,
            )
        }
        IconAction(
            R.drawable.ic_close,
            "Remove ${segment.label}",
            { send(SessionEvent.RemoveSegment(entry.id, segment.sectionId)) },
            tint = IntradaColor.inkFaintIcon,
        )
    }
}

@Composable
private fun RepsCard(entry: SetlistEntryView, limits: LimitsView, send: (SessionEvent) -> Boolean) {
    val target = entry.plannedRepTarget
    Column(
        Modifier.fillMaxWidth()
            .cardSurface()
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.controlGap)
    ) {
        ToggleRow(
            "Track repetitions",
            on = target != null,
            tag = "entrySettings.trackReps",
            onChange = { on ->
                send(SessionEvent.SetRepTarget(entry.id, if (on) limits.repTargetDefault else null))
            },
        )
        if (target != null) {
            StepperRow(
                "Target: $target repetitions",
                "repetitions",
                "entrySettings.repTarget",
                onStep = {
                    send(SessionEvent.SetRepTarget(entry.id, (target.toInt() + it).toUByte()))
                },
                canDecrease = target > limits.repTargetMin,
                canIncrease = target < limits.repTargetMax,
            )
        }
    }
}

@Composable
private fun DurationCard(
    entry: SetlistEntryView,
    limits: LimitsView,
    send: (SessionEvent) -> Boolean,
) {
    val secs = entry.plannedDurationSecs
    // The core bounds seconds and the stepper offers whole minutes, so both ends round inwards.
    val minMinutes =
        (limits.plannedDurationMinSecs.toInt() + SECONDS_PER_MINUTE - 1) / SECONDS_PER_MINUTE
    val maxMinutes = limits.plannedDurationMaxSecs.toInt() / SECONDS_PER_MINUTE
    Column(
        Modifier.fillMaxWidth()
            .cardSurface()
            .padding(horizontal = IntradaSpacing.card, vertical = IntradaSpacing.controlGap)
    ) {
        ToggleRow(
            "Planned duration",
            on = secs != null,
            tag = "entrySettings.duration",
            onChange = { on ->
                val minutes = limits.plannedDurationDefaultSecs.toInt() / SECONDS_PER_MINUTE
                send(
                    SessionEvent.SetEntryDuration(
                        entry.id,
                        if (on) (minutes * SECONDS_PER_MINUTE).toUInt() else null,
                    )
                )
            },
        )
        if (secs != null) {
            val minutes = secs.toInt() / SECONDS_PER_MINUTE
            StepperRow(
                "$minutes min",
                "minutes",
                "entrySettings.minutes",
                onStep = {
                    send(
                        SessionEvent.SetEntryDuration(
                            entry.id,
                            ((minutes + it) * SECONDS_PER_MINUTE).toUInt(),
                        )
                    )
                },
                canDecrease = minutes > minMinutes,
                canIncrease = minutes < maxMinutes,
            )
        }
    }
}

private const val SECONDS_PER_MINUTE = 60
