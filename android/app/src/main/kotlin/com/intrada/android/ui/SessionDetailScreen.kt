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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.unit.dp
import com.intrada.android.core.Store
import com.intrada.android.ui.components.FieldLabel
import com.intrada.android.ui.components.HairlineDivider
import com.intrada.android.ui.components.cardSurface
import com.intrada.android.ui.components.label
import com.intrada.shared.CompletionStatus
import com.intrada.shared.EntryStatus
import com.intrada.shared.PlayView
import com.intrada.shared.PracticeSessionView
import com.intrada.shared.SetlistEntryView

@Composable
fun SessionDetailRoute(store: Store, id: String, modifier: Modifier = Modifier) {
    val viewModel by store.viewModel.collectAsState()
    val sessions by store.sessionHistory.collectAsState()
    val topMark = viewModel?.limits?.scoreMax?.toInt() ?: return
    val session = sessions.firstOrNull { it.id == id }
    if (session == null) {
        MissingItem("Session", "This session is no longer in your history.", modifier)
        return
    }
    SessionDetailScreen(session, topMark, modifier)
}

/** Read-only: marks and notes are written on the summary screen, and this is the record of them. */
@Composable
fun SessionDetailScreen(session: PracticeSessionView, topMark: Int, modifier: Modifier = Modifier) {
    val subtitle =
        listOfNotNull(
                session.totalDurationSummary,
                session.itemCountDisplay,
                if (session.completionStatus == CompletionStatus.ENDEDEARLY) "ended early"
                else null,
            )
            .joinToString(" · ")
    ScreenScaffold(session.dayLabel, modifier, subtitle = subtitle) {
        Column(
            Modifier.fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(IntradaSpacing.card),
            verticalArrangement = Arrangement.spacedBy(IntradaSpacing.section),
        ) {
            session.sessionScore?.let { SessionScoreCard(it.toInt(), topMark) }
            session.notes
                ?.takeIf { it.isNotEmpty() }
                ?.let { notes ->
                    Column(
                        Modifier.fillMaxWidth().clearAndSetSemantics {
                            contentDescription = "Your note, $notes"
                        },
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        FieldLabel("Your note")
                        BasicText(notes, style = IntradaFont.body.copy(color = IntradaColor.ink))
                    }
                }
            Played(session.entries, topMark)
        }
    }
}

@Composable
private fun SessionScoreCard(score: Int, topMark: Int) {
    Row(
        Modifier.fillMaxWidth()
            .cardSurface()
            .clearAndSetSemantics {
                contentDescription = "How it went, your mark for the session, $score of $topMark"
            }
            .padding(IntradaSpacing.card),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.card),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        ScoreRing(score, topMark)
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            FieldLabel("How it went")
            BasicText(
                "Your mark for the session",
                style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
            )
        }
    }
}

@Composable
private fun Played(entries: List<SetlistEntryView>, topMark: Int) {
    Column(verticalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact)) {
        FieldLabel("What you played", Modifier.testTag("sessionDetail.played"))
        Column {
            entries.forEachIndexed { index, entry ->
                EntryRow(entry, topMark)
                if (index < entries.lastIndex) HairlineDivider()
            }
        }
    }
}

@Composable
private fun EntryRow(entry: SetlistEntryView, topMark: Int) {
    val played = entry.status == EntryStatus.COMPLETED
    val ring = if (played) entry.scoreSummary?.toInt() else null
    val lines = entry.detailLines()
    val spoken =
        (listOf(entry.itemTitle, entry.metaLine()) +
                lines +
                listOfNotNull(
                    ring?.let { "marked $it out of $topMark" },
                    entry.notes?.ifEmpty { null },
                ))
            .joinToString(", ")
    Row(
        Modifier.fillMaxWidth()
            .clearAndSetSemantics { contentDescription = spoken }
            .padding(vertical = IntradaSpacing.cardCompact),
        horizontalArrangement = Arrangement.spacedBy(IntradaSpacing.cardCompact),
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            BasicText(
                entry.itemTitle,
                style =
                    IntradaFont.bodyMedium.copy(
                        color = if (played) IntradaColor.ink else IntradaColor.inkSecondary
                    ),
            )
            BasicText(
                entry.metaLine(),
                style = IntradaFont.small.copy(color = IntradaColor.inkSecondary),
            )
            lines.forEach {
                BasicText(it, style = IntradaFont.small.copy(color = IntradaColor.inkSecondary))
            }
            entry.notes
                ?.takeIf { it.isNotEmpty() }
                ?.let {
                    BasicText(
                        it,
                        Modifier.padding(top = 2.dp),
                        style = IntradaFont.secondary.copy(color = IntradaColor.inkSecondary),
                    )
                }
        }
        ring?.let { ScoreRing(it, topMark, size = 34.dp) }
    }
}

// HACK(#2494): iOS builds these lines in Swift too; the core gap is noted on the issue.
private fun SetlistEntryView.metaLine(): String =
    when (status) {
        EntryStatus.NOTATTEMPTED -> "Not played"
        EntryStatus.SKIPPED -> "Skipped"
        EntryStatus.COMPLETED -> {
            val single = plays.singleOrNull()?.takeIf { it.label == null }
            (listOf(itemType.label, durationDisplay) + single?.metaParts.orEmpty().drop(1))
                .joinToString(" · ")
        }
    }

private fun SetlistEntryView.detailLines(): List<String> {
    val named = plays.singleOrNull()?.takeIf { it.label != null }
    return when {
        status != EntryStatus.COMPLETED -> emptyList()
        plays.size > 1 ->
            plays.map { play ->
                (listOf(play.label ?: "No variation") +
                        play.metaParts +
                        listOfNotNull(play.score?.let { "marked $it" }))
                    .joinToString(" · ")
            }
        named != null ->
            listOf((listOfNotNull(named.label) + named.metaParts.drop(1)).joinToString(" · "))
        else -> emptyList()
    }
}

private val PlayView.metaParts: List<String>
    get() =
        listOfNotNull(
            durationDisplay,
            achievedTempo?.let { "$it bpm" },
            repTarget?.let { "${repCount ?: 0u} of $it reps" },
        )
