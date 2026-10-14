package com.intrada.android.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.unit.dp
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.CompletionStatus
import com.intrada.shared.EntryStatus
import com.intrada.shared.Event
import com.intrada.shared.SessionEvent
import com.intrada.shared.SetlistEntryView
import com.intrada.shared.SummaryView

@Composable
fun SummaryScreen(model: SummaryModel, send: (Event) -> Boolean, modifier: Modifier = Modifier) {
    val summary = model.summary
    val range = model.limits.scoreMin.toInt()..model.limits.scoreMax.toInt()
    var confirmingDiscard by remember { mutableStateOf(false) }
    BackHandler { confirmingDiscard = true }
    ScreenScaffold("Session complete", modifier) {
        Column(Modifier.fillMaxSize()) {
            Column(
                Modifier.weight(1f)
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(IntradaSpacing.card),
                verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
            ) {
                Headline(summary)
                summary.topMover?.let {
                    BasicText(
                        "${it.itemTitle} moved up · ${it.previousScore ?: 0u} to ${it.currentScore}",
                        Modifier.fillMaxWidth().cardSurface().padding(IntradaSpacing.cardCompact),
                        style = IntradaFont.bodyMedium.copy(color = IntradaColor.ink),
                    )
                }
                Column(
                    Modifier.fillMaxWidth().cardSurface().padding(horizontal = IntradaSpacing.card)
                ) {
                    summary.entries.forEachIndexed { index, entry ->
                        if (index > 0) HairlineDivider()
                        EntryRow(entry, range, send)
                    }
                }
                Closing(summary, range, send, onDiscard = { confirmingDiscard = true })
            }
        }
    }
    if (confirmingDiscard) {
        ConfirmDialog(
            ConfirmCopy(
                "Discard this session?",
                "This practice won't be saved.",
                "Discard",
                "summary.discardConfirm",
            ),
            onConfirm = {
                confirmingDiscard = false
                send(Event.Session(SessionEvent.DiscardSession))
            },
            onDismiss = { confirmingDiscard = false },
        )
    }
}

@Composable
private fun Closing(
    summary: SummaryView,
    range: IntRange,
    send: (Event) -> Boolean,
    onDiscard: () -> Unit,
) {
    var note by remember { mutableStateOf(summary.notes.orEmpty()) }
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section)) {
        FormField(
            "",
            note,
            { value ->
                note = value
                if (value != summary.notes.orEmpty()) {
                    send(Event.Session(SessionEvent.UpdateSessionNotes(value)))
                }
            },
            "summary.note",
            Modifier.cardSurface(),
            placeholder = "A note on the whole session…",
            singleLine = false,
        )
        Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap)) {
            FieldLabel("Overall")
            ScoreSelector(
                summary.sessionScore?.toInt() ?: 0,
                range,
                "Overall session mark",
                "summary.overall",
                onSelect = { send(Event.Session(SessionEvent.UpdateSessionScore(it))) },
            )
        }
        Column(
            Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
        ) {
            InkButton(
                "Save session",
                "summary.save",
                { send(Event.Session(SessionEvent.SaveSession(SessionClock.now()))) },
            )
            TextAction("Discard", "summary.discard", onDiscard)
        }
    }
}

@Composable
private fun Headline(summary: SummaryView) {
    val base = "${summary.completedCount} of ${summary.entries.size}"
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        BasicText(
            summary.totalDurationDisplay,
            Modifier.testTag("summary.total"),
            style = IntradaFont.timer().copy(color = IntradaColor.ink),
        )
        BasicText(
            if (summary.completionStatus == CompletionStatus.ENDEDEARLY) "$base · ended early"
            else base,
            style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
        )
    }
}

@Composable
private fun EntryRow(entry: SetlistEntryView, range: IntRange, send: (Event) -> Boolean) {
    val unfinished = entry.status == EntryStatus.NOTATTEMPTED
    Column(
        Modifier.padding(vertical = IntradaSpacing.cardCompact),
        verticalArrangement = Arrangement.spacedBy(IntradaSpacing.controlGap),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                BasicText(
                    entry.itemTitle,
                    style =
                        IntradaFont.bodyMedium.copy(
                            color = if (unfinished) IntradaColor.inkSecondary else IntradaColor.ink
                        ),
                )
                BasicText(
                    entry.metaLine(unfinished),
                    style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
                )
                entry.intention?.takeIf { it.isNotEmpty() }?.let { AimLine(it) }
            }
            if (!unfinished) {
                BasicText(
                    entry.durationDisplay,
                    style = IntradaFont.figure.copy(color = IntradaColor.inkSecondary),
                )
            }
        }
        if (entry.status == EntryStatus.COMPLETED) {
            entry.plays.forEach { play ->
                if (entry.plays.size > 1) {
                    BasicText(
                        play.label ?: "No variation",
                        style = IntradaFont.secondary.copy(color = IntradaColor.ink),
                    )
                }
                ScoreSelector(
                    play.score?.toInt() ?: 0,
                    range,
                    "Mark for ${play.label ?: entry.itemTitle}",
                    "summary.mark",
                    onSelect = {
                        send(Event.Session(SessionEvent.UpdateEntryScore(entry.id, play.id, it)))
                    },
                )
            }
        }
        if (!unfinished) EntryNote(entry, send)
    }
}

@Composable
private fun AimLine(aim: String) {
    BasicText(
        "\u201C$aim\u201D",
        style =
            IntradaFont.small.copy(
                color = IntradaColor.inkSecondary,
                fontStyle = FontStyle.Italic,
            ),
    )
}

/** A written note stays on the row: a note the musician cannot see again is admin (T7). */
@Composable
private fun EntryNote(entry: SetlistEntryView, send: (Event) -> Boolean) {
    val written = entry.notes.orEmpty()
    var editing by remember(entry.id) { mutableStateOf(false) }
    var text by remember(entry.id) { mutableStateOf(written) }
    if (editing) {
        val focus = remember { FocusRequester() }
        DisposableEffect(Unit) {
            focus.requestFocus()
            onDispose {}
        }
        FormField(
            "",
            text,
            { value ->
                text = value
                // The field must not keep showing text the core refused (an over-long note).
                if (
                    value != written &&
                        !send(Event.Session(SessionEvent.UpdateEntryNotes(entry.id, value)))
                ) {
                    text = written
                }
            },
            "summary.entryNote",
            Modifier.focusRequester(focus).cardSurface(IntradaRadius.control).semantics {
                contentDescription = "Note for ${entry.itemTitle}"
            },
            placeholder = "Anything worth remembering",
            singleLine = false,
        )
    } else {
        val open = {
            text = written
            editing = true
        }
        Box(
            Modifier.fillMaxWidth()
                .heightIn(min = 48.dp)
                .buttonSemantics(
                    if (written.isEmpty()) "Add a note for ${entry.itemTitle}"
                    else "Note for ${entry.itemTitle}, $written",
                    "summary.addNote",
                    open,
                ),
            contentAlignment = Alignment.CenterStart,
        ) {
            BasicText(
                written.ifEmpty { "Add a note" },
                style =
                    IntradaFont.small.copy(
                        color =
                            if (written.isEmpty()) IntradaColor.accent
                            else IntradaColor.inkSecondary
                    ),
            )
        }
    }
}

private fun SetlistEntryView.metaLine(unfinished: Boolean): String {
    if (unfinished) return "Saved for next time"
    val tempo = plays.singleOrNull()?.achievedTempo?.let { "$it bpm" }
    return listOfNotNull(itemType.label, tempo).joinToString(" · ")
}
